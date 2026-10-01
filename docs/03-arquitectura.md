# 03 — Arquitectura

## Vista general: 1 servidor, N clientes

La API corre como servicio independiente; la app Tauri es igual en todos los PCs y en el
servidor apunta a `localhost`.

```
┌──────────────────────── PC servidor (Windows) ────────────────────────┐
│                                                                       │
│  Servicio "GestionBoxAPI" (NSSM)                                      │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ gestion-box-api.exe                                             │  │
│  │                                                                 │  │
│  │  ┌─────────────┐   ┌────────────┐   ┌──────────────────────┐    │  │
│  │  │ HTTP :8000  │──►│ servicio   │──►│ repositorio (SQL)    │    │  │
│  │  │ (Axum)      │   │ (reglas)   │   │ rusqlite + UNIQUE    │    │  │
│  │  └─────────────┘   └────────────┘   └──────────┬───────────┘    │  │
│  │  ┌─────────────┐                               ▼                │  │
│  │  │ UDP :8001   │   C:\ProgramData\GestionBox\                   │  │
│  │  │ descubrim.  │     ├── gestion-box.db                         │  │
│  │  └─────────────┘     ├── backups\gestion-box-AAAAMMDD-HHMMSS.db │  │
│  │                      └── api.log                                │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  App Tauri (misma que en clientes, apunta a localhost:8000)           │
└───────────────────────────────────────────────────────────────────────┘
            ▲  HTTP (JSON)                ▲  UDP broadcast 255.255.255.255:8001
            │                             │
┌───────────┴─────────── PC cliente (×N) ─┴─────────────────────────────┐
│  App Tauri                                                            │
│  ┌────────────────────────────┐  invoke  ┌──────────────────────────┐ │
│  │ Interfaz (HTML/CSS/TS)     │ ───────► │ Backend Rust de Tauri    │ │
│  │ grilla, modales, textos    │ ◄─────── │ cliente HTTP, UDP,       │ │
│  └────────────────────────────┘  JSON    │ config local, export CSV │ │
│                                          └──────────────────────────┘ │
│  %APPDATA%\cl.cesfam.gestionbox\cliente.json  (última dirección)     │
└───────────────────────────────────────────────────────────────────────┘
```

## Componentes

### `shared/` — tipos del dominio

Crate de librería sin dependencias de red ni de base de datos. Contiene:

- `BoxAtencion`, `Reserva` y los DTOs de entrada (`NuevaReserva`, `NuevoBox`, `CambiosBox`).
- Newtypes validados: `Bloque` (0–23), `Fecha` (AAAA-MM-DD).
- Constantes de horario: inicio 08:00, duración 30 min, 24 bloques.
- `ErrorRespuesta { codigo, mensaje }`, el cuerpo de todo error HTTP.
- Constantes del protocolo de descubrimiento UDP.

API y App dependen de este crate, así el contrato no puede desincronizarse sin que falle la compilación.

### `api/` — servidor

Capas, de afuera hacia adentro:

| Capa | Módulo | Responsabilidad |
|---|---|---|
| Arranque | `main.rs` | Leer configuración, iniciar log, respaldar, migrar, levantar HTTP y UDP |
| Configuración | `config.rs` | Resolver carpeta de datos, puertos y rutas absolutas (RNF-04) |
| HTTP | `http/` | Router, handlers, CORS, conversión de errores a respuestas |
| Servicio | `servicio.rs` | Reglas: box activo, bloque válido, nombres no vacíos |
| Repositorio | `repositorio/` | SQL parametrizado; traduce violación de `UNIQUE` a error de dominio |
| Base de datos | `db.rs` | Apertura, `PRAGMA`s, esquema y migraciones |
| Descubrimiento | `descubrimiento.rs` | Listener UDP en hilo aparte |
| Respaldo | `respaldo.rs` | Copia al arrancar y rotación a 30 |
| Log | `log.rs` | `tracing` hacia `api.log` |

Acceso concurrente a SQLite: una sola `rusqlite::Connection` protegida por `Mutex` dentro de
`AppState`, usada desde `tokio::task::spawn_blocking`. Con 3 usuarios no hace falta pool.

### `app/` — cliente Tauri 2

- `app/src-tauri/` (Rust): comandos `invoke`, cliente HTTP (`reqwest`), descubrimiento UDP,
  configuración local, exportación CSV, traducción de errores de red a mensajes en español.
- `app/src/` (TypeScript sin framework, empaquetado con Vite): grilla, modales, estado de
  conexión y refresco periódico.

## Decisiones

| Decisión | Alternativa descartada | Motivo |
|---|---|---|
| API y app separadas | App que incluye el servidor | Si alguien cierra la app en el PC servidor, los demás siguen funcionando |
| Sin autenticación ni roles | Login por funcionario | Datos no sensibles, solo red local; menos fallas y soporte |
| Bloques fijos de 30 min + `UNIQUE` | Horarios libres con detección de solapes | La base de datos garantiza atómicamente que no haya doble reserva |
| Refresco periódico | WebSockets | Con 3 usuarios es suficiente y mucho más simple |
| Conexión en capas | Solo IP fija | La IP del servidor puede cambiar (DHCP); el nombre o el broadcast la encuentran |
| Red en el lado Rust de Tauri | `fetch` desde la webview | UDP solo es posible en Rust; reutiliza tipos de `shared`; evita problemas de CORS/CSP |
| SQLite embebido | PostgreSQL / servidor externo | Un ejecutable, cero instalación, respaldo = copiar un archivo |
| NSSM | Servicio Windows nativo en Rust | Probado, reinicia ante caídas, sin código extra |

Más detalle y registro de decisiones nuevas en [`.ai-context/decisiones.md`](../.ai-context/decisiones.md).

## Flujo: crear una reserva

```
Funcionario   Interfaz TS        Backend Tauri          API                  SQLite
    │ clic celda │                    │                   │                     │
    │──────────►│ modal reserva       │                   │                     │
    │ nombre ok  │ invoke(crear_reserva)                  │                     │
    │──────────►│───────────────────►│ POST /reservas    │                     │
    │            │                    │──────────────────►│ valida (servicio)   │
    │            │                    │                   │ INSERT ... (?1..?5) │
    │            │                    │                   │────────────────────►│
    │            │                    │                   │   ok │ UNIQUE falla │
    │            │                    │  201 Reserva      │◄─────┘             │
    │            │                    │  ó 409 bloque_ocupado                   │
    │            │◄───────────────────│ Ok / Err(mensaje) │                     │
    │◄───────────│ refresca grilla / muestra "Ese bloque acaba de ser tomado"   │
```

## Flujo: descubrir el servidor (RF-09, RNF-08)

```
inicio app
   │
   ▼
¿hay dirección guardada? ──no──┐
   │ sí                        │
   ▼                           │
GET http://<guardada>/health   │
(timeout 1,5 s)                │
   │ ok → usar y guardar       │
   │ falla                     │
   ▼                           ▼
broadcast UDP "GESTIONBOX_BUSCAR" a 255.255.255.255:8001
(espera 1,5 s, hasta 2 intentos)
   │ responde → GET /health en la IP que respondió → usar y guardar
   │ nadie responde
   ▼
pantalla de ingreso manual ("nombre del PC o IP")
```

Tiempo máximo antes de pedir ingreso manual: ~4,5 s (< 5 s exigidos por RNF-08).
