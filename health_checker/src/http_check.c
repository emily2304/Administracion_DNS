#include <stdio.h>
#include <string.h>
#include <time.h>
#include <curl/curl.h>

#include "http_check.h"

// Devuelve el tiempo actual en milisegundos 
// Es monotónico, para medir RTT
static double get_time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1000.0 + (double)ts.tv_nsec / 1e6;
}

// Función para descartar el contenido del body de la respuesta HTTP
/*
ptr -> puntero a los datos recibidos
size -> tamaño de cada bloque de datos.
nmemb -> número de bloques de datos.
userdata -> puntero a datos de usuario.
*/
static size_t discard_body(void *ptr, size_t size, size_t nmemb, void *userdata) {
    (void)ptr;
    (void)userdata;
    return size * nmemb; // Cantidad total de bytes procesados
}

// Se revisa que el código de estado HTTP retornado coincide con los esperados por el target.
static int is_expected_code(const HealthTarget *target, long code) {
    if (target->expected_http_codes_count == 0) {
        return code == 200; // 200 = OK, respuesta válida por defecto en caso de que no se defina
    }

    // Se busca si el código obtenido coincide con los del target
    for (int i = 0; i < target->expected_http_codes_count; i++) {
        if (target->expected_http_codes[i] == (int)code) {
            return 1;
        }
    }
    return 0;
}

// Ejecuta un intento de health check HTTP.
/*
0: unhealthy
1: healthy
*/
int check_http_run(const HealthTarget *target, double *rtt_ms) {
    *rtt_ms = 0.0;

    // sesión de libcurl
    CURL *curl = curl_easy_init();
    if (curl == NULL) {
        return 0; // error al inicializar libcurl
    }

    // Construcción de la URL
    char url[512];
    if (target->port > 0) {
        snprintf(url, sizeof(url), "http://%s:%d%s",
                 target->ip, target->port, target->http_path);
    } else {
        snprintf(url, sizeof(url), "http://%s%s", target->ip, target->http_path);
    }

    // Configuración de opciones básicas para la petición
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, (long)target->timeout_sec);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, discard_body);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    
    // Configuración del Basic auth (si se definieron en el target las credenciales)
    if (target->basic_auth_user[0] != '\0') {
        curl_easy_setopt(curl, CURLOPT_HTTPAUTH, CURLAUTH_BASIC);
        curl_easy_setopt(curl, CURLOPT_USERNAME, target->basic_auth_user);
        curl_easy_setopt(curl, CURLOPT_PASSWORD, target->basic_auth_pass);
    }
    
    // Ejecutar la transferencia HTTP
    double start = get_time_ms();
    CURLcode res = curl_easy_perform(curl);
    *rtt_ms = get_time_ms() - start;

    int healthy = 0;
    
    // Verificación de exito
    if (res == CURLE_OK) {
        long http_code = 0;
        // Se extrae el código de estado retornado por el servidor web
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
        healthy = is_expected_code(target, http_code);
    } else { // errores a nivel de red/transporte
        fprintf(stderr, "[check_http] error contra %s: %s\n",
                url, curl_easy_strerror(res));
    }

    curl_easy_cleanup(curl);
    return healthy;
}