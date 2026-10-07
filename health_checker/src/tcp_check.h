#ifndef TCP_CHECK_H
#define TCP_CHECK_H

#include "db_client.h"

// Ejecuta un intento de health check TCP.
/*
0: unhealthy
1: healthy
*/
int check_tcp_run(const HealthTarget *target, double *rtt_ms); // RTT = Round Trip Time

#endif