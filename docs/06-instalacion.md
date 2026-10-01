# 06 — Instalación y solución de problemas

Guía para TI o para quien deba reinstalar el sistema. No requiere saber programar.

## Qué se instala dónde

| PC | Qué instalar |
|---|---|
| PC servidor (uno) | Carpeta `servidor\` con `instalar.bat` **y** el instalador de la app |
| Demás PCs | Solo el instalador de la app (`Gestion de Boxes_x.y.z_x64.msi`) |

## Instalar el servidor

1. Copia la carpeta `servidor\` (contiene `gestion-box-api.exe`, `nssm.exe`, `instalar.bat`,
   `desinstalar.bat`) al escritorio del PC servidor.
2. Clic derecho en `instalar.bat` → **Ejecutar como administrador**.
3. El script:
   - crea `C:\ProgramData\GestionBox\`,
   - copia `gestion-box-api.exe` y `nssm.exe`,
   - registra el servicio **GestionBoxAPI** con inicio automático y reinicio ante fallas,
   - abre el firewall: 8000/TCP y 8001/UDP (perfil privado y de dominio),
   - inicia el servicio y comprueba `http://localhost:8000/health`.
4. Debe terminar con el mensaje **"Instalación completada"**. Si dice otra cosa, sigue
   las instrucciones que muestra.
5. Instala la app en el mismo PC (paso siguiente). Allí se conectará a `localhost`.

## Instalar la app en cada PC

1. Doble clic en `Gestion de Boxes_x.y.z_x64.msi` → Siguiente → Instalar.
2. Abre **Gestión de Boxes** desde el menú Inicio.
3. La primera vez busca el servidor sola (unos segundos). Si no lo encuentra, pide el
   **nombre del PC servidor** (ej. `PC-SOME`) o su IP.

## Desinstalar

- Servidor: `desinstalar.bat` como administrador. **No borra** la base de datos ni los
  respaldos de `C:\ProgramData\GestionBox\` (hay que borrarlos a mano si se desea).
- App: Configuración de Windows → Aplicaciones → Gestión de Boxes → Desinstalar.

## Restaurar un respaldo

1. Abre una consola como administrador y detén el servicio:
   `C:\ProgramData\GestionBox\nssm.exe stop GestionBoxAPI`
2. En `C:\ProgramData\GestionBox\backups\` elige el respaldo (el nombre incluye fecha y hora).
3. Renombra `gestion-box.db` actual a `gestion-box.db.dañada` (no lo borres todavía).
4. Copia el respaldo elegido a `C:\ProgramData\GestionBox\gestion-box.db`.
5. Inicia el servicio: `C:\ProgramData\GestionBox\nssm.exe start GestionBoxAPI`.
6. Abre la app y verifica las reservas.

## Revisar el log

`C:\ProgramData\GestionBox\api.log` se abre con el Bloc de notas. Cada línea tiene fecha,
hora, nivel (`INFO`, `WARN`, `ERROR`) y mensaje. Busca `ERROR` cerca de la hora del problema.

## Solución de problemas

| Síntoma | Qué revisar |
|---|---|
| Todas las apps dicen "Sin conexión" | ¿Está encendido el PC servidor? En él, abre `http://localhost:8000/health` en el navegador. Si no responde: `services.msc` → GestionBoxAPI → Iniciar. Revisa `api.log`. |
| Solo un PC no conecta | En ese PC: `ping PC-SOME`. Si falla, revisa cable/Wi-Fi o usa la IP del servidor en la pantalla de conexión. Verifica que esté en la misma subred. |
| El servidor cambió de IP | La app lo encuentra sola por broadcast. Si no, ingresa el nombre del PC (no la IP) en la pantalla de conexión. |
| El servicio no arranca | `api.log` y el Visor de eventos. Verifica que el puerto 8000 no lo use otro programa: `netstat -ano | findstr :8000`. |
| La búsqueda automática nunca funciona | El firewall bloquea 8001/UDP o la red bloquea broadcast. Re-ejecuta `instalar.bat`; si persiste, usa el nombre del PC. |
| "Ese bloque acaba de ser tomado" | Normal: otra persona reservó ese bloque segundos antes. Elige otro. |
| Disco lleno | Los respaldos se limitan a 30; revisa otros archivos del PC. La base normalmente pesa pocos MB. |

## Compilar los entregables (desarrollo)

En un PC con Windows, Rust (MSVC) y Node.js:

```powershell
cargo build -p gestion-box-api --release
cd app; npm ci; npm run tauri build
```

Resultados:

- `target\release\gestion-box-api.exe`
- `app\src-tauri\target\release\bundle\msi\*.msi` (o `target\release\bundle\msi\` en el workspace)

`nssm.exe` (64 bits) se descarga de <https://nssm.cc> y se copia a `instalador\` antes de empaquetar.
