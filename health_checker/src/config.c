#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "config.h"

// Funciones para leer variables de entorno de forma segura y poder manejar casos de NULL:
/*
- read_env_str
- read_env_int
- read_env_double
*/

// Leer strings
// Si la variable no existe imprime un error y retorna -1.
static int read_env_str(const char *name, char *dest, size_t dest_size) {
    const char *env_value = getenv(name);
    if (env_value == NULL) {
        fprintf(stderr, "[config] Falta variable de entorno: %s\n", name);
        return -1;
    }
    strncpy(dest, env_value, dest_size - 1);
    dest[dest_size - 1] = '\0';
    return 0;
}

// Leer enteros
// Se asigna un valor default si la variable no existe
static int read_env_int(const char *name, int *dest, int default_value) {
    const char *env_value = getenv(name);
    if (env_value == NULL) {
        *dest = default_value;
        return 0;
    }
    *dest = atoi(env_value);
    return 0;
}

// Leer double
// Se asigna un valor default si la variable no existe
static int read_env_double(const char *name, double *dest, double default_value) {
    const char *env_value = getenv(name);
    if (env_value == NULL) {
        *dest = default_value;
        return 0;
    }
    *dest = atof(env_value);
    return 0;
}

int config_load(AppConfig *config) {
    int status = 0;
    
    // |= para evaluar bit a bit
    status |= read_env_str("SUPABASE_URL", config->supabase_url, sizeof(config->supabase_url));
    status |= read_env_str("SUPABASE_KEY", config->supabase_key, sizeof(config->supabase_key));
 
    read_env_int("POLL_INTERVAL_SEC", &config->poll_interval_sec, 30);
 
    status |= read_env_str("CHECKER_ID", config->checker_id, sizeof(config->checker_id));
    read_env_double("CHECKER_LATITUDE", &config->lat, 0.0);
    read_env_double("CHECKER_LONGITUDE", &config->longitude, 0.0);
    status |= read_env_str("CHECKER_COUNTRY", config->country, sizeof(config->country));
    status |= read_env_str("CHECKER_CITY", config->city, sizeof(config->city));
 
    return status;
}

void config_print(const AppConfig *config) {
    printf("=== Health Checker Config ===\n");
    printf("checker_id       : %s\n", config->checker_id);
    printf("supabase_url     : %s\n", config->supabase_url);
    printf("poll_interval_sec: %d\n", config->poll_interval_sec);
    printf("location         : %s, %s (lat=%.4f, lon=%.4f)\n",
           config->city, config->country, config->lat, config->longitude);
    printf("=== === === === === === === === ===\n");
}