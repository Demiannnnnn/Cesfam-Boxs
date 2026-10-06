# Cesfam-Boxs — Gestión de Boxes CESFAM

Aplicación de escritorio local (Rust + Tauri) y API básica (Rust + Axum + SQLite) para la
gestión de boxes de atención del CESFAM de Curicó. Reemplaza la planilla Excel de reservas
por una grilla simple donde cualquier funcionario reserva un box en un bloque horario desde
cualquier PC de la red local.

Diseñada para **despliegue automatizado** y **funcionamiento duradero sin mantenimiento**:
después de firmar el acta de entrega nadie va a mantener el sistema, así que todo debe
arrancar solo, recuperarse solo y explicar en español qué revisar cuando algo falla.

---

## Índice

- [Qué hace y qué no hace](#qué-hace-y-qué-no-hace)
- [Arquitectura en 30 segundos](#arquitectura-en-30-segundos)
- [Estructura del repositorio](#estructura-del-repositorio)
- [Requisitos de desarrollo](#requisitos-de-desarrollo)
- [Compilar y ejecutar](#compilar-y-ejecutar)
- [Pruebas y calidad](#pruebas-y-calidad)
- [Flujo de ramas](#flujo-de-ramas)
- [Hoja de ruta (fases)](#hoja-de-ruta-fases)
- [Documentación](#documentación)

---

## Qué hace y qué no hace

**Incluye**

- Una sola vista: grilla de boxes (columnas) por bloque horario (filas) para un día.
- Crear y cancelar reservas (nombre del funcionario obligatorio, motivo opcional).
- Administrar la lista de boxes (agregar, renombrar, desactivar).
- API local en un PC designado como servidor, que arranca sola como servicio de Windows.
- Descubrimiento automático del servidor en la red (dirección guardada → localhost → broadcast UDP → ingreso manual).
- Respaldo automático de la base de datos en cada arranque (últimas 30 copias).

**No incluye**

- Autenticación ni roles.
- Datos de pacientes (solo box, fecha, bloque, nombre del funcionario y motivo).
- Acceso remoto fuera del establecimiento.
- Notificaciones.
- Mantenimiento posterior a la firma del acta.

**Supuestos**

- Hasta 3 usuarios simultáneos; base de datos bajo 100 MB.
- Todos los PCs están en la misma red local y subred.
- Hay permisos de administrador para instalar.
- Existe un PC con Windows 10/11 designado como servidor, que puede apagarse o reiniciarse
  en cualquier momento; el sistema vuelve solo al encenderlo.

Bloques horarios: **30 minutos entre 08:00 y 20:00 (24 bloques, índices 0–23)**.
Pendiente de confirmar con el CESFAM (tarea `T-0.1`).

## Arquitectura en 30 segundos

```
          Red local del CESFAM (misma subred)
 ┌───────────────────────────────────────────────────────────┐
 │  PC servidor (Windows)                                    │
 │  ┌─────────────────────────────┐   ┌───────────────────┐  │
 │  │ gestion-box-api.exe (NSSM)  │   │ App Tauri         │  │
 │  │  HTTP  0.0.0.0:8000         │◄──┤ (apunta a         │  │
 │  │  UDP   0.0.0.0:8001         │   │  localhost)       │  │
 │  │  SQLite C:\ProgramData\...  │   └───────────────────┘  │
 │  └──────────▲──────────▲───────┘                          │
 │             │ HTTP     │ UDP broadcast (descubrimiento)   │
 │  ┌──────────┴───┐  ┌───┴──────────┐                       │
 │  │ App Tauri    │  │ App Tauri    │   … hasta N clientes  │
 │  │ PC cliente 1 │  │ PC cliente 2 │                       │
 │  └──────────────┘  └──────────────┘                       │
 └───────────────────────────────────────────────────────────┘
```

- **API** (`api/`): Axum + rusqlite (SQLite embebido). Única fuente de datos. Corre como
  servicio de Windows con NSSM, sin ventana, y se reinicia sola si se cae.
- **App** (`app/`): Tauri 2. Idéntica en todos los PCs. Toda la red (HTTP y UDP) se hace
  desde el lado Rust de Tauri; la interfaz (HTML/CSS/TypeScript sin framework) solo invoca
  comandos.
- **Shared** (`shared/`): crate con los tipos del dominio (`Box`, `Reserva`, DTOs) que
  comparten API y App, para que el contrato no se desincronice.

Detalle completo en [`docs/03-arquitectura.md`](docs/03-arquitectura.md).

## Estructura del repositorio

```
Cesfam-Boxs/
├── README.md              ← este archivo
├── AGENTS.md              ← reglas para agentes de IA y colaboradores
├── Cargo.toml             ← workspace de Cargo (api, shared, app/src-tauri)
├── api/                   ← API Axum + SQLite (gestion-box-api.exe)
├── app/                   ← App de escritorio Tauri (cliente)
├── shared/                ← tipos de dominio compartidos
├── instalador/            ← instalar.bat, desinstalar.bat (fase 4, aún no creada)
├── docs/                  ← documentación para personas
├── .ai-context/           ← contexto condensado para agentes de IA
└── tasks/                 ← tareas con pasos de ejecución y validaciones
```

## Requisitos de desarrollo

| Herramienta | Versión mínima | Para qué |
|---|---|---|
| Rust (rustup o paquete del sistema) | 1.80 | API, shared y backend de la app |
| Node.js + npm | 20 | Frontend de la app y CLI de Tauri |
| Linux: `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator` | — | Compilar Tauri en desarrollo |
| macOS: Xcode Command Line Tools (`xcode-select --install`) | — | Compilar Tauri en desarrollo |
| Windows: WebView2 + MSVC Build Tools | — | Compilar los entregables finales |

> El desarrollo puede hacerse en Linux o macOS. Los entregables (`.exe`, `.msi`) se compilan en
> Windows o con compilación cruzada; ver [`docs/06-instalacion.md`](docs/06-instalacion.md).

## Compilar y ejecutar

### Inicio rápido (primera vez)

```bash
git clone <url-del-repositorio> Cesfam-Boxs
cd Cesfam-Boxs
git switch develop

# Terminal 1: levantar la API
GESTIONBOX_DATA_DIR=./datos-dev cargo run -p gestion-box-api

# Terminal 2: abrir la app (la primera compilación tarda varios minutos)
cd app
npm ci
npm run tauri dev
```

Con la API corriendo en el mismo PC, la app la encuentra sola (prueba dirección guardada →
`localhost` → broadcast UDP). Si no la encuentra, pide ingresar la dirección a mano:
usa `127.0.0.1:8000`.

### Comandos individuales

```bash
# API en modo desarrollo (datos en ./datos-dev en vez de C:\ProgramData\GestionBox)
GESTIONBOX_DATA_DIR=./datos-dev cargo run -p gestion-box-api

# Probar que responde
curl http://localhost:8000/health

# App de escritorio en modo desarrollo
cd app
npm install
npm run tauri dev

# Compilación release
cargo build -p gestion-box-api --release      # → target/release/gestion-box-api(.exe)
cd app && npm run tauri build                 # → instalador .msi / .exe (en Windows)
```

Variables de entorno de la API (todas opcionales):

| Variable | Por defecto | Descripción |
|---|---|---|
| `GESTIONBOX_DATA_DIR` | `C:\ProgramData\GestionBox` (Windows) / `./datos-dev` (otros) | Carpeta de datos, respaldos y log |
| `GESTIONBOX_HTTP_PORT` | `8000` | Puerto HTTP |
| `GESTIONBOX_UDP_PORT` | `8001` | Puerto UDP de descubrimiento |
| `RUST_LOG` | `info` | Nivel de log (`tracing`) |

## Pruebas y calidad

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && npm run check     # tipos TypeScript

# Prueba de humo: levanta la API en una carpeta temporal y recorre todos los endpoints
bash api/scripts/humo.sh
```

Una tarea solo está terminada cuando pasan sus **validaciones** (ver `tasks/`) y los cuatro
comandos anteriores.

## Flujo de ramas

| Rama | Uso | Quién la toca |
|---|---|---|
| `main` | Versión entregada / estable | Solo el responsable del proyecto |
| `develop` | Integración | Solo el responsable del proyecto (merge de PRs) |
| `dev/<tema>` | Trabajo diario: una rama por tarea o grupo de tareas | Colaboradores y agentes |

Los agentes de IA trabajan **exclusivamente** en ramas `dev/...`. Detalle en [`AGENTS.md`](AGENTS.md).

## Hoja de ruta (fases)

| Fase | Contenido | Tareas |
|---|---|---|
| 0 | Validación en terreno (antes de programar) | [`tasks/fase-0`](tasks/fase-0) |
| 1 | Proyecto base: workspace, tipos compartidos, README | [`tasks/fase-1`](tasks/fase-1) |
| 2 | API: esquema, endpoints, UDP, respaldos, log | [`tasks/fase-2`](tasks/fase-2) |
| 3 | App Tauri: conexión, grilla, reservas, boxes | [`tasks/fase-3`](tasks/fase-3) |
| 4 | Instalación: release, `instalar.bat`, instalador de la app | [`tasks/fase-4`](tasks/fase-4) |
| 5 | Pruebas de aceptación y entrega | [`tasks/fase-5`](tasks/fase-5) |

El estado de cada tarea está en [`tasks/README.md`](tasks/README.md).

## Documentación

- [`docs/`](docs/README.md) — documentación para personas: visión, requisitos, arquitectura,
  contrato de la API, app, instalación, manual del funcionario y pruebas de aceptación.
- [`.ai-context/`](.ai-context/README.md) — contexto compacto para agentes de IA.
- [`tasks/`](tasks/README.md) — tareas con pasos, buenas prácticas y validaciones.
- Documento fuente: `Gestión de Boxes CESFAM — Requisitos y tareas.docx`. No está versionado
  en el repositorio; pídelo al responsable del proyecto.
