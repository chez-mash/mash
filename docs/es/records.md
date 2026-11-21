# Usa la memoria de Mash

Mash puede guardar decisiones, preferencias o datos con una cita de la
conversación en la que surgieron. Puedes abrir esa cita para ver el contexto.

<span id="inspect-existing-records"></span>

## Consulta los recuerdos guardados

[Abre una decisión guardada en la demo](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
y elige **open the message** debajo de la cita. La demo abre el mensaje
sobre guardar el token de actualización en el llavero del dispositivo.

Para ver tus recuerdos, ejecuta `mash dash`, abre la dirección que aparece
y elige **memory**. Abre un registro para ver su origen, revisiones y
registros relacionados.
[Busca y filtra recuerdos en el panel](dashboard.md#inspect-conversation-records)

<span id="save-a-conclusion"></span>

## Guarda una conclusión

Con el [complemento de Mash instalado](plugin.md#start-with-the-plugin),
pídele al agente en la misma conversación que guarde la decisión para futuras sesiones.

> Recuerda que este proyecto guarda los tokens de actualización en el llavero del dispositivo.

En Claude Code también puedes usar `/mash:remember`. En Codex o Copilot,
pide al agente que use la habilidad `remember`. Comprueba qué guardó en la
respuesta y abre **memory** para leer la cita en su contexto.

<span id="recall-records"></span>

## Recupera recuerdos

En otra conversación del mismo proyecto, pregunta «¿Qué recuerda Mash
sobre los tokens de actualización? Muéstrame dónde lo decidimos».
En Claude Code también puedes usar `/mash:recall refresh tokens`.

El complemento puede mostrar al agente algunos recuerdos al iniciar una
sesión o al leer un archivo. El agente también puede buscar en conversaciones
anteriores para encontrar decisiones que Mash aún no guardó en la memoria.
[Usa Mash desde un agente](agents.md#try-it)

<span id="how-saving-works"></span>

## Cómo guarda Mash los recuerdos

Mash coteja la cita con la conversación original para que puedas comprobar
el contexto. El agente puede pasar algo por alto o interpretar mal lo que dijiste.

- El agente puede dar por confirmadas decisiones y preferencias basadas en
  tus propias palabras. Otras interpretaciones pueden quedar como
  **candidatas** pendientes de revisión.
- El agente puede guardar decisiones propias. Las decisiones que tú tomas tienen prioridad.
- Si una conversación indexada cambia y la cita ya no coincide, Mash marca
  la fuente como desactualizada y la omite al recuperar recuerdos normalmente.

El guardado utiliza el agente que ya está en marcha. El complemento puede
detectar conclusiones mientras trabajas y recordar al agente que guarde las
que se le hayan pasado. En Codex, revisa los cambios de los hooks del
complemento en `/hooks` para habilitarlos.
[Qué hace el complemento sin pedírselo](plugin.md#what-happens-without-asking)

Usa `MASH_RECORD_CAPTURE=off` para desactivar la captura automática de recuerdos
y seguir consultándolos. Usa `MASH_RECORD_RECALL=off` para desactivar los
recuerdos automáticos y mantener las sugerencias de conversaciones.
[Ajustes del complemento](plugin.md#what-happens-without-asking)

<span id="correct-or-replace"></span>

## Corrige o sustituye un recuerdo

Si un recuerdo es incorrecto o has cambiado de opinión, pide al agente que
lo actualice usando la conversación en la que lo corregiste. Usa la habilidad
`review`, o `/mash:review` en Claude Code, para revisar candidatos,
fuentes desactualizadas y decisiones en conflicto. Consultar un recuerdo
deja su estado como está.

También puedes pedir al agente que olvide un recuerdo. Esto borra sus
revisiones y citas. Si [eliminas una conversación citada](privacy.md#remove-a-chat),
Mash también elimina los recuerdos que dependen de ella.

<span id="backfill-earlier-memory"></span>

## Extrae recuerdos de conversaciones anteriores

Esta revisión opcional puede proponer recuerdos a partir de conversaciones
anteriores. Utiliza tu sesión iniciada de Codex o Claude Code, envía el texto
seleccionado al servicio de ese agente y consume parte de su presupuesto de tokens.
La indexación normal y `mash init --yes` dejan esta revisión desactivada.

Puedes solicitarla durante la configuración interactiva para las
conversaciones activas en los últimos tres días. Para revisar luego
conversaciones que ya estén indexadas, ejecuta:

```sh
mash init --yes --no-index --no-agents --backfill-memory codex
```

Puedes sustituir `codex` por `claude-code`. Comprueba los recuerdos sugeridos
antes de usarlos. Mash se detiene tras cien conversaciones o si no consigue
terminar una revisión. Dispone de hasta diez minutos por conversación.
Puedes volver a ejecutar el comando para continuar. Mash omite las
revisiones ya completas salvo que la conversación haya cambiado.

### Recuerdos compartidos

Si publicas el historial en un [servidor Mash compartido](shared-index.md),
puedes leer los recuerdos publicados en el panel. Los recuerdos vinculados a
la ruta de un proyecto local permanecen en ese equipo.
[Consulta la fuente de un recuerdo compartido](dashboard.md#inspect-conversation-records)

## Haz una copia de los recuerdos

La [copia de seguridad portátil](backup.md#make-a-portable-backup) incluye
tus recuerdos, sus revisiones y el texto citado. Trátala con la misma
privacidad que las conversaciones originales.

## Llama directamente a las herramientas

Consulta los argumentos y ejemplos en las
[herramientas MCP para recuerdos (en inglés)](../record-tools.md).

<!-- mash-docs-source: 36885e948f7746e5e39820ec03eac164d61f7874ac71b5ddbaf6c01efb02afdc -->
