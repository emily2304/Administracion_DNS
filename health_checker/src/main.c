#include <stdio.h>
#include <unistd.h>
#include <pthread.h>
#include "config.h"
#include "db_client.h"
#include "health_evaluator.h"
#include "target_task.h"

// Se ejecuta el Health Checker
int main(void) {
    AppConfig config;

    // Se cargan las env variables
    if (config_load(&config) != 0) {
        fprintf(stderr, "[main] Configuración incompleta. Saliendo.\n");
        return 1;
    }
    
    // Print para verificar la config.
    config_print(&config);

    printf("[main] Health Checker iniciado. Intervalo de polling: %d s\n",
           config.poll_interval_sec);

    while (1) {

        if (db_upsert_checker_location(&config) != 0) {
            fprintf(stderr, "[main] No se pudo registrar la ubicación del checker %s\n",
                    config.checker_id);
        }

        HealthTarget targets[MAX_TARGETS];
        // Obtener la lista de objetivos a evaluar desde Supabase.
        int count = db_fetch_targets(&config, targets);

        if (count < 0) {
            fprintf(stderr, "[main] Error al obtener targets desde la BD.\n");
        } else {
            printf("[main] Targets obtenidos: %d\n", count);

            pthread_t threads[MAX_TARGETS];
            TargetTaskArgs target_task_args[MAX_TARGETS];

            // Se lanza un hilo por cada target para que se revisen en paralelo
            for (int i = 0; i < count; i++) {
                target_task_args[i].config = &config;
                target_task_args[i].target = &targets[i];

                if (pthread_create(&threads[i], NULL, target_task_run, &target_task_args[i]) != 0) {
                    fprintf(stderr, "[main] Error al crear thread para target %s\n",
                            targets[i].record_ip_id);
                }
            }

            // Se espera a que todos los hilos terminen su tarea
            // Para asegurar sincronización
            for (int i = 0; i < count; i++) {
                pthread_join(threads[i], NULL);
            }
        }

        // Esperar segun lo configurado para la siguiente ronda de monitoreo
        sleep(config.poll_interval_sec);
    }

    return 0;
}
