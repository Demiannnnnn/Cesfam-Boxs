@echo off
chcp 65001 >nul
setlocal

:: Cambiar al directorio del script
cd /d "%~dp0"
if errorlevel 1 goto error_ruta_unc

:: Verificar privilegios de administrador
net session >nul 2>&1
if errorlevel 1 goto error_permisos_admin

:: Carpeta de datos compartida
set "DESTINO=%ProgramData%\GestionBox"

:: Comprobar si el servicio existe
sc query GestionBoxAPI >nul 2>&1
if errorlevel 1 goto servicio_no_existe

echo Deteniendo y eliminando servicio GestionBoxAPI...

:: Determinar ejecutable de NSSM a utilizar
set "NSSM_EXE="
if exist "%DESTINO%\nssm.exe" set "NSSM_EXE=%DESTINO%\nssm.exe"
if not defined NSSM_EXE if exist "%~dp0nssm.exe" set "NSSM_EXE=%~dp0nssm.exe"

if defined NSSM_EXE goto usar_nssm
echo No se encontró nssm.exe. Deteniendo y eliminando mediante sc.exe...
sc stop GestionBoxAPI >nul 2>&1
ping -n 3 127.0.0.1 >nul
sc delete GestionBoxAPI >nul 2>&1
goto verificar_servicio

:usar_nssm
"%NSSM_EXE%" stop GestionBoxAPI >nul 2>&1
"%NSSM_EXE%" remove GestionBoxAPI confirm >nul 2>&1

:verificar_servicio
echo Esperando a que el sistema libere los recursos del servicio...
ping -n 3 127.0.0.1 >nul

sc query GestionBoxAPI >nul 2>&1
if not errorlevel 1 echo Advertencia: El servicio aún figura en el sistema. Cierra services.msc si está abierto para completar la eliminación

:servicio_no_existe

:: Eliminar reglas de firewall
echo Eliminando reglas de firewall...
netsh advfirewall firewall delete rule name="GestionBox HTTP" >nul 2>&1
netsh advfirewall firewall delete rule name="GestionBox UDP" >nul 2>&1

:: Eliminar únicamente los ejecutables gestion-box-api.exe y nssm.exe
echo Eliminando ejecutables del servidor...
set "INTENTOS_DEL=0"

:bucle_borrar_archivos
set /a INTENTOS_DEL+=1
if exist "%DESTINO%\gestion-box-api.exe" del /f /q "%DESTINO%\gestion-box-api.exe" >nul 2>&1
if exist "%DESTINO%\nssm.exe" del /f /q "%DESTINO%\nssm.exe" >nul 2>&1

if not exist "%DESTINO%\gestion-box-api.exe" if not exist "%DESTINO%\nssm.exe" goto archivos_borrados
if %INTENTOS_DEL% geq 5 goto aviso_archivos_en_uso
ping -n 3 127.0.0.1 >nul
goto bucle_borrar_archivos

:aviso_archivos_en_uso
echo Advertencia: No se pudieron borrar los ejecutables porque siguen bloqueados
echo Puedes eliminarlos manualmente tras reiniciar el equipo
goto finalizar_desinstalacion

:archivos_borrados
echo Ejecutables eliminados correctamente

:finalizar_desinstalacion
echo Desinstalación completada
echo Los datos de la base de datos, respaldos y logs se conservan en %DESTINO%
pause
exit /b 0

:: Controladores de error

:error_ruta_unc
echo Error: No se puede ejecutar desde una ruta de red
echo Copia la carpeta al escritorio e intenta nuevamente
pause
exit /b 1

:error_permisos_admin
echo Error: Se requieren permisos de administrador
echo Haz clic derecho en desinstalar.bat y selecciona Ejecutar como administrador
pause
exit /b 1
