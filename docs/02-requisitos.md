# 02 — Requisitos

La app cubre ver, reservar y cancelar en una sola pantalla; la API es la única fuente de datos.

## Requisitos funcionales (RF)

| ID | Requisito | Componente | Tareas |
|---|---|---|---|
| RF-01 | Mostrar una grilla del día con boxes activos en columnas y bloques horarios en filas | App | T-0.1, T-0.2, T-3.3 |
| RF-02 | Navegar entre días con los botones anterior, hoy y siguiente, y con un selector de fecha | App | T-3.3 |
| RF-03 | Crear una reserva al hacer clic en una celda libre, con nombre del funcionario (obligatorio) y motivo (opcional) | App + API | T-2.4, T-3.4 |
| RF-04 | Ver el detalle de una celda ocupada y cancelar la reserva | App + API | T-2.4, T-3.4 |
| RF-05 | Rechazar una reserva si el box ya está ocupado en ese bloque, y avisar "Ese bloque acaba de ser tomado" | API (UNIQUE) + App | T-2.1, T-2.5, T-3.5 |
| RF-06 | Agregar, renombrar y desactivar boxes desde un modal de configuración | App + API | T-0.2, T-2.3, T-3.6 |
| RF-07 | Refrescar la grilla automáticamente cada 15 a 30 segundos | App | T-3.7 |
| RF-08 | Mostrar el estado de conexión con el servidor (conectado o sin conexión) | App (`/health`) | T-2.6, T-3.8 |
| RF-09 | Encontrar el servidor automáticamente: dirección guardada, luego broadcast UDP, luego ingreso manual | App + API | T-0.4, T-0.5, T-2.8, T-3.1, T-3.2 |
| RF-10 | Guardar la última dirección válida del servidor en la configuración local | App | T-3.1 |
| RF-11 | Respaldar la base de datos al iniciar la API, conservando los últimos 30 respaldos | API | T-2.9 |
| RF-12 | Exportar las reservas de un rango de fechas a Excel o CSV (opcional) | App | T-3.10 |

**Bloques horarios:** 30 minutos entre 08:00 y 20:00 → 24 bloques. El bloque `0` es
08:00–08:30 y el bloque `23` es 19:30–20:00. *Pendiente de confirmar con el CESFAM (T-0.1).*

## Requisitos no funcionales (RNF)

El criterio central es que el sistema funcione sin nadie que lo mantenga después de la entrega.

| ID | Requisito | Tareas |
|---|---|---|
| RNF-01 | La API arranca sola con Windows como servicio (NSSM), sin sesión iniciada, y se reinicia sola si se cae | T-4.2 |
| RNF-02 | La API corre sin ventana de consola ni intervención del usuario | T-4.1 |
| RNF-03 | La API escucha en `0.0.0.0:8000` (HTTP) y `0.0.0.0:8001` (UDP de descubrimiento) | T-2.8, T-4.2 |
| RNF-04 | Datos, respaldos y log usan rutas absolutas en `C:\ProgramData\GestionBox\` | T-2.2 |
| RNF-05 | Un solo ejecutable por componente, con SQLite embebido (`rusqlite` bundled) y sin runtimes externos | T-1.1 |
| RNF-06 | Todas las consultas SQL usan parámetros, nunca texto concatenado | T-2.1 – T-2.5 |
| RNF-07 | La API permite CORS desde la app Tauri | T-2.7 |
| RNF-08 | La app encuentra el servidor aunque cambie su IP, en menos de 5 segundos | T-0.3, T-0.6, T-3.1 |
| RNF-09 | Todo error de red muestra un mensaje en español que indica qué revisar (p. ej. "Verifica que PC-SOME esté encendido") | T-3.9 |
| RNF-10 | La API registra eventos y errores en `C:\ProgramData\GestionBox\api.log` | T-2.10 |
| RNF-11 | La interfaz se usa sin capacitación: textos en español, una pantalla, acciones de un clic | T-3.3 – T-3.6 |
| RNF-12 | La instalación del servidor es un solo `instalar.bat` ejecutado como administrador | T-4.2 |
| RNF-13 | Compatible con Windows 10 y 11 | T-0.7, T-4.4 |

## Trazabilidad

Cada tarea en `tasks/` declara los requisitos que cubre y cada prueba de aceptación en
[08-pruebas-aceptacion.md](08-pruebas-aceptacion.md) indica qué requisito verifica. Si
agregas una tarea, agrega su ID a esta tabla.
