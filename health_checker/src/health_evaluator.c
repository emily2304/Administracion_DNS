#include <stdio.h>
#include "health_evaluator.h"
#include "tcp_check.h"
#include "http_check.h"

// Se evalua la salud de un HealthTarget ejecutando reintentos y aplicando un algoritmo de votación por mayoría simple.
// Determina si es unhealthy/healthy y se calucla el RTT de las pruebas exitosas
/*
0: unhealthy
1: healthy
*/
int health_evaluator_run(const HealthTarget *target, double *avg_rtt_ms) {
    int attempts = target->retries > 0 ? target->retries : 1; // Si no se configuró un mínimo de intentos, por defecto es 1

    int success_count = 0;
    double rtt_sum = 0.0;
 
    for (int i = 0; i < attempts; i++) {
        double rtt_ms = 0.0;
        int ok;
 
        if (target->check_type == CHECK_TCP) {
            ok = check_tcp_run(target, &rtt_ms);
        } else {
            ok = check_http_run(target, &rtt_ms);
        }
        
        // Ver en consola el resultado individual del intento actual
        printf("[health_evaluator] %s intento %d/%d -> %s (%.2f ms)\n",
               target->record_ip_id, i + 1, attempts,
               ok ? "ok" : "fallo", rtt_ms);
            
        // Si la prueba tuvo exito se aumenta el contador y cuenta para el promedio de RTT
        if (ok) {
            success_count++;
            rtt_sum += rtt_ms;
        }
    }
 
    // se considera que cumple con mayoría simple si más de la mitad de los intentos son exitosos
    int is_healthy = success_count > attempts / 2;
    
    // Promedio del rtt, se dejan por fuera los intentos no exitosos para evitar sesgos por timeout
    *avg_rtt_ms = (success_count > 0) ? (rtt_sum / success_count) : 0.0;
    
    // Resultado en consola
    printf("[health_evaluator] %s resultado final: %s (%d/%d exitosos)\n",
           target->record_ip_id, is_healthy ? "healthy" : "unhealthy",
           success_count, attempts);
 
    return is_healthy;
}