# 05 — App de escritorio

Una sola pantalla, textos en español, acciones de un clic (RNF-11).

## Pantalla principal

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Gestión de Boxes                         ● Conectado a PC-SOME   [⚙ Boxes]  │
│ [◄ Anterior] [Hoy] [Siguiente ►]   [ 01/10/2026 📅 ]   Miércoles 1 de octubre│
├────────────┬───────────────┬───────────────┬───────────────┬────────────────┤
│ Bloque     │ Box 1         │ Box 2         │ Box Dental    │ Box 4          │
├────────────┼───────────────┼───────────────┼───────────────┼────────────────┤
│ 08:00      │ María Pérez   │               │               │                │
│ 08:30      │               │ Juan Soto     │               │                │
│ 09:00      │               │               │               │                │
│ …          │               │               │               │                │
│ 19:30      │               │               │               │                │
└────────────┴───────────────┴───────────────┴───────────────┴────────────────┘
```

- Celda libre: fondo claro, al pasar el mouse dice "Reservar". Clic → modal de reserva.
- Celda ocupada: muestra el nombre del funcionario (y el motivo como texto secundario).
  Clic → modal de detalle con botón **Cancelar reserva**.
- La fila del bloque actual (si se ve el día de hoy) se resalta.
- Bloques pasados del día de hoy se ven atenuados, pero siguen siendo clicables
  (el CESFAM puede registrar algo a posteriori).

## Modales

| Modal | Campos / contenido | Botones |
|---|---|---|
| Reservar | Box, fecha y hora (solo lectura); **Nombre del funcionario** (obligatorio); Motivo (opcional) | Reservar · Cerrar |
| Detalle | Box, fecha, hora, funcionario, motivo, creada el… | Cancelar reserva (pide confirmación) · Cerrar |
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
