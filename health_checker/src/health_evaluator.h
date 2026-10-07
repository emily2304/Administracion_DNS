#ifndef HEALTH_RUNNER_H
#define HEALTH_RUNNER_H
#include "db_client.h"

// Ejecuta el health check (TCP o HTTP) repitiendo hasta target->retries intentos
// Determina el estado final por mayoría simple
/*
avg_rtt_ms se llena con el promedio de RTT de los intentos exitosos.
(0.0 si ninguno tuvo éxito), para uso en registros tipo "round-trip".
0: unhealthy
1: healthy
*/
int health_evaluator_run(const HealthTarget *target, double *avg_rtt_ms);
 
#endif