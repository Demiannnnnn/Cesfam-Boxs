# 01 — Visión general

## El problema

Hoy el CESFAM reserva sus boxes de atención en una planilla Excel compartida. Una planilla
compartida tiene problemas conocidos: dos personas editan la misma celda, alguien deja el
archivo abierto y bloquea al resto, se borra una fila sin querer y nadie sabe quién fue.

## La solución

Una aplicación de escritorio muy simple, instalada en todos los PCs de la red local, que
muestra **una sola pantalla**: una grilla del día con los boxes en columnas y los bloques de
30 minutos en filas. Una celda libre se reserva con un clic; una celda ocupada muestra quién
la reservó y permite cancelarla.

Los datos viven en **un solo PC** (el "servidor"), dentro de un pequeño programa (la API) que
arranca solo con Windows. Si dos personas intentan tomar el mismo bloque al mismo tiempo, la
base de datos acepta a la primera y a la segunda le avisa: *"Ese bloque acaba de ser tomado"*.

## Lo que queda fuera (a propósito)

| Fuera de alcance | Por qué |
|---|---|
| Usuarios, contraseñas, roles | Los datos no son sensibles y el acceso es solo desde la red interna. Menos piezas = menos fallas. |
| Datos de pacientes | No se necesitan para reservar un box y evitan obligaciones de protección de datos. |
| Acceso desde fuera del CESFAM | No es necesario y abriría riesgos de seguridad. |
| Notificaciones | La grilla se refresca sola cada 15–30 s. |
| Mantenimiento futuro | No hay presupuesto ni persona asignada; el sistema debe sobrevivir solo. |

## Supuestos

- Hasta **3 usuarios simultáneos**; la base de datos se mantiene **bajo 100 MB**.
- Todos los PCs están en la **misma red local y subred**.
- Hay **permisos de administrador** para instalar.
- Hay un **PC con Windows** designado como servidor. Puede apagarse o reiniciarse (actualizaciones,
  cortes de luz, fin de jornada); el sistema vuelve solo al encenderlo, sin que nadie inicie sesión.
- Los datos guardados son: box, fecha, bloque, nombre del funcionario y motivo.

## Principio rector

> **El sistema debe funcionar sin nadie que lo mantenga después de la entrega.**

Cada decisión técnica se evalúa contra esta frase: ¿arranca solo?, ¿se recupera solo?,
¿explica en español qué revisar cuando falla?, ¿alguien sin conocimientos técnicos puede
reinstalarlo siguiendo una guía de una página?
