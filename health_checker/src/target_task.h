#ifndef TARGET_TASK_H
#define TARGET_TASK_H

#include "config.h"
#include "db_client.h"

// Struct con los argumentos para el procesamiento de un target
typedef struct {
    const AppConfig *config;
    const HealthTarget *target;
} TargetTaskArgs;

// Ejecuta cada thread
// Funciona sobre su target asignado y escribe el resultado con db_write_result()
void *target_task_run(void *arg);

#endif