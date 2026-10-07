#ifndef HTTP_CLIENT_H
#define HTTP_CLIENT_H
 
// Buffer para acumular el cuerpo de una respuesta HTTP.
typedef struct {
    char *data;
    size_t size;
} HttpResponse;

// Función para liberar memoria dinámica
void http_response_free(HttpResponse *response);

/*
- Configura los headers
- Hace la petición GET
- Llena la estructura HttpResponse con el cuerpo JSON
 */
// Se va a usar para db_fetch_targets
long http_get_supabase(const char *url, const char *api_key, HttpResponse *response);

/*
- Hace un PATCH a "url" con el cuerpo ya serializado
- Retorna el código de estado HTTP, o -1 en caso de error
 */
// Se va a usar para db_write_result
long http_patch_supabase(const char *url, const char *api_key, const char *json_body);

/*
- Hace un POST a "url" con el cuerpo ya serializado
- Retorna el código de estado HTTP, o -1 en caso de error
 */
// Se va a usar para db_write_result
long http_upsert_supabase(const char *url, const char *api_key, const char *json_body);

#endif