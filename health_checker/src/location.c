#include <math.h>
#include <string.h>
#include "location.h"

#define EARTH_RADIUS_KM 6371.0

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

// Se convierte de grados a radianes
static inline double deg_to_rad(double deg) {
    return deg * (M_PI / 180.0);
}

double get_distance_haversine(double lat1, double lon1, double lat2, double lon2){
    // convertir a rad
    double dlat = deg_to_rad(lat2 - lat1);
    double dlon = deg_to_rad(lon2 - lon1);
    
    // Formula de Haversine
    double a = sin(dlat / 2) * sin(dlat / 2) +
               cos(deg_to_rad(lat1)) * cos(deg_to_rad(lat2)) *
               sin(dlon / 2) * sin(dlon / 2);
    
    // Calculo de la distancia angular
    double c = 2 * atan2(sqrt(a), sqrt(1 - a));
    
    // convertir a km
    return EARTH_RADIUS_KM * c;
}

