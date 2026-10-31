# 05 — App de escritorio

Una sola pantalla, textos en español, acciones de un clic (RNF-11).

## Pantalla principal

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Gestión de Boxes ● Conectado a PC-SOME [Buscar][Exportar][Funcionarios][Etiquetas][⚙ Boxes]│
│ [Día|Semana] [◄ Anterior] [Hoy] [Siguiente ►] [ 22/10/2026 📅 ] [Buscar funcionario…][Todas las etiquetas▾]│
│ [Enfermería] [Cirugía Menor]                                                           │
├────────┬──────────────── MAÑANA ───────────────┬──────────── TARDE ────────────────────┤
│ Box    │ 08:00  09:00  10:00  11:00  12:00 13:00│ 14:00  15:00  16:00 …                 │
├────────┼────────────────────────────────────────┼───────────────────────────────────────┤
│ Box 3  │       [ Nutri. Javiera · Ingresos ]    │                                       │
│ Box 17 │ [ E.U. Olivia ─────────────────────── ]│                                       │
│ Box 12 │                                        │ [ E.U. Verónica ]                     │
└────────┴────────────────────────────────────────┴───────────────────────────────────────┘
```

- Boxes en filas (fijas a la izquierda) y bloques de 30 min en columnas; el encabezado
  muestra una etiqueta por hora y queda fijo al desplazarse.
- Si un box tiene una etiqueta activa asignada, su encabezado de fila muestra una franja y
  un chip con el color de la etiqueta y su nombre en `title` (RF-14).
- Sobre la grilla se muestra una leyenda compacta con las etiquetas activas (color + nombre).
- Una línea gruesa separa **mañana** y **tarde** (`shared::horario::HORA_INICIO_TARDE`),
  igual que "BOX AM / BOX PM" en la planilla que usaba el CESFAM.
- Celda libre: blanca, al pasar el mouse muestra "+". Clic → modal de reserva.
- Bloques seguidos del mismo box con el mismo funcionario y motivo se dibujan como **una
  sola barra** con el nombre (y el motivo debajo). Si el funcionario tiene una etiqueta
  activa, la barra usa el color de esa etiqueta con texto negro o blanco según contraste WCAG;
  sin etiqueta, usa el azul por defecto. Clic → detalle del tramo; **Cancelar reserva** libera
  todos sus bloques.
- Para hoy, la hora en curso se resalta en el encabezado y los bloques libres ya pasados se
  ven rayados, pero siguen siendo clicables (el CESFAM puede registrar algo a posteriori).
- **Anterior / Siguiente** saltan sábados y domingos (la planilla solo planificaba de lunes a
  viernes); el selector de fecha permite ir a cualquier día.

## Vista semanal

El selector **Día | Semana** (a la izquierda de la navegación) cambia a una tabla de lunes a
viernes con dos columnas por día, **AM** y **PM**, como la planilla:

- Cada celda dice quién tiene el box ese medio día y usa el color de la etiqueta del
  funcionario (o azul/celeste si no tiene etiqueta). Clic → abre ese día en la vista Día.
- Celda vacía → reservar ese box ese medio día (el modal propone el periodo completo).
- La fila final **Libres** (fija abajo) lista los boxes sin ninguna reserva en cada medio
  día: reemplaza la hoja "BOX DISPONIBLE" que el CESFAM armaba a mano.
- **Anterior / Siguiente** avanzan una semana.

## Filtros rápidos y búsqueda (RF-15)

- El campo **Buscar funcionario…** (a la derecha de la barra de navegación) resalta en ambas
  vistas las reservas del funcionario buscado (comparando por funcionario registrado, sin
  distinguir tildes ni mayúsculas) y atenúa el resto: responde "¿dónde está Verónica esta
  semana?". Sugiere los funcionarios activos registrados.
- A su lado, el selector **Todas las etiquetas** resalta (y atenúa el resto) las reservas cuyo
  funcionario o box tenga la etiqueta seleccionada, en vista Día y Semana.
- El botón **Buscar** en la barra superior (o el atajo `Ctrl+F` / `⌘F`) abre un modal con dos
  pestañas: **Reservas** y **Box libre**.

## Modales

| Modal | Campos / contenido | Botones |
|---|---|---|
| Buscar | Dos pestañas (`aria-pressed`): **Reservas** (funcionario opcional, etiqueta opcional, box opcional, rango desde/hasta por defecto hoy a 4 semanas; resultados agrupados en tramos con fecha corta, horario, box y funcionario con chip; clic salta a ese día en vista Día) y **Box libre** (fecha visible por defecto, horario desde/hasta con atajos *Mañana · Tarde · Día completo*, etiqueta opcional; lista de boxes libres con chip y botón *Reservar* que abre el modal de reserva con datos precargados) | Buscar · Reservar · Cerrar |
| Reservar | Box y fecha (solo lectura); **Horario** desde/hasta con atajos *Mañana · Tarde · Día completo*; **Nombre del funcionario** (buscador de funcionarios registrados activos; muestra el chip de color de su etiqueta; si no existe ofrece registrarlo en línea); Motivo (opcional); *Repetir cada semana hasta el…*; resumen | Reservar · (Reservar solo los libres) · (Registrar «nombre») · Cerrar |
| Detalle | Box, fecha, horario (desde – hasta), funcionario, etiqueta (si tiene), motivo, creada el… | Cancelar reserva — o, si es parte de una serie, *Cancelar solo este día* · *Cancelar este día y los siguientes* (piden confirmación) · Cerrar |
| Funcionarios | Buscador por nombre arriba; lista de funcionarios con nombre, selector de etiqueta ("Sin etiqueta" + activas con muestra de color) y casilla "Activo"; abajo "Nuevo funcionario" | Guardar · Agregar · Cerrar |
| Etiquetas | Lista de etiquetas con nombre, color (`<input type="color">` + paleta de 8 colores sugeridos) y casilla "Activa"; abajo "Nueva etiqueta" | Guardar · Agregar · Cerrar |
| Boxes | Lista de boxes con nombre editable, selector de etiqueta y casilla "Activo"; abajo "Nuevo box" | Guardar · Agregar · Cerrar |
| Conexión | "Nombre del PC servidor o IP"; texto de ayuda | Conectar · Buscar de nuevo |

**Reservar un rango.** Un clic en una celda libre propone desde ese bloque hasta el fin de su
medio día (mañana o tarde), sin pisar la siguiente reserva del box: es el "BOX AM / BOX PM" de
la planilla. También se puede **arrastrar** sobre celdas libres seguidas de un box para elegir
el rango. Todo se envía en un solo `POST /reservas/lote` con `funcionario_id`. Si parte del
rango o de la serie ya está tomada no se reserva nada y el modal ofrece **Reservar solo los
libres**; el aviso final dice cuántos bloques se omitieron.

El identificador del último funcionario que reservó desde ese PC se recuerda localmente para
precargar el campo si sigue activo (comodidad, no identidad).

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

Cada **20 segundos** (dentro del rango 15–30 s) la app vuelve a pedir boxes, etiquetas,
funcionarios y reservas de lo visible. También refresca justo después de reservar, cancelar,
cerrar un modal de administración o cambiar de día. El refresco se pausa mientras hay un modal
abierto para no mover la grilla bajo el usuario.

## Mensajes de error (RNF-09)

Catálogo único en `app/src/textos.ts` (frontend) y `app/src-tauri/src/errores.rs` (backend).

| Situación | Mensaje |
|---|---|
| Servidor no responde (timeout / conexión rechazada) | "No se pudo conectar con el servidor. Verifica que {equipo} esté encendido y conectado a la red." |
| Descubrimiento sin respuesta | "No se encontró el servidor en la red. Verifica que el PC servidor esté encendido o ingresa su nombre manualmente." |
| Nombre de PC no resuelve | "No se encontró el equipo \"{nombre}\". Revisa que el nombre esté bien escrito o usa su dirección IP." |
| 409 `bloque_ocupado` | "Ese bloque acaba de ser tomado" |
| 409 `nombre_duplicado` | "Ya existe {un box / un funcionario / una etiqueta} con ese nombre." |
| 404 al cancelar | "Esa reserva ya había sido cancelada." |
| 422 `box_inactivo` | "Ese box está desactivado. Actívalo en Configuración de boxes." |
| 422 `funcionario_inactivo` | "Ese funcionario está desactivado. Actívalo en Funcionarios." |
| 422 `etiqueta_inactiva` | "Esa etiqueta está desactivada. Actívala en Etiquetas." |
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
  "ultimo_funcionario_id": 1
}
```

Se reescribe de forma atómica (archivo temporal + renombrar) cada vez que una conexión es
exitosa con una dirección distinta a la guardada o al reservar. Si existe el campo antiguo
`ultimo_funcionario` de texto, se ignora sin fallar.

## Exportar (RF-12, opcional)

Botón **Exportar** en la barra superior → pide rango (desde/hasta) → guarda un CSV con
separador `;` y codificación UTF-8 con BOM (para que Excel en español lo abra con tildes):

```
Fecha;Hora;Box;Funcionario;Etiqueta;Motivo
01-10-2026;08:00;Box 1;María Pérez;Enfermería;Control sano
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
│   ├── estado.ts             estado de la pantalla (día, boxes, etiquetas, funcionarios, reservas)
│   ├── tipos.ts              interfaces del contrato HTTP
│   ├── colores.ts            paleta de colores y contraste WCAG (colorTexto)
│   ├── fechas.ts             utilidades de fechas sin desfase UTC
│   ├── tramos.ts             agrupación de bloques contiguos en barras
│   ├── dom.ts                creación segura de nodos con textContent
│   ├── componentes/          grilla.ts, semana.ts, buscar.ts, reservar.ts, funcionarios.ts, etiquetas.ts, modales.ts, formulario.ts, dialogo.ts, aviso.ts, conexion.ts
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
