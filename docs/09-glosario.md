# 09 — Glosario

| Término | Significado |
|---|---|
| **Box** | Sala o consultorio de atención del CESFAM. En el código: `BoxAtencion` (tabla `box`). |
| **Bloque** | Intervalo fijo de 30 minutos. Índice 0 = 08:00–08:30, 23 = 19:30–20:00. |
| **Reserva** | Asignación de un box a un funcionario en una fecha y bloque. |
| **Funcionario** | Persona del CESFAM que reserva. Solo se guarda su nombre. |
| **PC servidor** | El PC con Windows donde corre la API y vive la base de datos. |
| **API** | Programa `gestion-box-api.exe` que guarda y entrega los datos por la red. |
| **App** | Programa de escritorio "Gestión de Boxes" que usan los funcionarios. |
| **NSSM** | *Non-Sucking Service Manager*: herramienta que convierte un `.exe` en servicio de Windows y lo reinicia si se cae. |
| **Servicio de Windows** | Programa que arranca con Windows sin que nadie inicie sesión. |
| **Broadcast UDP** | Mensaje enviado a todos los equipos de la subred; se usa para encontrar al servidor. |
| **Descubrimiento** | Proceso por el cual la app encuentra la dirección del servidor sin configurarla a mano. |
| **`/health`** | Dirección de la API que responde "ok" si está funcionando. |
| **Respaldo** | Copia completa de la base de datos tomada cada vez que la API arranca. |
| **SQLite** | Base de datos contenida en un solo archivo (`gestion-box.db`). |
| **UNIQUE** | Regla de la base de datos que impide dos reservas del mismo box, fecha y bloque. |
| **409 Conflict** | Respuesta de la API cuando una reserva choca con otra existente. |
| **CORS** | Permiso del navegador/webview para llamar a la API desde otra "origen". |
| **Tauri** | Framework para apps de escritorio con interfaz web y backend en Rust. |
| **Acta de entrega** | Documento firmado con el CESFAM que cierra el proyecto. |
