# instalador — Entregables y scripts para el servidor

Esta carpeta contiene los scripts necesarios para instalar y desinstalar el servicio
de la API en el PC servidor de CESFAM.

## Contenido de la carpeta `servidor\` para despliegue

Al preparar el paquete para el servidor, se debe reunir en una carpeta `servidor\` lo siguiente:

| Archivo | Origen |
|---|---|
| `gestion-box-api.exe` | Compilado en modo release (`target\release\gestion-box-api.exe`). No versionado. |
| `nssm.exe` | NSSM de 64 bits de la compilación 2.24-101-g897c7ad o posterior (descargar de `https://nssm.cc/ci/nssm-2.24-101-g897c7ad.zip`, carpeta `win64`). **No usar la versión 2.24 estable**, ya que falla al iniciar servicios en Windows 10 versión 1703 o posterior. No versionado. |
| `instalar.bat` | Script de instalación automática del servicio y configuración de firewall. |
| `desinstalar.bat` | Script para detener y eliminar el servicio y reglas de firewall sin tocar la base de datos ni los respaldos. |
| `cargar-boxes.ps1` | Script PowerShell para inicialización automática de boxes desde archivo de texto. |
| `boxes-iniciales.txt` | *(Opcional)* Lista de nombres de boxes a precargar (uno por línea, guardado en codificación UTF-8). Si no se provee, los boxes se crean desde la app. |
| `boxes-iniciales.ejemplo.txt` | Plantilla de ejemplo con formato y comentarios de uso. |

## Uso

1. Copiar la carpeta completa `servidor\` al escritorio del equipo designado como servidor.
2. Hacer clic derecho sobre `instalar.bat` y seleccionar **Ejecutar como administrador**.
3. El script configurará el servicio en `C:\ProgramData\GestionBox`, abrirá los puertos en el firewall de Windows y verificará el estado en `http://localhost:8000/health`.
