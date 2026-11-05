@echo off
chcp 65001 >nul
setlocal

:: Cambiar al directorio del script
cd /d "%~dp0"
if errorlevel 1 goto error_ruta_unc

:: Verificar privilegios de administrador
net session >nul 2>&1
if errorlevel 1 goto error_permisos_admin

:: Carpeta de datos compartida con la configuracion de la API
set "DESTINO=%ProgramData%\GestionBox"

:: Comprobar presencia de ejecutables requeridos
if not exist "%~dp0gestion-box-api.exe" goto error_falta_api
if not exist "%~dp0nssm.exe" goto error_falta_nssm

:: Crear carpetas de datos y respaldos
if not exist "%DESTINO%" mkdir "%DESTINO%"
if errorlevel 1 goto error_crear_carpetas
if not exist "%DESTINO%\backups" mkdir "%DESTINO%\backups"
if errorlevel 1 goto error_crear_carpetas

:: Comprobar si el servicio ya existe
set "SERVICIO_EXISTE=0"
sc query GestionBoxAPI >nul 2>&1
if errorlevel 1 goto servicio_no_existe
set "SERVICIO_EXISTE=1"
echo Deteniendo servicio existente para actualizar...
"%~dp0nssm.exe" stop GestionBoxAPI >nul 2>&1

:servicio_no_existe
:: Copiar ejecutables con reintentos si origen y destino son distintos
if /i "%~dp0"=="%DESTINO%\" goto saltar_copia

set "INTENTOS_COPIA=0"

:bucle_copia
set /a INTENTOS_COPIA+=1
copy /y "%~dp0gestion-box-api.exe" "%DESTINO%\gestion-box-api.exe" >nul 2>&1
if errorlevel 1 goto fallo_intento_copia
copy /y "%~dp0nssm.exe" "%DESTINO%\nssm.exe" >nul 2>&1
if errorlevel 1 goto fallo_intento_copia
goto copia_exitosa

:fallo_intento_copia
if %INTENTOS_COPIA% geq 5 goto copia_fallida_definitiva
ping -n 3 127.0.0.1 >nul
goto bucle_copia

:copia_fallida_definitiva
if not "%SERVICIO_EXISTE%"=="1" goto copia_fallida_sin_servicio
echo Error: No se pudieron copiar los archivos al directorio de destino
echo Intentando reanudar el servicio previo...
"%~dp0nssm.exe" start GestionBoxAPI >nul 2>&1
pause
exit /b 1

:copia_fallida_sin_servicio
echo Error: No se pudieron copiar los archivos a %DESTINO%
pause
exit /b 1

:saltar_copia
:copia_exitosa

:: Configurar servicio con NSSM usando la copia del destino
if "%SERVICIO_EXISTE%"=="1" goto servicio_existente_configurar
"%DESTINO%\nssm.exe" install GestionBoxAPI "%DESTINO%\gestion-box-api.exe" >nul
if errorlevel 1 goto error_instalacion_servicio
goto configurar_parametros_nssm

:servicio_existente_configurar
call :ajustar_nssm Application "%DESTINO%\gestion-box-api.exe"
if errorlevel 1 goto error_ajustar_nssm

:configurar_parametros_nssm
call :ajustar_nssm AppDirectory "%DESTINO%"
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm DisplayName "Gestión de Boxes CESFAM - API"
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm Start SERVICE_AUTO_START
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppExit Default Restart
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppRestartDelay 5000
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppStdout "%DESTINO%\servicio.log"
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppStderr "%DESTINO%\servicio.log"
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppRotateFiles 1
if errorlevel 1 goto error_ajustar_nssm
call :ajustar_nssm AppRotateBytes 1048576
if errorlevel 1 goto error_ajustar_nssm
:: AppNoConsole 1: el exe no usa consola y NSSM 2.24 estable falla al arrancar servicios en Windows 10 1703 o posterior sin este ajuste
call :ajustar_nssm AppNoConsole 1
if errorlevel 1 goto error_ajustar_nssm

:: Reglas de firewall para HTTP y UDP
echo Configurando reglas de firewall...
netsh advfirewall firewall delete rule name="GestionBox HTTP" >nul 2>&1
netsh advfirewall firewall delete rule name="GestionBox UDP" >nul 2>&1
netsh advfirewall firewall add rule name="GestionBox HTTP" dir=in action=allow protocol=TCP localport=8000 profile=private,domain >nul 2>&1
if errorlevel 1 goto error_firewall_http
netsh advfirewall firewall add rule name="GestionBox UDP" dir=in action=allow protocol=UDP localport=8001 profile=private,domain >nul 2>&1
if errorlevel 1 goto error_firewall_udp

:: Advertencia de perfil de red publica
powershell -NoProfile -Command "try { if (Get-NetConnectionProfile | Where-Object NetworkCategory -eq 'Public') { exit 1 } else { exit 0 } } catch { exit 0 }" >nul 2>&1
if errorlevel 1 goto advertencia_red_publica
goto red_verificada

:advertencia_red_publica
echo Advertencia: Se detecto una conexion de red configurada como Publica
echo Los demas computadores no podran conectarse a este servidor
echo Cambia el perfil de red a Privada en la configuracion de Windows

:red_verificada
:: Iniciar el servicio y verificar endpoint de salud
echo Iniciando servicio GestionBoxAPI...
"%DESTINO%\nssm.exe" start GestionBoxAPI >nul 2>&1

echo Esperando a que el servicio responda...
set "INTENTOS_HEALTH=0"

:bucle_health
set /a INTENTOS_HEALTH+=1
powershell -NoProfile -Command "try { $r = Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 -Uri 'http://localhost:8000/health'; if ($r.StatusCode -eq 200) { exit 0 } else { exit 1 } } catch { exit 1 }" >nul 2>&1
if not errorlevel 1 goto health_correcto
if %INTENTOS_HEALTH% geq 30 goto error_health_agotado
ping -n 3 127.0.0.1 >nul
goto bucle_health

:health_correcto
echo El servicio responde correctamente en http://localhost:8000/health

:: Carga opcional de boxes iniciales
if not exist "%~dp0boxes-iniciales.txt" goto fin_carga_boxes
if not exist "%~dp0cargar-boxes.ps1" goto aviso_falta_ps1

echo Cargando boxes iniciales desde boxes-iniciales.txt...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0cargar-boxes.ps1" -Archivo "%~dp0boxes-iniciales.txt"
if errorlevel 1 echo Advertencia: Ocurrio un problema al cargar los boxes iniciales
goto fin_carga_boxes

:aviso_falta_ps1
echo Advertencia: Se encontro boxes-iniciales.txt pero no existe cargar-boxes.ps1

:fin_carga_boxes
echo Instalación completada
pause
exit /b 0

:: Subrutinas y controladores de error

:ajustar_nssm
"%DESTINO%\nssm.exe" set GestionBoxAPI %* >nul
if errorlevel 1 goto fallo_ajuste_nssm
exit /b 0

:fallo_ajuste_nssm
echo Error al configurar el parametro %1 en NSSM
exit /b 1

:error_ruta_unc
echo Error: No se puede ejecutar desde una ruta de red
echo Copia la carpeta al escritorio e intenta nuevamente
pause
exit /b 1

:error_permisos_admin
echo Error: Se requieren permisos de administrador
echo Haz clic derecho en instalar.bat y selecciona Ejecutar como administrador
pause
exit /b 1

:error_falta_api
echo Error: No se encuentra gestion-box-api.exe junto a instalar.bat
pause
exit /b 1

:error_falta_nssm
echo Error: No se encuentra nssm.exe junto a instalar.bat
pause
exit /b 1

:error_crear_carpetas
echo Error: No se pudo crear la carpeta de datos en %DESTINO%
pause
exit /b 1

:error_instalacion_servicio
echo Error: No se pudo registrar el servicio GestionBoxAPI con NSSM
pause
exit /b 1

:error_ajustar_nssm
echo Error: Fallo la configuracion de los parametros de NSSM
pause
exit /b 1

:error_firewall_http
echo Error: No se pudo crear la regla de firewall para el puerto 8000 TCP
pause
exit /b 1

:error_firewall_udp
echo Error: No se pudo crear la regla de firewall para el puerto 8001 UDP
pause
exit /b 1

:error_health_agotado
echo Error: El servicio no responde en http://localhost:8000/health tras varios intentos
echo Revisa el archivo api.log en %DESTINO%
echo Revisa el archivo servicio.log en %DESTINO%
echo Revisa el estado del servicio en services.msc
echo Comprueba que ningun otro programa este usando el puerto 8000
pause
exit /b 1
