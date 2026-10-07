#include <stdio.h>
#include <pthread.h>

#include "target_task.h"
#include "health_evaluator.h"

// Función ejecutada por cada hilo (pthread) para procesar un objetivo.
/*
Se delega la ejecución de la prueba al health_evaluator y luego se escribe en la BD.
*/
void *target_task_run(void *arg) {
    // Cast al tipo de estructura del task
    TargetTaskArgs *args = (TargetTaskArgs *)arg;

    // En caso de que se reciba un NULL
    if (args == NULL || args->target == NULL) {
        return NULL;
    }

    double avg_rtt_ms = 0.0;

    // Se ejecuta la evaluación de health con retries y con la votación por mayoría simple
    int is_healthy = health_evaluator_run(args->target, &avg_rtt_ms);

    // Se escribe el estado final (healthy/unhealthy) y el RTT promedio en la BD
    db_write_result(args->config, args->target, is_healthy, avg_rtt_ms);

    return NULL;
}