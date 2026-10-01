# 08 — Pruebas de aceptación

Estas pruebas se hacen **en el CESFAM, en los PCs reales**, antes de firmar el acta.
Se necesitan al menos el PC servidor y dos PCs cliente (A y B).

| # | Prueba | Cómo ejecutarla | Resultado esperado | Cubre | ✔ |
|---|---|---|---|---|---|
| 1 | Reiniciar el PC servidor sin iniciar sesión | Reiniciar el servidor y dejarlo en la pantalla de inicio de sesión. Cronometrar desde que aparece esa pantalla. | La app de otro PC se conecta en menos de 2 minutos | RNF-01 | ☐ |
| 2 | Reserva simultánea | En A y B abrir el modal del mismo box y bloque, escribir nombres, contar "3, 2, 1" y presionar Reservar a la vez | Solo una reserva se guarda y el otro ve "Ese bloque acaba de ser tomado" | RF-05 | ☐ |
| 3 | Refresco automático | Reservar en A sin tocar B | B la ve sin tocar nada en menos de 30 segundos | RF-07 | ☐ |
| 4 | Cambio de IP | Cambiar la IP del servidor, o editar `cliente.json` de A con una IP incorrecta, y abrir la app | La app encuentra el servidor sola (menos de 5 s) | RF-09, RNF-08 | ☐ |
| 5 | Servidor apagado | Con la app abierta en A, apagar el servidor | Aparece "Sin conexión" con un mensaje claro de qué revisar | RF-08, RNF-09 | ☐ |
| 6 | Cerrar la app en el servidor | Cerrar la app en el PC servidor; reservar desde A | Los demás PCs siguen funcionando | RNF-01 | ☐ |
| 7 | Matar la API | Administrador de tareas → finalizar `gestion-box-api.exe`; esperar 10 s | NSSM la reinicia sola; A sigue funcionando | RNF-01 | ☐ |
| 8 | Respaldos | Reiniciar el servicio varias veces; revisar `C:\ProgramData\GestionBox\backups` | Hay un respaldo por arranque, máximo 30 | RF-11 | ☐ |
| 9 | Uso sin capacitación | Pedir a un funcionario, sin explicarle nada, que reserve y cancele | Lo logra sin ayuda | RNF-11 | ☐ |

Para la prueba 8 se puede acelerar con un script que reinicie el servicio 35 veces:

```bat
for /L %%i in (1,1,35) do (
  C:\ProgramData\GestionBox\nssm.exe restart GestionBoxAPI
  timeout /t 3 >nul
)
dir C:\ProgramData\GestionBox\backups
```

## Registro

| Fecha | Ejecutó | Pruebas aprobadas | Observaciones |
|---|---|---|---|
| | | /9 | |

Todas deben aprobarse para firmar el acta de entrega (T-5.5).
