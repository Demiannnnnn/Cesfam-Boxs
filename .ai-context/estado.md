# Estado del proyecto

Última actualización: 2026-11-06.

## Resumen

| Fase | Estado |
|---|---|
| 0 — Terreno | bloqueada (requiere visita; supuestos del `.docx` en uso) |
| 1 — Proyecto base | completada (T-1.1, T-1.2, T-1.3) |
| 2 — API | completada: tests + `api/scripts/humo.sh` OK |
| 3 — App | implementada: 23 tests Rust + `tsc` OK; verificada a mano la grilla conectada por localhost. Falta validar modales/refresco/broadcast a mano (ver cada T-3.x) |
| 4 — Instalación | implementada, validación manual pendiente: scripts y configuración listos; falta generar el `.msi` y probar en Windows (ver cada T-4.x) |
| 5 — Entrega | manual y guía implementados, validación manual pendiente, PDF en `docs/pdf/`; T-5.1, T-5.4 y T-5.5 bloqueadas (terreno) |
| 6 — Grilla agenda, rangos y semana | implementada (ADR-014, ADR-015, ADR-016) |
| 7 — Funcionarios, etiquetas y búsqueda | en curso: T-7.1 completada, T-7.2 y T-7.3 implementadas (validación manual pendiente) |
| 8 — Varios equipos | en curso (ADR-020 a ADR-022, T-8.1 a T-8.7): T-8.3 completada, T-8.1 y T-8.4 implementadas (validación manual pendiente) |

## Ramas activas

| Rama | Contenido |
|---|---|
| `dev/documentacion-base` | README, AGENTS.md, docs/, .ai-context/, tasks/ |
| `dev/fase-1-2-api` | workspace, crate shared, API completa (sobre `dev/documentacion-base`) |
| `dev/fase-3-app` | app Tauri completa (sobre `dev/fase-1-2-api`) |
| `dev/fase-8-multiequipo` | fase 8: modos de equipo, copia diaria y servidor de reemplazo |

## Supuestos vigentes (hasta cerrar fase 0)

- Horario 08:00–20:00, bloques de 30 min (T-0.1).
- Lista de boxes desconocida: se cargan desde el modal (T-0.2).

## Próximo paso

Validaciones manuales de fases 3 y 4 en Windows: compilar los entregables, generar el `.msi` y probar `instalar.bat` en una VM.
