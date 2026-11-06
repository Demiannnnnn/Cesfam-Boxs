# Contexto del proyecto

## Resumen

Reemplazo de la planilla Excel de reservas de boxes del CESFAM de Curicó (Chile). App de
escritorio Tauri en N PCs de la red local + API Axum/SQLite en 1 PC servidor Windows.
Una pantalla: grilla día × (boxes activos en columnas, 24 bloques de 30 min en filas).

## Restricciones duras

- Sin mantenimiento tras la entrega → todo arranca, se recupera y se explica solo.
- ≤ 3 usuarios simultáneos, DB < 100 MB, misma subred, Windows 10/11.
- Sin auth, sin roles, sin pacientes, sin acceso externo, sin notificaciones.
- Datos: box, fecha, bloque, nombre funcionario, motivo. Nada más.
- Bloques: 08:00–20:00, 30 min, índices 0..=23 (pendiente confirmar, T-0.1). Definidos en
  `shared::horario` → un solo lugar para cambiarlos.
- Doble reserva imposible por `UNIQUE(box_id, fecha, bloque)` → HTTP 409 `bloque_ocupado`
  → texto exacto **"Ese bloque acaba de ser tomado"**.
- API: HTTP `0.0.0.0:8000`, UDP `0.0.0.0:8001`, datos en `C:\ProgramData\GestionBox\`
  (override `GESTIONBOX_DATA_DIR`), log `api.log`, respaldos `backups\` (30 máx.).
- API como servicio NSSM, `windows_subsystem = "windows"` en release.
- Un ejecutable por componente; `rusqlite` con feature `bundled`.
- SQL siempre parametrizado.
- Descubrimiento: guardada → `/health` → broadcast UDP → manual; < 5 s.
- Tres modos de equipo (`servidor`, `escritura`, `lectura`), elegidos en asistente al primer inicio. Al menos dos de escritura contando el servidor ([ADR-020](decisiones.md#adr-020--2026-11-06--aceptada--tres-modos-de-equipo-elegidos-al-instalar)).
- Copia diaria automática en equipos de escritura (`GET /respaldo`), en `C:\ProgramData\GestionBox\copias-del-servidor\`, últimas 7 ([ADR-021](decisiones.md#adr-021--2026-11-06--aceptada--copia-diaria-de-la-base-en-los-equipos-de-escritura)).
- Servidor de reemplazo: promoción de emergencia oculta (`Ctrl+Alt+Shift+S`, clave `sum41rules`), niega si hay servidor vivo por UDP ([ADR-022](decisiones.md#adr-022--2026-11-06--aceptada--promoción-a-servidor-oculta-solo-si-el-servidor-murió)).
- Servidor no se suspende ni hiberna con corriente alterna (`powercfg`, RNF-14).
- Errores de red → mensaje en español que dice qué revisar.
- Todo en español (docs, UI, commits, comentarios).

## Stack

| Pieza | Tecnología |
|---|---|
| Lenguaje | Rust 2021 (workspace Cargo) |
| API | axum 0.8, tokio, tower-http (cors), rusqlite (bundled), tracing + tracing-subscriber, thiserror, chrono, serde |
| Shared | serde, chrono, thiserror |
| App backend | tauri 2, reqwest (rustls, json), tokio, serde_json |
| App frontend | TypeScript estricto, Vite, sin framework, CSS plano |
| Instalación | NSSM + `instalar.bat`; instalador Tauri `.msi` |

## Workspace

```
Cargo.toml          [workspace] members = ["shared", "api", "app/src-tauri"]
shared/             crate lib  gestion-box-shared
api/                crate bin  gestion-box-api   (+ lib para tests de integración)
app/src-tauri/      crate bin  gestion-box-app
app/src/            frontend TS
```

## Ramas

Agentes: solo `dev/*`. Nunca `main` ni `develop` (ni pull, ni push, ni merge).
