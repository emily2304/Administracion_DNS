#ifndef CONFIG_H
#define CONFIG_H

/*
Configuración del Health Checker
Se lee desde variables de entorno.
*/
typedef struct {
    char supabase_url[256]; // URL de Supabase
    char supabase_key[256]; // API key
 
    int poll_interval_sec; // intervalo que define cada cuánto se ejecutan los checks
 
    // Ubicación simulada del Health Checker
    char checker_id[64];
    double lat; // latitud
    double longitude; // longitud
    char country[64];
    char city[64];
} AppConfig;

/* Funciones */

// Carga la configuración desde variables de entorno.
int config_load(AppConfig *config);
 
// Imprime la config cargada
// Usada para debug
void config_print(const AppConfig *config);

#endif 