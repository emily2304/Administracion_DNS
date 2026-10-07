#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <time.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "tcp_check.h"

// Devuelve el tiempo actual en milisegundos 
// Es monotónico, para medir RTT
static double get_time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1000.0 + (double)ts.tv_nsec / 1e6;
}

// Intento de conexión con TCP.
// Sin retries. Los retries se manejan en otro archivo.
static int tcp_attempt(const char *ip, int port, int timeout_sec, double *rtt_ms) { // en el puntero rtt_ms se almacena la latencia calculada
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        return 0; // Error
    }

    // Para controlar los timeouts --> no bloqueante
    int flags = fcntl(sock, F_GETFL, 0);
    fcntl(sock, F_SETFL, flags | O_NONBLOCK);
    
    // Config. de la dirección destino (IP y puerto)
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((uint16_t)port); // pasa a little-endian
    
    // inet_pton para convertir la IP a formato binario
    if (inet_pton(AF_INET, ip, &addr.sin_addr) <= 0) {
        close(sock);
        return 0; // En caso de una IP inválida
    }

    double start = get_time_ms();
    
    // Aseguramos que los dispositivos esten listos para comunicarse, como un TCP handshake
    int result = connect(sock, (struct sockaddr *)&addr, sizeof(addr));

    // Como es un socket no bloqueant, connect() devuelve -1 con errno = EINPROGRESS
    if (result < 0 && errno != EINPROGRESS) {
        close(sock);
        return 0;
    }
    
    // En caso de que la conexión quede en progreso (EINPROGRESS),
    // lo que se hace es esperar mediante select()
    if (result < 0) {
        fd_set write_fds;
        FD_ZERO(&write_fds);
        FD_SET(sock, &write_fds); // Monitoreo por si el socket es writable
 
        struct timeval tv;
        tv.tv_sec = timeout_sec;
        tv.tv_usec = 0;
    
        // Esperar hasta que se complete la conexión o expire el timeout
        int sel = select(sock + 1, NULL, &write_fds, NULL, &tv);
        if (sel <= 0) { // sel == 0 -> timeout, sel < 0 -> error en la llamada select()
            close(sock);
            return 0;
        }

        // se verifica si la conexión realmente se completó sin error
        int so_error = 0;
        socklen_t len = sizeof(so_error);
        getsockopt(sock, SOL_SOCKET, SO_ERROR, &so_error, &len);
        if (so_error != 0) {
            close(sock);
            return 0;
        }
    }

    // Calculo del rtt y se cierra el socket
    *rtt_ms = get_time_ms() - start; 
    close(sock);
    return 1; // conexión exitosa
}

// ejecutar la verificación del estado TCP sobre un objetivo
/*
0: unhealthy
1: healthy
*/
int check_tcp_run(const HealthTarget *target, double *rtt_ms) {
    *rtt_ms = 0.0;
    return tcp_attempt(target->ip, target->port, target->timeout_sec, rtt_ms); }