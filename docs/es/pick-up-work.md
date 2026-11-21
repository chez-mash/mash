# Retoma el trabajo donde lo dejaste

Abre un chat anterior e identifica el qué puedes hacer a continuación. En la demo, abre la decisión guardada sobre el inicio de sesión y selecciona **open the message**. El chat muestra la decisión de usar tokens de acceso de corta duración y luego indica qué trabajo queda pendiente en la versión móvil. [Pruébalo con chats de ejemplo](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)

<span id="find-the-next-step-in-your-chats"></span>

## Encuentra el siguiente paso en tus chats

1. Abre **chats** en el panel. Busca un nombre de archivo, un error o una frase relacionada con el trabajo. Usa el filtro de proyectos si hay varios proyectos con chats parecidos.
2. Abre un resultado y lee los últimos mensajes. Fíjate en qué se terminó, qué decisiones se acordaron y qué pruebas o tareas quedaron pendientes. Despliega los resultados de las herramientas que sean pertinentes para ver qué pasó.
3. Da el siguiente paso por tu cuenta o lleva un breve resumen a una nueva sesión del agente. Incluye el objetivo, la decisión que sigue vigente y lo que queda por hacer.

Para encontrar el chat adecuado, consulta [la búsqueda en el panel](dashboard.md#find-and-read-a-chat)
y [qué revisar si no encuentras un chat](troubleshooting.md#search-returns-no-results).

<span id="reopen-the-original-session"></span>

## Vuelve a abrir la sesión original

En uno de tus chats de Codex o Claude Code, selecciona **Copy session ID** en la transcripción. En el equipo donde está guardada la sesión original, abre una terminal en el directorio del proyecto y sustituye `SESSION_ID` en el comando de tu agente.

```sh
codex resume SESSION_ID
claude --resume SESSION_ID
```

Si el agente no puede abrir la sesión con ese ID, busca el chat por su título en el selector de sesiones o inicia una sesión nueva con un resumen.

<span id="ask-an-agent-for-a-recap"></span>

## Pide un resumen a un agente

Una vez configurados el complemento de Mash y las herramientas de búsqueda, inicia una sesión en el proyecto que quieras retomar. Ejecuta `/mash:resume` en Claude Code. En Codex o Copilot, pide «Usa la habilidad `resume` de Mash para decirme dónde nos quedamos en este proyecto».
[Configura Mash en tu agente](plugin.md#finish-setup-in-the-agent)

La habilidad intenta actualizar el índice, luego comprueba si alguien dejó notas de traspaso y lee los chats recientes del proyecto. Resume qué pasó, qué queda pendiente y cuál es el siguiente paso. Pídele que te muestre el mensaje original en el que se tomó cualquier decisión que quieras comprobar.

<span id="leave-a-handoff-for-the-next-session"></span>

## Deja un resumen de traspaso para la próxima sesión

1. Antes de terminar una sesión, ejecuta `/mash:handoff` en Claude Code o pídele a tu agente que use la habilidad `handoff` de Mash. La habilidad escribe un breve resumen en el chat actual. Comprueba que indique los archivos o la rama, las decisiones tomadas, el trabajo pendiente y el siguiente paso.
2. Cuando el agente termine de escribir el resumen, ejecuta `mash index` en el equipo donde está el chat. En ese mismo equipo, la habilidad `resume` también puede actualizar el índice cuando vuelvas.
3. Abre el proyecto en tu próxima sesión y pídele al agente que use `resume`. También puedes buscar `"Mash handoff"` en el panel y leer el resumen por tu cuenta.

Si cambias de equipo, configura primero el uso compartido y ejecuta la indexación en el equipo donde se escribió el resumen para que pueda publicar el chat. Para hacer el traspaso manualmente, copia el resumen en la nueva sesión. Comparte solo los detalles que quieras que lea el próximo agente. [Busca en todos tus equipos](shared-index.md) · [Qué puede ver un agente](privacy.md#when-history-can-leave-your-machine)

<!-- mash-docs-source: 6248ed759dbb495e792ddf0cdb080fb54a4a6d188e6948c31407a5aed576c670 -->
