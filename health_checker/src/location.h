#ifndef LOCATION_H
#define LOCATION_H
#include "config.h"

// Struct con la ubicación de un Health Checker conocido
typedef struct {
    char checker_id[64];
    double lat; // latitud
    double longitude;
} CheckerLocation;

// Se usa la formula de Haversine para estimar cual que Health Checker está mas cerca de un punto dado.
// La distancia se calcula en KM
double get_distance_haversine(double lat1, double lon1, double lat2, double lon2);

// Dado la latitud, longitud y la lista de los health checkers, se calcula el índice del checker más cercano
// -1 si la lista está vacía
int location_find_nearest(double lat, double lon, const CheckerLocation *checkers, int checkers_count);

#endif 