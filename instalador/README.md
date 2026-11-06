# instalador — Entregables y scripts para el servidor

Esta carpeta contiene los scripts necesarios para instalar y desinstalar el servicio
de la API en el PC servidor de CESFAM.

## Contenido de la carpeta `servidor\` para despliegue

Al preparar el paquete para el servidor, se debe reunir en una carpeta `servidor\` lo siguiente:

| Archivo | Origen |
|---|---|
| `gestion-box-api.exe` | Compilado en modo release (`target\release\gestion-box-api.exe`). No versionado. |
| `nssm.exe` | NSSM de 64 bits de la compilación 2.24-101-g897c7ad o posterior (descargar de `https://nssm.cc/ci/nssm-2.24-101-g897c7ad.zip`, SHA-1: `ca2f6782a05af85facf9b620e047b01271edd11d`, carpeta `win64`). **No usar la versión 2.24 estable**, ya que falla al iniciar servicios en Windows 10 versión 1703 o posterior. No versionado. |
| `instalar.bat` | Script de instalación automática del servicio y configuración de firewall. |
| `desinstalar.bat` | Script para detener y eliminar el servicio y reglas de firewall sin tocar la base de datos ni los respaldos. |
| `cargar-boxes.ps1` | Script PowerShell para inicialización automática de boxes desde archivo de texto. |
| `boxes-iniciales.txt` | *(Opcional)* Lista de nombres de boxes a precargar (uno por línea, guardado en codificación UTF-8). Si no se provee, los boxes se crean desde la app. |
| `boxes-iniciales.ejemplo.txt` | Plantilla de ejemplo con formato y comentarios de uso. Es solo plantilla de referencia; no hace falta llevarlo al servidor. |

## Uso

1. Copiar la carpeta completa `servidor\` al escritorio del equipo designado como servidor.
2. Hacer clic derecho sobre `instalar.bat` y seleccionar **Ejecutar como administrador**.
3. El script configurará el servicio en `C:\ProgramData\GestionBox`, abrirá los puertos en el firewall de Windows y verificará el estado en `http://localhost:8000/health`.

## Integración con el instalador de la app

Para que la app de escritorio pueda instalar el servidor automáticamente desde su asistente (T-8.2), los archivos `gestion-box-api.exe`, `nssm.exe`, `instalar.bat` y `desinstalar.bat` deben copiarse a `app/src-tauri/servidor/` antes de ejecutar `npm run tauri build`.

## Obtener los instaladores desde GitHub Actions (CI)

Para evitar la compilación y preparación manual en un entorno de desarrollo, el repositorio incluye el workflow automatizado **"Instalador Windows"** (`.github/workflows/instalador-windows.yml`):

1. El responsable del proyecto accede a la pestaña **Actions** en GitHub.
2. Selecciona el workflow **Instalador Windows** en la barra lateral izquierda y hace clic en el botón **Run workflow**.
3. Al finalizar la ejecución, se descarga el artefacto **`instalador-windows`** (archivo comprimido `.zip` que contiene los instaladores generados):
   - **`*-setup.exe`** (NSIS): Es el instalador más simple y recomendado para uso general en los computadores del CESFAM.
   - **`*.msi`** (WiX): Destinado a instalaciones corporativas administradas mediante directivas de grupo (GPO).
4. Durante la ejecución en GitHub Actions, la versión de NSSM utilizada se descarga y valida contra su hash SHA-256 (`99f5045fffbffb745d67fe3a065a953c4a3d9c253b868892d9b685b0ee7d07b8`), garantizando la integridad de los componentes del servidor incluidos en la app.
