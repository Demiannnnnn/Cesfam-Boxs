# AGENTS.md — Guía para agentes de IA y colaboradores

Este archivo define **cómo se trabaja** en este repositorio. Léelo completo antes de tocar
código. Si una instrucción de una tarea contradice este archivo, gana este archivo; si
tienes dudas, detente y pregunta.

## 1. Idioma

- Toda la documentación, los comentarios, los mensajes de commit, los textos de la interfaz
  y los mensajes de error se escriben **en español**.
- Los identificadores de código se escriben en español cuando son conceptos del dominio
  (`reserva`, `bloque`, `funcionario`, `crear_reserva`) y en inglés solo cuando son
  convenciones del lenguaje o de una librería (`new`, `from_str`, `IntoResponse`, `main`).
- `Box` es el nombre del dominio para un box de atención. En Rust choca con
  `std::boxed::Box`, por lo que el struct se llama **`BoxAtencion`** y la tabla SQL **`box`**.

## 2. Ramas — reglas estrictas

| Rama | Permitido para agentes |
|---|---|
| `main` | **Nada.** No hacer checkout para editar, commit, merge, rebase, pull ni push. |
| `develop` | **Nada.** Solo se usa como base de nuevas ramas (`git switch -c dev/x develop`). |
| `dev/<tema>` | Libertad completa: crear, commitear, rehacer, borrar. |

- Nombres: `dev/<fase>-<tema-corto>` en minúsculas con guiones. Ej.: `dev/fase-2-endpoints-reservas`.
- Una rama por tarea o grupo pequeño de tareas relacionadas.
- **No** ejecutar `git pull`, `git push --force` ni operaciones remotas sobre `main`/`develop`.
- El merge a `develop` lo hace el responsable humano mediante PR.

## 3. Commits

Formato *Conventional Commits* en español:

```
<tipo>(<ámbito>): <resumen en imperativo, minúsculas, sin punto>

<cuerpo opcional: qué y por qué>

Tareas: T-2.4, T-2.5
```

Tipos: `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `chore`.
Ámbitos: `api`, `app`, `shared`, `instalador`, `docs`, `tasks`, `ai-context`.

## 4. Cómo tomar una tarea

1. Abre [`tasks/README.md`](tasks/README.md) y elige la primera tarea `pendiente` cuyas
   dependencias estén `completada`.
2. Lee el archivo de la tarea completo (objetivo, contexto, pasos, patrones, validaciones).
3. Lee los documentos que la tarea enlaza en `.ai-context/` y `docs/`.
4. Crea la rama `dev/...` desde `develop` (o continúa la rama de trabajo actual si la tarea
   es continuación directa).
5. Implementa siguiendo los **pasos de ejecución**.
6. Ejecuta **todas** las validaciones de la tarea y los comandos de calidad (sección 7).
7. Marca la tarea como `completada` en `tasks/README.md` y en `.ai-context/estado.md`.
8. Commit con el ID de la tarea en el pie.

Las tareas de **fase 0** y parte de la **fase 5** son trabajo en terreno: un agente no puede
completarlas; solo puede preparar material (checklists, plantillas) y dejarlas `bloqueada`.

## 5. Arquitectura que no se negocia

Estas decisiones vienen del documento de requisitos. No las cambies sin aprobación:

- API y App son **procesos separados**. Cerrar la App nunca detiene la API.
- **Sin autenticación ni roles.**
- **Bloques fijos de 30 min** (08:00–20:00, índices 0–23). La doble reserva se evita con
  `UNIQUE(box_id, fecha, bloque)` en SQLite, nunca con lógica de "consultar y luego insertar".
- **Refresco periódico** (15–30 s), no WebSockets.
- **Descubrimiento en capas**: dirección guardada → `/health` → broadcast UDP → ingreso manual.
- **Un ejecutable por componente**, SQLite embebido (`rusqlite` con `bundled`), sin runtimes externos.
- **Rutas absolutas** en `C:\ProgramData\GestionBox\` en producción.
- **Todas las consultas SQL con parámetros** (`?1`, `params![]`). Prohibido `format!` en SQL.
- Toda la red de la App (HTTP y UDP) vive en el **lado Rust** de Tauri. El frontend solo usa
  `invoke`.

## 6. Convenciones de código

### Rust (api, shared, app/src-tauri)

- Edición 2021, `cargo fmt` sin configuración especial, `clippy` sin advertencias.
- Módulos pequeños con una sola responsabilidad. Un archivo > 300 líneas es una señal para dividir.
- Errores con `thiserror` en librerías/capas y un único `ErrorApi` que implementa
  `IntoResponse` en la capa HTTP. Prohibido `unwrap()`/`expect()` fuera de tests y de
  `main` durante el arranque (y ahí con mensaje claro).
- Validación en la frontera: convertir entradas a tipos válidos (`Bloque`, `Fecha`,
  `NombreFuncionario`) apenas llegan ("parsear, no validar").
- Documentar con `///` todo ítem público: qué hace, errores posibles y requisito que cubre
  (`/// Cubre RF-05.`).
- Tests unitarios junto al código (`#[cfg(test)] mod tests`) y tests de integración en
  `api/tests/`. La base de datos de tests es SQLite en memoria.

### TypeScript (app/src)

- Sin framework. Módulos ES pequeños: `api/`, `componentes/`, `estado/`, `util/`.
- `strict: true`. Nada de `any` salvo justificación comentada.
- Ningún texto visible hardcodeado fuera de `src/textos.ts` (catálogo único en español).
- Accesible: botones reales (`<button>`), etiquetas en formularios, contraste suficiente.

### Patrones de diseño usados

| Patrón | Dónde | Por qué |
|---|---|---|
| Repositorio | `api/src/repositorio/` | Aísla SQL del resto; permite tests con SQLite en memoria |
| Capa de servicio delgada | `api/src/servicio.rs` | Reglas de negocio (validar bloque, box activo) fuera de los handlers |
| Inyección por estado | `AppState` en Axum | Handlers sin globales; tests con estado propio |
| Error tipado → respuesta | `ErrorApi: IntoResponse` | Un solo lugar decide códigos HTTP y mensajes |
| Newtype / parsear-no-validar | `shared::Bloque`, `shared::Fecha` | Un `Bloque` inválido no puede existir |
| Cadena de responsabilidad | `app/src-tauri/src/conexion.rs` | Estrategias de descubrimiento probadas en orden |
| Fábrica de router | `api::http::crear_router(estado)` | Mismo router en producción y en tests |
| Configuración centralizada | `api/src/config.rs` | Rutas y puertos resueltos una sola vez |

## 7. Comandos de calidad (obligatorios antes de cada commit de código)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && npm run check
```

## 8. Dónde está cada cosa

| Necesito… | Ver |
|---|---|
| Contexto condensado del proyecto | `.ai-context/contexto-proyecto.md` |
| Contrato exacto de la API | `.ai-context/contrato-api.md` y `docs/04-api.md` |
| Decisiones y su porqué | `.ai-context/decisiones.md` |
| Convenciones detalladas | `.ai-context/convenciones.md` |
| Estado actual del trabajo | `.ai-context/estado.md` y `tasks/README.md` |
| Requisitos con ID (RF/RNF) | `docs/02-requisitos.md` |

## 9. Lo que nunca debes hacer

- Tocar `main` o `develop` (ver sección 2).
- Agregar autenticación, WebSockets, ORMs, bases de datos externas o runtimes (Node, .NET, Python) al entregable.
- Guardar datos de pacientes o campos no listados en los requisitos.
- Concatenar texto en SQL.
- Mostrar al usuario mensajes técnicos en inglés (`connection refused`, stack traces).
- Marcar una tarea como completada sin ejecutar sus validaciones.
- Borrar o reescribir el `.docx` de requisitos.
