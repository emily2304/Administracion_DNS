#ifndef HTTP_CHECK_H
#define HTTP_CHECK_H
 
#include "db_client.h"

// Ejecuta un intento de health check HTTP.
/*
0: unhealthy
1: healthy
*/
int check_http_run(const HealthTarget *target, double *rtt_ms);
 
#endif