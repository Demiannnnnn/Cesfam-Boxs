# Contrato de la API (referencia compacta)

Fuente legible: `docs/04-api.md`. Tipos Rust: `shared/src/`.

## Tipos

```text
BoxAtencion   { id: i64, nombre: String, activo: bool, orden: i64 }
Reserva       { id: i64, box_id: i64, fecha: Fecha, bloque: Bloque,
                funcionario: String, motivo: Option<String>, creada_en: String }
NuevoBox      { nombre: String }
CambiosBox    { nombre: Option<String>, activo: Option<bool> }
NuevaReserva  { box_id: i64, fecha: Fecha, bloque: Bloque,
                funcionario: String, motivo: Option<String> }
EstadoSalud   { estado: "ok", version: String, equipo: String }
ErrorRespuesta{ codigo: CodigoError, mensaje: String }
RespuestaDescubrimiento { servicio: "gestionbox", version, puerto_http: u16, equipo }

Fecha  = "AAAA-MM-DD"  (serde como string, newtype sobre chrono::NaiveDate)
Bloque = u8 en 0..=23  (serde como número, rechaza fuera de rango al deserializar)
CodigoError = datos_invalidos | no_encontrado | bloque_ocupado | nombre_duplicado
            | box_inactivo | error_interno      (serde snake_case)
```

## Endpoints

| Método | Ruta | Entrada | 2xx | Errores |
|---|---|---|---|---|
| GET | `/health` | — | 200 `EstadoSalud` | — |
| GET | `/boxes` | `?incluir_inactivos=bool` | 200 `[BoxAtencion]` | — |
| POST | `/boxes` | `NuevoBox` | 201 `BoxAtencion` | 400, 409 nombre_duplicado |
| PATCH | `/boxes/{id}` | `CambiosBox` | 200 `BoxAtencion` | 400, 404, 409 nombre_duplicado |
| GET | `/reservas` | `?fecha=` ó `?desde=&hasta=` | 200 `[Reserva]` orden (fecha, bloque, box_id) | 400 |
| POST | `/reservas` | `NuevaReserva` | 201 `Reserva` | 400, 404, 409 bloque_ocupado, 422 box_inactivo |
| DELETE | `/reservas/{id}` | — | 204 | 404 |

## Validaciones

- `nombre` box: trim, 1..=40 chars, único sin distinguir mayúsculas.
- `funcionario`: trim, 1..=80 chars. `motivo`: trim, ≤200, vacío → null.
- Rango `desde..=hasta`: `desde <= hasta`, ≤ 366 días.
- Box nuevo: `orden = max(orden)+1`.

## Mensajes fijos

| codigo | mensaje |
|---|---|
| bloque_ocupado | Ese bloque acaba de ser tomado |
| nombre_duplicado | Ya existe un box con ese nombre |
| box_inactivo | Ese box está desactivado |
| no_encontrado | No se encontró {el box / la reserva} |
| datos_invalidos | (detalle específico en español) |
| error_interno | Ocurrió un error interno en el servidor |

## UDP

Petición: bytes exactos `GESTIONBOX_BUSCAR` → `255.255.255.255:8001`.
Respuesta (unicast al origen): JSON `RespuestaDescubrimiento`. Otro contenido: ignorar.
