# 05 — App de escritorio

Una sola pantalla, textos en español, acciones de un clic (RNF-11).

## Pantalla principal

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Gestión de Boxes                     ● Conectado a PC-SOME [Exportar][⚙ Boxes]│
│ [◄ Anterior] [Hoy] [Siguiente ►]   [ 22/10/2026 📅 ]   Jueves, 22 de octubre  │
├────────┬──────────────── MAÑANA ───────────────┬──────────── TARDE ──────────┤
│ Box    │ 08:00  09:00  10:00  11:00  12:00 13:00│ 14:00  15:00  16:00 …       │
├────────┼────────────────────────────────────────┼─────────────────────────────┤
│ Box 3  │       [ Nutri. Javiera · Ingresos ]    │                             │
│ Box 17 │ [ E.U. Olivia ─────────────────────── ]│                             │
│ Box 12 │                                        │ [ E.U. Verónica ]           │
└────────┴────────────────────────────────────────┴─────────────────────────────┘
```

- Boxes en filas (fijas a la izquierda) y bloques de 30 min en columnas; el encabezado
  muestra una etiqueta por hora y queda fijo al desplazarse.
- Una línea gruesa separa **mañana** y **tarde** (`shared::horario::HORA_INICIO_TARDE`),
  igual que "BOX AM / BOX PM" en la planilla que usaba el CESFAM.
- Celda libre: blanca, al pasar el mouse muestra "+". Clic → modal de reserva.
- Bloques seguidos del mismo box con el mismo funcionario y motivo se dibujan como **una
  sola barra** azul con el nombre (y el motivo debajo). Clic → detalle del tramo; **Cancelar
  reserva** libera todos sus bloques.
- Para hoy, la hora en curso se resalta en el encabezado y los bloques libres ya pasados se
  ven rayados, pero siguen siendo clicables (el CESFAM puede registrar algo a posteriori).
- **Anterior / Siguiente** saltan sábados y domingos (la planilla solo planificaba de lunes a
  viernes); el selector de fecha permite ir a cualquier día.

## Modales

| Modal | Campos / contenido | Botones |
|---|---|---|
| Reservar | Box, fecha y hora (solo lectura); **Nombre del funcionario** (obligatorio); Motivo (opcional) | Reservar · Cerrar |
| Detalle | Box, fecha, horario (desde – hasta), funcionario, motivo, creada el… | Cancelar reserva (pide confirmación) · Cerrar |
| Boxes | Lista de boxes con nombre editable y casilla "activo"; campo "Nuevo box" | Agregar · Guardar nombre · Cerrar |
| Conexión | "Nombre del PC servidor o IP"; texto de ayuda | Conectar · Buscar de nuevo |

El nombre del último funcionario que reservó desde ese PC se recuerda localmente para
rellenar el campo (comodidad, no identidad).

## Estado de conexión (RF-08)

Indicador en la esquina superior derecha:

| Estado | Visual | Cuándo |
|---|---|---|
| Conectado | ● verde "Conectado a PC-SOME" | Último `/health` o petición correcta |
| Buscando | ● amarillo "Buscando servidor…" | Durante el descubrimiento |
| Sin conexión | ● rojo "Sin conexión" + mensaje de qué revisar | Falla una petición o el `/health` periódico |

Mientras no hay conexión la grilla se mantiene visible (último estado conocido) pero
atenuada y sin permitir reservar. La app reintenta sola en cada ciclo de refresco.

## Refresco (RF-07)

Cada **20 segundos** (dentro del rango 15–30 s) la app vuelve a pedir boxes y reservas del
día visible. También refresca justo después de reservar, cancelar o cambiar de día. El
refresco se pausa mientras hay un modal abierto para no mover la grilla bajo el usuario.

## Mensajes de error (RNF-09)

Catálogo único en `app/src/textos.ts` (frontend) y `app/src-tauri/src/errores.rs` (backend).

| Situación | Mensaje |
|---|---|
| Servidor no responde (timeout / conexión rechazada) | "No se pudo conectar con el servidor. Verifica que {equipo} esté encendido y conectado a la red." |
| Descubrimiento sin respuesta | "No se encontró el servidor en la red. Verifica que el PC servidor esté encendido o ingresa su nombre manualmente." |
| Nombre de PC no resuelve | "No se encontró el equipo \"{nombre}\". Revisa que el nombre esté bien escrito o usa su dirección IP." |
| 409 `bloque_ocupado` | "Ese bloque acaba de ser tomado" |
| 409 `nombre_duplicado` | "Ya existe un box con ese nombre." |
| 404 al cancelar | "Esa reserva ya había sido cancelada." |
| 422 `box_inactivo` | "Ese box está desactivado. Actívalo en Configuración de boxes." |
| 500 / desconocido | "Ocurrió un error en el servidor. Si se repite, avisa a TI y menciona la hora." |

`{equipo}` es el nombre del PC servidor conocido (guardado o descubierto); si no se conoce,
se usa "el PC servidor".

## Configuración local (RF-10)

Archivo JSON en la carpeta de configuración de la app
(`%APPDATA%\cl.cesfam.gestionbox\cliente.json` en Windows):

```json
{
  "servidor": "PC-SOME:8000",
  "equipo_servidor": "PC-SOME",
  "ultimo_funcionario": "María Pérez"
}
```

Se reescribe de forma atómica (archivo temporal + renombrar) cada vez que una conexión es
exitosa con una dirección distinta a la guardada.

## Exportar (RF-12, opcional)

Botón **Exportar** junto a la navegación de fechas → pide rango (desde/hasta) → guarda un CSV
con separador `;` y codificación UTF-8 con BOM (para que Excel en español lo abra con tildes):

```
Fecha;Hora;Box;Funcionario;Motivo
01-10-2026;08:00;Box 1;María Pérez;Control sano
```

## Estructura del código

```
app/
├── index.html
├── package.json / tsconfig.json / vite.config.ts
├── src/                      interfaz (TypeScript)
│   ├── main.ts               arranque y ciclo de refresco
│   ├── textos.ts             todos los textos visibles
│   ├── api.ts                envoltorios tipados de invoke()
│   ├── estado.ts             estado de la pantalla (día, boxes, reservas, conexión)
│   ├── horario.ts            bloque ↔ hora, formateo de fechas
│   ├── componentes/          grilla.ts, modales.ts, conexion.ts
│   └── estilos.css
└── src-tauri/                backend (Rust)
    ├── tauri.conf.json
    └── src/
        ├── main.rs / lib.rs  registro de comandos
        ├── comandos.rs       funciones #[tauri::command]
        ├── cliente_api.rs    cliente HTTP tipado (reqwest + shared)
        ├── conexion.rs       estrategias de descubrimiento (cadena de responsabilidad)
        ├── config_local.rs   lectura/escritura de cliente.json
        ├── exportar.rs       CSV
        └── errores.rs        error → mensaje en español
```
