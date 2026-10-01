# .ai-context

Contexto **condensado y orientado a máquinas** para agentes de IA. A diferencia de `docs/`
(prosa para personas), aquí cada archivo es denso, sin redundancia y fácil de citar.

## Orden de lectura recomendado

1. `contexto-proyecto.md` — qué es, restricciones duras, stack. (Siempre.)
2. `estado.md` — qué está hecho y qué sigue. (Siempre.)
3. `convenciones.md` — cómo escribir código aquí. (Antes de programar.)
4. `contrato-api.md` — endpoints, tipos y errores exactos. (Si tocas api/, app/ o shared/.)
5. `decisiones.md` — por qué las cosas son como son. (Antes de proponer un cambio de diseño.)
6. `mapa-codigo.md` — dónde vive cada responsabilidad. (Para navegar.)

## Reglas de mantenimiento

- Si cambias el contrato de la API, actualiza `contrato-api.md` **y** `docs/04-api.md` en el mismo commit.
- Si tomas una decisión de diseño nueva, agrégala a `decisiones.md` con fecha (formato ADR corto).
- Al completar una tarea, actualiza `estado.md`.
- Nada de información que se pueda derivar leyendo el código en 10 segundos.
