# 03 — Arquitectura

## Vista general: 1 servidor, N clientes

La API corre como servicio independiente de Windows en un único computador servidor; la app Tauri
se instala en todos los equipos y define su comportamiento según el **modo de equipo** configurado
en su primer inicio (`servidor`, `escritura` o `lectura`, [ADR-020](../.ai-context/decisiones.md#adr-020--2026-11-06--aceptada--tres-modos-de-equipo-elegidos-al-instalar)):

- **Servidor:** Instala y levanta el servicio local de la API (`GestionBoxAPI`), alojando la base de datos SQLite y respondiendo peticiones HTTP (8000) y de descubrimiento UDP (8001). Su app apunta a `localhost:8000` y permite reservar sin descargar copia adicional.
- **Escritura:** Equipos de trabajo para reservar y cancelar horas. Descargan periódicamente una copia consistente diaria de la base (`GET /respaldo`) conservando las últimas 7 en su propio disco ([ADR-021](../.ai-context/decisiones.md#adr-021--2026-11-06--aceptada--copia-diaria-de-la-base-en-los-equipos-de-escritura)). En caso de muerte definitiva del servidor, uno de estos equipos puede promoverse a nuevo servidor ([ADR-022](../.ai-context/decisiones.md#adr-022--2026-11-06--aceptada--promoción-a-servidor-oculta-solo-si-el-servidor-murió)).
- **Lectura:** Puestos de solo consulta para visualizar la grilla y buscar, con los controles de creación y edición ocultos.

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
│  App Tauri (modo servidor: apunta a localhost:8000)                   │
└───────────────────────────────────────────────────────────────────────┘
            ▲  HTTP (JSON / respaldo)     ▲  UDP broadcast 255.255.255.255:8001
            │                             │
┌───────────┴─────────── PC cliente (×N) ─┴─────────────────────────────┐
│  App Tauri (modo escritura o lectura)                                 │
│  ┌────────────────────────────┐  invoke  ┌──────────────────────────┐ │
│  │ Interfaz (HTML/CSS/TS)     │ ───────► │ Backend Rust de Tauri    │ │
│  │ grilla, modales, asistente │ ◄─────── │ cliente HTTP, UDP,       │ │
│  │ solo consulta o escritura  │  JSON    │ copia diaria, config     │ │
│  └────────────────────────────┘          └─────────────┬────────────┘ │
│                                                        ▼              │
│  C:\ProgramData\GestionBox\copias-del-servidor\ (7 copias, escritura) │
│  %APPDATA%\cl.cesfam.gestionbox\cliente.json  (modo y dirección)     │
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
| Respaldo | `respaldo.rs` | Copia al arrancar y rotación a 30 (RF-11); entrega consistente `GET /respaldo` (RF-18) |
| Log | `log.rs` | `tracing` hacia `api.log` |

Acceso concurrente a SQLite: una sola `rusqlite::Connection` protegida por `Mutex` dentro de
`AppState`, usada desde `tokio::task::spawn_blocking`. Con 3 usuarios no hace falta pool.

### `app/` — cliente Tauri 2

- `app/src-tauri/` (Rust):
  - Comandos `invoke`, cliente HTTP (`reqwest`), descubrimiento UDP, configuración local (`cliente.json`).
  - `servidor_local.rs`: instalación del servicio Windows con elevación UAC al elegir modo servidor (RF-16, RNF-12, RNF-14).
  - `copia_diaria/`: tarea en segundo plano que descarga una copia diaria de la base (`GET /respaldo`) en equipos de escritura y rota dejando 7 (RF-18, ADR-021).
  - `avanzado.rs`: cambio entre escritura y lectura y promoción de emergencia a nuevo servidor restaurando la copia local más reciente (RF-19, ADR-022).
  - Exportación CSV y traducción de errores de red a mensajes en español.
- `app/src/` (TypeScript sin framework, empaquetado con Vite):
  - `asistente.ts`: pantalla de bienvenida al primer inicio para elegir el modo de uso del equipo (RF-16, RF-17, ADR-020).
  - `avanzado.ts`: ventana de opciones avanzadas protegida por clave (RF-19, ADR-022).
  - Grilla, vista semanal, búsqueda, modales de reserva, funcionarios y etiquetas, estado de conexión y refresco periódico.

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
| Tres modos de equipo al instalar ([ADR-020](../.ai-context/decisiones.md#adr-020--2026-11-06--aceptada--tres-modos-de-equipo-elegidos-al-instalar)) | Perfiles con login | Distingue servidor, escritura y lectura en la app sin requerir autenticación en la API |
| Copia diaria en equipos de escritura ([ADR-021](../.ai-context/decisiones.md#adr-021--2026-11-06--aceptada--copia-diaria-de-la-base-en-los-equipos-de-escritura)) | Respaldo manual en red | Conserva 7 copias en discos distintos al servidor sin configuración ni intervención humana |
| Promoción a servidor oculta ([ADR-022](../.ai-context/decisiones.md#adr-022--2026-11-06--aceptada--promoción-a-servidor-oculta-solo-si-el-servidor-murió)) | Failover automático complejo | Procedimiento de emergencia si el servidor muere, verificando antes la ausencia de servidor vivo por UDP |

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
