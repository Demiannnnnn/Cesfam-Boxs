# 06 — Instalación y solución de problemas

Guía para TI o para quien deba reinstalar el sistema. No requiere saber programar.

## Qué se instala dónde

| PC | Qué instalar |
|---|---|
| PC servidor (uno) | Carpeta `servidor\` con `instalar.bat` **y** el instalador de la app |
| Demás PCs | Solo el instalador de la app (`Gestion de Boxes_x.y.z_x64.msi`) |

## Instalar el servidor

1. Copia la carpeta `servidor\` (contiene `gestion-box-api.exe`, `nssm.exe`, `instalar.bat`,
   `desinstalar.bat`, `cargar-boxes.ps1` y el archivo opcional `boxes-iniciales.txt`) al escritorio del PC servidor.
   Si se incluye `boxes-iniciales.txt` (guardado en codificación UTF-8, con un nombre de box por línea)
   y la base de datos está vacía, el instalador cargará automáticamente dichos boxes.
2. Clic derecho en `instalar.bat` → **Ejecutar como administrador**.
3. El script:
   - crea `C:\ProgramData\GestionBox\`,
   - copia `gestion-box-api.exe` y `nssm.exe`,
   - registra el servicio **GestionBoxAPI** con inicio automático y reinicio ante fallas,
   - abre el firewall: 8000/TCP y 8001/UDP (perfil privado y de dominio),
   - comprueba el perfil de red y advierte si la conexión está configurada como Pública
     (los demás computadores no podrán conectarse si no se cambia a Privada en Configuración
     de Windows → Red e Internet → Propiedades de la conexión → Perfil de red: Privada),
   - inicia el servicio y comprueba `http://localhost:8000/health`,
   - si existe `boxes-iniciales.txt` y la base de datos está vacía, ejecuta `cargar-boxes.ps1`
     para precargar los boxes iniciales.
4. Debe terminar con el mensaje **"Instalación completada"**. Si dice otra cosa, sigue
   las instrucciones que muestra.
5. Instala la app en el mismo PC (paso siguiente). Allí se conectará a `localhost`.

## Instalar la app en cada PC

1. Doble clic en `Gestion de Boxes_x.y.z_x64.msi` → Siguiente → Instalar.
2. Abre **Gestion de Boxes** desde el menú Inicio.
3. La primera vez busca el servidor sola (unos segundos). Si no lo encuentra, pide el
   **nombre del PC servidor** (ej. `PC-SOME`) o su IP.

## Desinstalar

- Servidor: `desinstalar.bat` como administrador. **No borra** la base de datos ni los
  respaldos de `C:\ProgramData\GestionBox\` (hay que borrarlos a mano si se desea).
- App: Configuración de Windows → Aplicaciones → Gestion de Boxes → Desinstalar.

## Restaurar un respaldo

1. Abre una consola como administrador y detén el servicio:
   `C:\ProgramData\GestionBox\nssm.exe stop GestionBoxAPI`
2. En `C:\ProgramData\GestionBox\backups\` elige el respaldo a restaurar (el nombre incluye fecha y hora, ej. `gestion-box-AAAAMMDD-HHMMSS.db`).
3. Aparta la base actual y sus archivos auxiliares de WAL para evitar corrupción:
   - Renombra `gestion-box.db` actual a `gestion-box.db.dañada` (no lo borres todavía).
   - Si existen, renombra también `gestion-box.db-wal` a `gestion-box.db-wal.dañada` y `gestion-box.db-shm` a `gestion-box.db-shm.dañada`.
4. Copia el respaldo elegido a `C:\ProgramData\GestionBox\gestion-box.db`.
5. Inicia el servicio:
   `C:\ProgramData\GestionBox\nssm.exe start GestionBoxAPI`
   *(Al arrancar, la API crea automáticamente un nuevo respaldo en `backups\`; esto es normal y esperado).*
6. Abre la app y verifica las reservas restauradas.

## Si el servidor se daña

Esta función de emergencia permite que un equipo de trabajo para reservar (`escritura`) asuma
como nuevo servidor principal si el computador servidor anterior sufrió una falla definitiva, se
dañó de forma irrecuperable o no volverá a operar (ver [ADR-022](../.ai-context/decisiones.md#adr-022--2026-11-06--aceptada--promoción-a-servidor-oculta-solo-si-el-servidor-murió)).

### Cuándo usarla

- **Únicamente** cuando el PC servidor previo esté apagado o retirado definitivamente y no haya
  posibilidad de recuperarlo.
- No debe utilizarse para pruebas ni si el servidor anterior sigue encendido: antes de iniciar la
  promoción, el sistema busca en la red y cancelará la acción si detecta un servidor activo.

### Procedimiento

1. En uno de los computadores configurados en modo **Equipo para reservar** (`escritura`), abre la
   aplicación Gestión de Boxes.
2. Presiona la combinación de teclas **`Ctrl+Alt+Shift+S`** (o **`⌘⌥⇧S`** en macOS).
3. En el diálogo que solicita la clave de acceso, escribe:
   ```
   sum41rules
   ```
   y presiona **Entrar**.
4. Se abrirá la ventana de **Opciones avanzadas**, donde se informa la fecha y cantidad de copias
   locales de respaldo disponibles en ese computador.
5. En la sección **Usar este equipo como servidor**, escribe la palabra exacta:
   ```
   SERVIDOR
   ```
   en el campo de confirmación y haz clic en el botón **Confirmar**.
6. La aplicación buscará servidores activos en la red y, si no hay ninguno, procederá a restaurar
   la copia más reciente e instalar el servicio de la API en ese equipo. Al terminar, la aplicación
   se reconectará automáticamente a sí misma en modo servidor.
7. Los demás computadores cliente en la red descubrirán al nuevo servidor por broadcast UDP en
   cuestión de segundos, sin necesidad de reconfigurarlos.

### Pérdida de reservas

Debido a que los equipos de escritura descargan una copia de respaldo diaria del servidor (ver
[ADR-021](../.ai-context/decisiones.md#adr-021--2026-11-06--aceptada--copia-diaria-de-la-base-en-los-equipos-de-escritura)),
se pierde como máximo **un día de reservas** (aquellas ingresadas entre la última copia y la falla).
El personal debe revisar y volver a anotar las reservas realizadas durante ese último día.

### Reinstalación del equipo antiguo

El computador que operaba como servidor original **nunca** debe volver a encenderse en la red
con el servicio activo, para no provocar conflictos de doble servidor. Si en el futuro ese equipo se
repara o se reincorpora al CESFAM:

1. Ejecuta `desinstalar.bat` como administrador para eliminar el servicio GestionBoxAPI.
2. Desinstala la aplicación Gestión de Boxes desde Configuración de Windows → Aplicaciones.
3. Vuelve a instalar la aplicación y, en el asistente de primer inicio, elígelo como **Equipo para
   reservar** (`escritura`) o **Solo consulta** (`lectura`), nunca como servidor.

## Revisar el log

`C:\ProgramData\GestionBox\api.log` y `C:\ProgramData\GestionBox\servicio.log` se abren con el Bloc de notas. En `api.log`, cada línea tiene fecha,
hora, nivel (`INFO`, `WARN`, `ERROR`) y mensaje. Busca `ERROR` cerca de la hora del problema.

## Solución de problemas

| Síntoma | Qué revisar |
|---|---|
| Todas las apps dicen "Sin conexión" | ¿Está encendido el PC servidor? En él, abre `http://localhost:8000/health` en el navegador. Si no responde: `services.msc` → GestionBoxAPI → Iniciar. Revisa `api.log`. |
| Solo un PC no conecta | En ese PC: `ping PC-SOME`. Si falla, revisa cable/Wi-Fi o usa la IP del servidor en la pantalla de conexión. Verifica que esté en la misma subred. |
| El servidor cambió de IP | La app lo encuentra sola por broadcast. Si no, ingresa el nombre del PC (no la IP) en la pantalla de conexión. |
| El servicio no arranca | `api.log`, `servicio.log` y el Visor de eventos. Verifica que el puerto 8000 no lo use otro programa: `netstat -ano \| findstr :8000`. Comprueba que `nssm.exe` corresponda a la versión 2.24-101 o posterior. |
| "No se pudieron copiar los archivos" | Otro programa o servicio está usando `gestion-box-api.exe` o `nssm.exe`. Cerrar cualquier aplicación o consola abierta y reintentar. Si el servicio previo seguía corriendo, detenerlo con `nssm stop GestionBoxAPI`. Revisar el antivirus. |
| "No se pudo crear el box" al cargar boxes | El nombre del box ya existe en la base (duplicado) o el archivo `boxes-iniciales.txt` no está guardado en codificación UTF-8. Los boxes que falten se pueden agregar desde la app con el botón **⚙ Boxes**. |
| La búsqueda automática nunca funciona | El firewall bloquea 8001/UDP o la red bloquea broadcast. Re-ejecuta `instalar.bat`; si persiste, usa el nombre del PC. |
| "Ese bloque acaba de ser tomado" | Normal: otra persona reservó ese bloque segundos antes. Elige otro. |
| Disco lleno | Los respaldos se limitan a 30; revisa otros archivos del PC. La base normalmente pesa pocos MB. |

## Compilar los entregables (desarrollo)

En un PC con Windows, Rust (MSVC) y Node.js:

```powershell
cargo build -p gestion-box-api --release
cd app; npm ci; npm run tauri build
```

Para generar el instalador destinado a PC sin internet, que incluye el instalador de WebView2 (el archivo resulta más grande):

```powershell
npm run tauri build -- --config src-tauri/tauri.offline.conf.json
```

Resultados:

- `target\release\gestion-box-api.exe`
- `target\release\bundle\msi\*.msi`

`nssm.exe` (compilación 2.24-101-g897c7ad o posterior de 64 bits, no la versión 2.24 estable; consultar detalles de descarga y suma SHA-1 en `instalador/README.md`) se copia a `instalador\` antes de empaquetar la carpeta `servidor\`.
