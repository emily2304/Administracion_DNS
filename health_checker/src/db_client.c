#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <cjson/cJSON.h>
#include "http_client.h"
 
#include "db_client.h"

// === === === === === 
//      Helpers
// === === === === === 

// Convierte el texto de record_type (la columna de dns_records) al enum RecordType
static RecordType parse_record_type(const char *text) {
    if (text == NULL) return RECORD_UNKNOWN;
    if (strcmp(text, "single") == 0) return RECORD_SINGLE;
    if (strcmp(text, "multi") == 0) return RECORD_MULTI;
    if (strcmp(text, "weight") == 0) return RECORD_WEIGHT;
    if (strcmp(text, "geo") == 0) return RECORD_GEO;
    if (strcmp(text, "round-trip") == 0) return RECORD_ROUND_TRIP;
    return RECORD_UNKNOWN;
}

// Convierte el texto de check_type (la columna de health_check_configs) al enum CheckType 
static CheckType parse_check_type(const char *text) {
    if (text != NULL && strcmp(text, "http") == 0) {
        return CHECK_HTTP;
    }
    return CHECK_TCP; // default
}

// Copia un string de un objeto cJSON a un buffer.
static void copy_json_string(const cJSON *obj, const char *key, char *dest, size_t dest_size) {
    const cJSON *field = cJSON_GetObjectItemCaseSensitive(obj, key);
    dest[0] = '\0';
    if (cJSON_IsString(field) && field->valuestring != NULL) {
        strncpy(dest, field->valuestring, dest_size - 1);
        dest[dest_size - 1] = '\0';
    }
}

// Obtiene un int de una propiedad cJSON.
static int get_json_int(const cJSON *obj, const char *key, int default_value) {
    const cJSON *field = cJSON_GetObjectItemCaseSensitive(obj, key);
    if (cJSON_IsNumber(field)) {
        return field->valueint;
    }
    return default_value;
}

// Extrae la configuración de health check desempaquetándola si viene en formato array o simple también
static const cJSON *unwrap_health_check_config(const cJSON *field) {
    if (cJSON_IsArray(field)) {
        return cJSON_GetArrayItem(field, 0);
    }
    return field; // objeto suelto o NULL si no viene
}

// Parseo de un único elemento de dns_record_ips a un HealthTarget
static void parse_target_from_json(const cJSON *item, HealthTarget *target) {
    memset(target, 0, sizeof(HealthTarget));

    // Atributos de la tabla de dns_record_ips
    copy_json_string(item, "id", target->record_ip_id, sizeof(target->record_ip_id));
    copy_json_string(item, "record_id", target->dns_record_id, sizeof(target->dns_record_id));
    copy_json_string(item, "ip", target->ip, sizeof(target->ip));
    copy_json_string(item, "country", target->country, sizeof(target->country));
    target->weight = get_json_int(item, "weight", 0);

    // Embedding: dns_records
    const cJSON *dns_record = cJSON_GetObjectItemCaseSensitive(item, "dns_records");
    if (cJSON_IsObject(dns_record)) {
        copy_json_string(dns_record, "host", target->host, sizeof(target->host));
 
        char record_type_str[32] = {0};
        copy_json_string(dns_record, "record_type", record_type_str, sizeof(record_type_str));
        target->record_type = parse_record_type(record_type_str);
    }

    // Embedding: health_check_configs
    // Parámetros del Check (HTTP/TCP)
    const cJSON *config_field = cJSON_GetObjectItemCaseSensitive(item, "health_check_configs");
    const cJSON *config = unwrap_health_check_config(config_field);

    if (cJSON_IsObject(config)) {
        char check_type_str[16] = {0};
        copy_json_string(config, "check_type", check_type_str, sizeof(check_type_str));
        target->check_type = parse_check_type(check_type_str);

        target->port = get_json_int(config, "port", 0);
        copy_json_string(config, "http_path", target->http_path, sizeof(target->http_path));
        copy_json_string(config, "basic_auth_user", target->basic_auth_user, sizeof(target->basic_auth_user));
        copy_json_string(config, "basic_auth_pass", target->basic_auth_pass, sizeof(target->basic_auth_pass));
        target->timeout_sec = get_json_int(config, "timeout_sec", 5);
        target->retries = get_json_int(config, "retries", 3);
        target->interval_sec = get_json_int(config, "interval_sec", 30);
        
        // Mapeo del arreglo de códigos HTTP esperados (como por ejemplo: [200, 204, 301])
        const cJSON *codes = cJSON_GetObjectItemCaseSensitive(config, "expected_http_codes");
        target->expected_http_codes_count = 0;
        if (cJSON_IsArray(codes)) {
            int count = 0;
            const cJSON *code_item = NULL;
            cJSON_ArrayForEach(code_item, codes) {
                if (count >= 8) break; // límite del array fijo en HealthTarget
                if (cJSON_IsNumber(code_item)) {
                    target->expected_http_codes[count++] = code_item->valueint;
                }
            }
            target->expected_http_codes_count = count;
        }
    } else {
        // Fallback
        // Si no tiene una fila en health_check_configs, se asume que es TCP con algunos defaults
        target->check_type = CHECK_TCP;
        target->timeout_sec = 5;
        target->retries = 3;
        target->interval_sec = 30;
    }
}

// === === === === === 
//      Funciones principales
// === === === === === 

int db_fetch_targets(const AppConfig *config, HealthTarget targets[MAX_TARGETS]) {
    char url[1024];
    // Petición con para traer todo en 1 solo RTT
    snprintf(url, sizeof(url),
             "%s/rest/v1/dns_record_ips"
             "?select=id,record_id,ip,weight,country,"
             "dns_records(host,record_type),"
             "health_check_configs(check_type,port,http_path,expected_http_codes,"
             "basic_auth_user,basic_auth_pass,timeout_sec,retries,interval_sec)",
             config->supabase_url);

    HttpResponse response;
    long status = http_get_supabase(url, config->supabase_key, &response);

    if (status != 200) {
        fprintf(stderr, "[db_client] db_fetch_targets: status HTTP inesperado (%ld). Body: %s\n",
                status, response.data != NULL ? response.data : "(vacío)");
        http_response_free(&response);
        return -1;
    }

    // Parseo del body resultante
    cJSON *root = cJSON_Parse(response.data);
    http_response_free(&response); // Se libera luego de que es parseado

    if (root == NULL || !cJSON_IsArray(root)) {
        fprintf(stderr, "[db_client] db_fetch_targets: la respuesta no es un array JSON válido\n");
        cJSON_Delete(root);
        return -1;
    }

    int count = 0;
    const cJSON *item = NULL;
    cJSON_ArrayForEach(item, root) {
        if (count >= MAX_TARGETS) {
            fprintf(stderr, "[db_client] Se alcanzó MAX_TARGETS (%d), entonces se ignoran registros adicionales\n",
                    MAX_TARGETS);
            break;
        }
        parse_target_from_json(item, &targets[count]);
        count++;
    }

    cJSON_Delete(root);
    return count;
}

// Serializa el body, hace el upsert contra url y finalmente, libera los recursos.
static int upsert_and_free(const AppConfig *config, const char *url, cJSON *body, const char *what) {
    char *json_body = cJSON_PrintUnformatted(body);
    long status = http_upsert_supabase(url, config->supabase_key, json_body);

    int result = 0;
    if (status < 200 || status >= 300) {
        fprintf(stderr, "[db_client] %s: status HTTP inesperado (%ld)\n", what, status);
        result = -1;
    }

    free(json_body);
    cJSON_Delete(body);
    return result;
}

int db_write_result(const AppConfig *config, const HealthTarget *target, int is_healthy, double rtt_ms) {
    if (target->record_ip_id[0] == '\0') {
        fprintf(stderr, "[db_client] db_write_result: el target no tiene record_ip_id, no se puede actualizar\n");
        return -1;
    }

    // Timestamp actual en UTC que sea compatible con timestamptz
    time_t now = time(NULL);
    struct tm tm_utc;
    gmtime_r(&now, &tm_utc);
    char timestamp[32];
    strftime(timestamp, sizeof(timestamp), "%Y-%m-%dT%H:%M:%SZ", &tm_utc);

    // Construcción del payload (JSON) para el PATCH
    cJSON *body = cJSON_CreateObject();
    cJSON_AddBoolToObject(body, "is_healthy", is_healthy);
    cJSON_AddStringToObject(body, "last_checked_at", timestamp);
    cJSON_AddNumberToObject(body, "rtt_ms", rtt_ms);
    cJSON_AddStringToObject(body, "checker_id", config->checker_id);

    char *json_body = cJSON_PrintUnformatted(body);

    // Endpoint enfocado a la IP concreta
    // Se usa un filtro
    char url[1024];
    snprintf(url, sizeof(url), "%s/rest/v1/dns_record_ips?id=eq.%s",
             config->supabase_url, target->record_ip_id);

    long status = http_patch_supabase(url, config->supabase_key, json_body);

    int result = 0;
    if (status < 200 || status >= 300) {
        fprintf(stderr, "[db_client] db_write_result: status HTTP inesperado (%ld) para %s\n",
                status, target->record_ip_id);
        result = -1;
    }

    // Se libera la memoria
    free(json_body);
    cJSON_Delete(body);

    // Para el round trip
    // UPSERT
    cJSON *measurement = cJSON_CreateObject();
    cJSON_AddStringToObject(measurement, "checker_id", config->checker_id);
    cJSON_AddStringToObject(measurement, "record_ip_id", target->record_ip_id);

    if (is_healthy) {
        cJSON_AddNumberToObject(measurement, "rtt_ms", rtt_ms);
    } else {
        // Se guarda un NULL en lugar de un 0 para evitar confusiones
        cJSON_AddNullToObject(measurement, "rtt_ms");
    }

    cJSON_AddBoolToObject(measurement, "is_healthy", is_healthy);
    cJSON_AddStringToObject(measurement, "checked_at", timestamp);
 
    snprintf(url, sizeof(url),
             "%s/rest/v1/health_checker_measurements?on_conflict=checker_id,record_ip_id",
             config->supabase_url);

    if (upsert_and_free(config, url, measurement, "db_write_result (measurements)") != 0) {
        result = -1;
    }

    return result;

}

// Registra la ubicación del Health Checker en la tabla de health_checker_locations
int db_upsert_checker_location(const AppConfig *config) {
    // Timestamp actual en UTC que sea compatible con timestamptz
    time_t now = time(NULL);
    struct tm tm_utc;
    gmtime_r(&now, &tm_utc);
    char timestamp[32];
    strftime(timestamp, sizeof(timestamp), "%Y-%m-%dT%H:%M:%SZ", &tm_utc);

    // Creación del cuerpo
    cJSON *body = cJSON_CreateObject();
    cJSON_AddStringToObject(body, "checker_id", config->checker_id);
    cJSON_AddNumberToObject(body, "latitude", config->lat);
    cJSON_AddNumberToObject(body, "longitude", config->longitude);
    cJSON_AddStringToObject(body, "country", config->country);
    cJSON_AddStringToObject(body, "city", config->city);
    cJSON_AddStringToObject(body, "updated_at", timestamp);

    char url[1024];
    snprintf(url, sizeof(url), "%s/rest/v1/health_checker_locations?on_conflict=checker_id",
             config->supabase_url);

    return upsert_and_free(config, url, body, "db_upsert_checker_location");
}