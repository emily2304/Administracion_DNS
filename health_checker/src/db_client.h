#ifndef DB_CLIENT_H
#define DB_CLIENT_H
 
#include "config.h"

// Tipo de health check configurado para un target
typedef enum {
    CHECK_TCP,
    CHECK_HTTP
} CheckType;

// Tipos de registro DNS soportados 
// TODO: Revisar que coincidan con los del DNS API
typedef enum {
    RECORD_SINGLE,
    RECORD_MULTI,
    RECORD_WEIGHT,
    RECORD_GEO,
    RECORD_ROUND_TRIP,
    RECORD_UNKNOWN
} RecordType;

// Inicialmente se va a usar un array de un size fijo en lugar de memoria dinámica
// En caso de que se presente un crecimiento indefinido, es mejor cambiar a mem. dinámica
// Se usa 64 porque es un valor razonable para un entorno de pruebas
#define MAX_TARGETS 64 

// Configuración de health check para un target (IP) específico
typedef struct {
    char record_ip_id[64];
    char dns_record_id[64]; // Referencia al registro DNS (la clave foranea hacia la otra tabla)
    char host[256];
    RecordType record_type;
    char ip[64];
    int weight; // Se usa si record_type == RECORD_WEIGHT
    char country[64]; // Si record_type == RECORD_GEO

    CheckType check_type;
    int port; // Para el TCP check
    char http_path[256]; // Para el HTTP check
    int expected_http_codes[8];
    int expected_http_codes_count;
    char basic_auth_user[64]; // queda vacío en caso de que no aplique
    char basic_auth_pass[64];
 
    int timeout_sec;
    int retries;
    int interval_sec;
} HealthTarget;

// Trae la lista de targets a checkear desde supabase
// Retorna la cantidad de targets obtenida. En caso de error retorna -1.
int db_fetch_targets(const AppConfig *config, HealthTarget targets[MAX_TARGETS]);

// Escribe el resultado de un health check 
/*
Establece si es healthy/unhealthy
Agrega el RTT
Ambos valores en supabase
*/
int db_write_result(const AppConfig *config, const HealthTarget *target,
                     int is_healthy, double rtt_ms);

// Registra la ubicación del Health Checker en la tabla de health_checker_locations
int db_upsert_checker_location(const AppConfig *config);

#endif