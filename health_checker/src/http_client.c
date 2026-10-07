#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <curl/curl.h>

#include "http_client.h"

// Callback de libcurl, se va acumulando cada chunk recibido en el buffer dinámico
/*
Parametros:
    - contents Puntero a los datos recibidos en el chunk actual.
    - size Tamaño del elemento de datos (típicamente 1 byte).
    - nmemb Número de elementos recibidos.
    - userdata Puntero que se castea a HttpResponse.

Return:
    - size_t Cantidad real de bytes procesados.
*/
static size_t write_callback(void *contents, size_t size, size_t nmemb, void *userdata) {
    HttpResponse *response = (HttpResponse *)userdata;
    size_t chunk_size = size * nmemb;
    
    // Intento de reasignar memoria dinámicamente
    char *new_data = realloc(response->data, response->size + chunk_size + 1);
    if (new_data == NULL) {
        return 0; // ERROR, se notifica a libcurl
    }

    response->data = new_data;
    memcpy(response->data + response->size, contents, chunk_size);
    response->size += chunk_size;
    response->data[response->size] = '\0'; // String adapatado para parsear a JSON

    return chunk_size;
}

// Función para liberar memoria dinámica
void http_response_free(HttpResponse *response) {
    free(response->data);
    response->data = NULL;
    response->size = 0;
}

// Construye los headers de autenticación que espera Supabase
// Para el api_key se utilizara el anon token 
static struct curl_slist *build_auth_headers(const char *api_key) {
    struct curl_slist *headers = NULL;
    
    // Primer header
    char apikey_header[512];
    snprintf(apikey_header, sizeof(apikey_header), "apikey: %s", api_key);
    headers = curl_slist_append(headers, apikey_header);

    // Segundo header para evaluar RLS
    char auth_header[512];
    snprintf(auth_header, sizeof(auth_header), "Authorization: Bearer %s", api_key);
    headers = curl_slist_append(headers, auth_header);
 
    return headers;
}

/*
- Configura los headers
- Hace la petición GET
- Llena la estructura HttpResponse con el cuerpo JSON
 */
// Se va a usar para db_fetch_targets
long http_get_supabase(const char *url, const char *api_key, HttpResponse *response) {
    // Se inicializa la respuesta
    response->data = NULL;
    response->size = 0;

    CURL *curl = curl_easy_init();
    if (curl == NULL) {
        return -1;
    }

    struct curl_slist *headers = build_auth_headers(api_key);

    // Config. de opciones para la sesión de curl
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, response);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 15L); // 15 segundos para el timeout

    CURLcode res = curl_easy_perform(curl);

    long http_code = -1;
    if (res == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
    } else {
        fprintf(stderr, "[http_client] GET error contra %s: %s\n",
                url, curl_easy_strerror(res));
    }

    // Se liberan los recursos
    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return http_code;
}

static long send_json_request(const char *method, const char *url, const char *api_key,
                               const char *json_body, const char *prefer) {
    CURL *curl = curl_easy_init();
    if (curl == NULL) {
        return -1;
    }
    
    // Construcción de headers de auth y tipos específicos
    struct curl_slist *headers = build_auth_headers(api_key);
    headers = curl_slist_append(headers, "Content-Type: application/json");

    char prefer_header[256];
    snprintf(prefer_header, sizeof(prefer_header), "Prefer: %s", prefer);
    headers = curl_slist_append(headers, prefer_header);
    
    // Buffer para capturar posibles mensajes de error
    HttpResponse response = {NULL, 0};
    
    // Config. de opciones para la sesión de curl
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, method);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, json_body);
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 15L);

    CURLcode res = curl_easy_perform(curl);
    
    // ERROR
    long http_code = -1;
    if (res == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
        if (http_code >= 400 && response.data != NULL) {
            fprintf(stderr, "[http_client] %s %s -> %ld: %s\n",
                    method, url, http_code, response.data);
        }
    } else {
        fprintf(stderr, "[http_client] %s error contra %s: %s\n",
                method, url, curl_easy_strerror(res));
    }
 
    http_response_free(&response);
    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
 
    return http_code;
}

/*
- Hace un PATCH a "url" con el cuerpo ya serializado
- Retorna el código de estado HTTP, o -1 en caso de error
 */
// Se va a usar para db_write_result
long http_patch_supabase(const char *url, const char *api_key, const char *json_body)
{
    return send_json_request("PATCH", url, api_key, json_body, "return=minimal");
}

/*
- Hace un POST a "url" con el cuerpo ya serializado
- Retorna el código de estado HTTP, o -1 en caso de error
 */
// Se va a usar para db_write_result
long http_upsert_supabase(const char *url, const char *api_key, const char *json_body)
{
    return send_json_request("POST", url, api_key, json_body,
                             "resolution=merge-duplicates,return=minimal");
}