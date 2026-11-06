# Contrato de la API (referencia compacta)

Fuente legible: `docs/04-api.md`. Tipos Rust: `shared/src/`.

## Tipos

```text
Etiqueta           { id: i64, nombre: String, color: Color, activo: bool }
NuevaEtiqueta      { nombre: String, color: Color }
CambiosEtiqueta    { nombre?: String, color?: Color, activo?: bool }
Funcionario        { id: i64, nombre: String, etiqueta_id: Option<i64>, activo: bool }
NuevoFuncionario   { nombre: String, etiqueta_id: Option<i64> }
CambiosFuncionario { nombre?: String, etiqueta_id?: Option<Option<i64>>, activo?: bool }
BoxAtencion        { id: i64, nombre: String, activo: bool, orden: i64, etiqueta_id: Option<i64> }
NuevoBox           { nombre: String, etiqueta_id: Option<i64> }
CambiosBox         { nombre?: String, activo?: bool, etiqueta_id?: Option<Option<i64>> }
Reserva            { id: i64, box_id: i64, fecha: Fecha, bloque: Bloque,
                     funcionario_id: Option<i64>, funcionario: String,
                     motivo: Option<String>, creada_en: String, grupo: Option<i64> }
NuevaReserva       { box_id: i64, fecha: Fecha, bloque: Bloque,
                     funcionario_id: i64, motivo: Option<String> }
NuevoLote          { box_id: i64, fechas: [Fecha] (1..=46), desde: Bloque, hasta: Bloque,
                     funcionario_id: i64, motivo: Option<String>, omitir_ocupados: bool = false }
ResultadoLote      { creadas: [Reserva], omitidos: [{ fecha: Fecha, bloque: Bloque }] }
ResultadoCancelacion { canceladas: usize }
ConsultaBuscarReservas { desde: Fecha, hasta: Fecha, funcionario_id?: i64, etiqueta_id?: i64, box_id?: i64 }
ConsultaBoxesLibres { fecha: Fecha, desde: Bloque, hasta: Bloque, etiqueta_id?: i64 }
EstadoSalud        { estado: "ok", version: String, equipo: String }
ErrorRespuesta     { codigo: CodigoError, mensaje: String }
RespuestaDescubrimiento { servicio: "gestionbox", version, puerto_http: u16, equipo }

Fecha  = "AAAA-MM-DD"  (serde como string, newtype sobre chrono::NaiveDate)
Bloque = u8 en 0..=23  (serde como número, rechaza fuera de rango al deserializar)
Color  = "#rrggbb"     (serde como string, acepta hex mayús/minús y normaliza a minúsculas)
CodigoError = datos_invalidos | no_encontrado | bloque_ocupado | nombre_duplicado
            | box_inactivo | funcionario_inactivo | etiqueta_inactiva | error_interno  (snake_case)
```

En `CambiosBox` y `CambiosFuncionario`, `etiqueta_id` es doble `Option`: omitido (`None`) no
cambia la etiqueta; `"etiqueta_id": null` (`Some(None)`) la quita; `"etiqueta_id": 3` (`Some(Some(3))`) la asigna.

## Endpoints

| Método | Ruta | Entrada | 2xx | Errores |
|---|---|---|---|---|
| GET | `/health` | — | 200 `EstadoSalud` | — |
| GET | `/boxes` | `?incluir_inactivos=bool` | 200 `[BoxAtencion]` | — |
| GET | `/boxes/libres` | `?fecha=&desde=&hasta=&etiqueta_id=` | 200 `[BoxAtencion]` activos orden grilla | 400 |
| POST | `/boxes` | `NuevoBox` | 201 `BoxAtencion` | 400, 404, 409 nombre_duplicado, 422 etiqueta_inactiva |
| PATCH | `/boxes/{id}` | `CambiosBox` | 200 `BoxAtencion` | 400, 404, 409 nombre_duplicado, 422 etiqueta_inactiva |
| GET | `/etiquetas` | `?incluir_inactivos=bool` | 200 `[Etiqueta]` orden por nombre | — |
| POST | `/etiquetas` | `NuevaEtiqueta` | 201 `Etiqueta` | 400, 409 nombre_duplicado |
| PATCH | `/etiquetas/{id}` | `CambiosEtiqueta` | 200 `Etiqueta` | 400, 404, 409 nombre_duplicado |
| GET | `/funcionarios` | `?incluir_inactivos=bool` | 200 `[Funcionario]` orden por nombre | — |
| POST | `/funcionarios` | `NuevoFuncionario` | 201 `Funcionario` | 400, 404, 409 nombre_duplicado, 422 etiqueta_inactiva |
| PATCH | `/funcionarios/{id}` | `CambiosFuncionario` | 200 `Funcionario` | 400, 404, 409 nombre_duplicado, 422 etiqueta_inactiva |
| GET | `/reservas` | `?fecha=` ó `?desde=&hasta=` | 200 `[Reserva]` orden (fecha, bloque, box_id) | 400 |
| GET | `/reservas/buscar` | `?desde=&hasta=&funcionario_id=&etiqueta_id=&box_id=` | 200 `[Reserva]` orden (fecha, box_id, bloque), máx. 2000 | 400 |
| POST | `/reservas` | `NuevaReserva` | 201 `Reserva` | 400, 404, 409 bloque_ocupado, 422 box_inactivo \| funcionario_inactivo |
| POST | `/reservas/lote` | `NuevoLote` | 201 `ResultadoLote` | 400, 404, 409 bloque_ocupado, 422 box_inactivo \| funcionario_inactivo |
| DELETE | `/reservas/{id}` | — | 204 | 404 |
| DELETE | `/reservas/grupo/{grupo}` | `?desde=` | 200 `ResultadoCancelacion` | 400, 404 |
| GET | `/respaldo` | — | 200 SQLite binario (`application/octet-stream`) | 500 |

## Validaciones

- `nombre` box y etiqueta: trim, 1..=40 chars, único sin distinguir mayúsculas.
- `nombre` funcionario: trim, 1..=80 chars, único sin distinguir mayúsculas.
- `motivo`: trim, ≤200, vacío → null.
- `etiqueta_id` al asignarla a box o funcionario: debe existir (404) y estar activa (422).
- `funcionario_id` al reservar: debe existir (404) y estar activo (422).
- Al leer reservas, `funcionario` proviene de `LEFT JOIN funcionario` (`COALESCE(f.nombre, r.funcionario)`), reflejando renombres.
- Rango `desde..=hasta`: `desde <= hasta`, ≤ 366 días.
- `/reservas/buscar`: exige `desde..=hasta` y al menos uno de `funcionario_id`, `etiqueta_id` o `box_id` (400 `"Indica al menos un funcionario, etiqueta o box"`); `etiqueta_id` filtra por etiqueta del funcionario o del box; máx. 2000 filas.
- `/boxes/libres`: `desde <= hasta` (`Bloque`); devuelve boxes activos sin reservas en `desde..=hasta` de `fecha` (`NOT EXISTS`), filtrados por `etiqueta_id` del box si se indica.
- Box nuevo: `orden = max(orden)+1`.
- Lote: `desde <= hasta`; fechas sin repetir (se ordenan). Una transacción: sin
  `omitir_ocupados` es todo o nada (409); con él crea los libres y lista los ocupados
  (409 si no quedó ninguno). `grupo` = id de la primera reserva creada.
- Respaldo: `GET /respaldo` entrega una copia consistente de la base SQLite para la copia diaria de los equipos de escritura (RF-18, ADR-021) sin autenticación (ADR-020). Cabecera `Content-Disposition: attachment; filename="gestion-box-AAAAMMDD-HHMMSS.db"`. El temporal en el servidor se borra antes de enviar la respuesta.

## Mensajes fijos

| codigo | mensaje |
|---|---|
| bloque_ocupado | Ese bloque acaba de ser tomado |
| nombre_duplicado | Ya existe {un box / un funcionario / una etiqueta} con ese nombre |
| box_inactivo | Ese box está desactivado |
| funcionario_inactivo | Ese funcionario está desactivado |
| etiqueta_inactiva | Esa etiqueta está desactivada |
| no_encontrado | No se encontró {el box / la etiqueta / el funcionario / la reserva} |
| datos_invalidos | (detalle específico en español) |
| error_interno | Ocurrió un error interno en el servidor |

## UDP

Petición: bytes exactos `GESTIONBOX_BUSCAR` → `255.255.255.255:8001`.
Respuesta (unicast al origen): JSON `RespuestaDescubrimiento`. Otro contenido: ignorar.
