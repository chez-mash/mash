# El complemento de Mash para Claude Code, Codex y Copilot

El complemento de Mash permite que Claude Code, Codex o GitHub Copilot CLI consulten trabajos anteriores mientras conversas, guarden una decisión cuando se lo pidas y abran el panel de Mash. Añade habilidades y acciones automáticas (hooks). El [servidor MCP](../mcp.md) de Mash proporciona al agente las herramientas de búsqueda que usan esas habilidades. También puedes usar MCP por separado y pedirle al agente que busque con Mash.

* **¿Ya usas el panel o la terminal de Mash?** [Conecta tu agente](agents.md#connect-claude-code-codex-or-copilot)
  sin volver a indexar. La configuración instala el complemento en los agentes que conecta.
* **¿Ya instalaste el complemento desde el catálogo de tu agente?** [Termina la configuración en el agente](#finish-setup-in-the-agent).
* **¿Empiezas de cero?** Sigue la [guía de inicio rápido](quick-start.md), o bien [instala primero el complemento](#start-with-the-plugin) y deja que el agente te guíe durante la configuración.

<span id="install-update-and-remove"></span>

<span id="install-the-plugin"></span>

## Instala el complemento

<span id="requirements"></span>

La instalación requiere acceso a internet. Los hooks requieren Python 3.9 o posterior (`python3`, `python` o el lanzador `py` de Windows).

<span id="start-with-the-plugin"></span>

### Empieza por el complemento

Ejecuta los dos comandos de terminal para tu agente. Si ya añadiste el catálogo de Mash, usa el segundo comando.

#### Claude Code

```sh
claude plugin marketplace add mashium/mash
claude plugin install mash@mash
```

#### Codex

```sh
codex plugin marketplace add mashium/mash
codex plugin add mash@mash
```

<span id="install-in-github-copilot-cli"></span>

#### Instala el complemento en GitHub Copilot CLI

```sh
copilot plugin marketplace add mashium/mash
copilot plugin install mash@mash
```

<span id="finish-setup-in-the-agent"></span>

### Termina la configuración en el agente

Inicia una sesión nueva. En Claude Code, ejecuta `/mash:setup`. En Codex o Copilot, di «Usa la habilidad `setup` de Mash para configurar Mash en este equipo». El agente revisa contigo qué se instalará y qué chats se indexarán, y luego te muestra cómo abrir el panel. Conectar agentes es opcional. Si decides conectar Copilot CLI y usas Mash v0.1.1-alpha, [completa la conexión de Copilot](#github-copilot-cli)
después de la configuración.

Si conectaste un agente, reinícialo y pídele «Usa Mash para encontrar un chat anterior sobre autenticación. Muéstrame el mensaje en el que se basa la respuesta». Elige un tema que ya hayas tratado. Para activar los hooks de Codex, abre `/hooks` en Codex, revisa los hooks de Mash y márcalos como confiables. [Comprueba la conexión](agents.md#confirm-the-connection)
si el agente no encuentra Mash.

<span id="skills-and-commands"></span>

## Habilidades y comandos

| Habilidad | En Claude Code | En Codex | Qué hace |
| --- | --- | --- | --- |
| `setup` | `/mash:setup` | elige la habilidad `setup` o pide «Configura Mash en este equipo» | Instala Mash con tu aprobación, conecta tu historial y comprueba que puedes encontrar una conversación anterior. |
| `remember` | `/mash:remember what to keep` | elige la habilidad `remember` o di «recuerda esto» | Guarda una decisión, preferencia o dato en la [memoria](../records.md), usando tus propias palabras como respaldo. Si no añades nada al comando, guarda la decisión más reciente de la conversación. |
| `resume` | `/mash:resume` | elige la habilidad `resume` o pregunta «¿en qué nos quedamos?» | Lee los chats más recientes del proyecto de todos los agentes y equipos, y te dice qué estaba en curso, qué quedó pendiente y qué se decidió. |
| `handoff` | `/mash:handoff` | elige la habilidad `handoff` o di «deja esto listo para que otra persona lo retome» | Escribe en la conversación un resumen de la sesión con el objetivo, lo que se terminó, lo que quedó a medias, las decisiones y el siguiente paso. `resume` lo encuentra desde cualquier agente o equipo una vez indexado el chat. |
| `recall` | `/mash:recall a topic` | elige la habilidad `recall` o pregunta «¿qué recuerda Mash sobre…?» | Muestra los registros de [memoria](../records.md) sobre un tema o sobre este proyecto, con el origen de cada uno. Te avisa cuando hay registros que requieren revisión. |
| `review` | `/mash:review` | elige la habilidad `review` o di «revisa la memoria de Mash» | Revisa unos pocos registros de memoria a la vez que requieren una decisión tuya. Incluye los que guardó un agente sin tu confirmación, los que han quedado obsoletos y los pares que se contradicen. Puedes conservar, corregir, retirar, eliminar u omitir cada uno. |
| `forget` | `/mash:forget what to remove` | elige la habilidad `forget` o pide explícitamente que olvide o elimine algo que Mash guardó | Encuentra un registro de memoria o un chat indexado, te muestra exactamente qué se eliminaría y solo lo elimina si das tu aprobación. Si el registro contiene un error, te ofrece la opción de corregirlo. |
| `standup` | `/mash:standup`, o bien `/mash:standup this week` | elige la habilidad `standup` o pregunta «¿en qué trabajé ayer?» | Resume tu trabajo durante un período, agrupado por proyecto, en todos tus agentes y equipos. Si no añades nada al comando, abarca el día laborable anterior. |
| `export` | `/mash:export`, o bien `/mash:export a chat` | elige la habilidad `export` o di «exporta este chat» | Abre el [panel](dashboard.md#save-a-transcript) directamente en el chat, donde el menú Export permite guardarlo como Markdown, HTML o PDF. Si se lo pides, el agente puede escribir directamente un archivo Markdown. |
| `status` | `/mash:status` | elige la habilidad `status` o pregunta «¿funciona Mash?» | Comprueba el índice, la búsqueda, las conexiones de los agentes y tus otros equipos. Explica el resultado en lenguaje sencillo e indica el siguiente paso si detecta algún problema. |
| `dashboard` | `/mash:dashboard` | elige la habilidad `dashboard` o di «abre el panel de Mash» | Localiza el [panel](dashboard.md) que ya está en ejecución o inicia uno, y lo abre en tu navegador. Pide «este chat» para ir directamente a la conversación actual. |
| `mash-history` | `/mash:mash-history`, o simplemente pídelo | elige la habilidad `mash-history` o simplemente pídelo | Busca en tus chats anteriores y responde con los mensajes en los que se basa el resultado. El agente también usa esta habilidad por iniciativa propia cuando mencionas trabajos anteriores. |

GitHub Copilot CLI carga las mismas habilidades. Puedes indicar el nombre de la habilidad o pedir lo que necesitas con tus propias palabras.

Para el uso diario puedes pedir lo que necesites con tus propias palabras. Por ejemplo, puedes hacer estas consultas.

* «¿Qué decidimos sobre la migración de la base de datos?»
* «Encuentra el chat en el que configuramos el despliegue».
* «Recuerda que solo desplegamos desde `main`».
* «Abre el panel de Mash».
* «¿En qué nos quedamos con este proyecto?»
* «¿Qué recuerda Mash sobre el despliegue?»
* «Ese registro de memoria sobre la URL del entorno de preproducción es incorrecto. Corrígelo».

Después de guardar algo, el agente te confirma qué guardó. Rechaza las solicitudes para guardar credenciales o tokens. La eliminación es permanente. `forget` gestiona un solo registro o chat a la vez, siempre te lo muestra y te pide confirmación antes de eliminarlo.

<span id="what-happens-without-asking"></span>

## Qué ocurre automáticamente

| Cuándo | Qué hace el complemento | Cómo desactivarlo |
| --- | --- | --- |
| Se inicia una sesión | Muestra al agente los registros guardados y los chats recientes de este proyecto, y ejecuta `mash index` en segundo plano como máximo una vez cada diez minutos | `MASH_AUTO_INDEX=off` detiene la indexación automática del complemento. `MASH_RECORD_RECALL=off` oculta los registros guardados en los avisos de sesión y de archivo. Los avisos sobre chats siguen apareciendo. |
| Envías un mensaje que alude a trabajos anteriores o menciona algo que aún no ha aparecido en la conversación | Le recuerda al agente que busque en Mash antes de hacer suposiciones | `MASH_PROMPT_RECALL=off` |
| Expresas una decisión o preferencia que quieres mantener | Le recuerda al agente cómo guardarla. Si la sesión se prolonga sin guardar nada, recibe como máximo un recordatorio cada quince minutos | `MASH_RECORD_CAPTURE=off` |
| El agente lee un archivo del proyecto | Muestra registros y chats anteriores sobre ese archivo una vez por archivo y por sesión | `MASH_FILE_RECALL=off` |

En Copilot CLI, el aviso sobre un archivo llega junto con el contenido que leyó el agente, justo después de la lectura.

Define la variable en el entorno desde el que se inicia el agente. Si Mash no está instalado, aún no se ha configurado o tarda en responder, los hooks se omiten en silencio para que el agente pueda seguir trabajando. Los hooks ejecutan Mash en tu equipo. Los fragmentos que muestran al iniciar la sesión o al leer archivos pasan al contexto del agente, que puede enviar esos fragmentos a su proveedor de modelos. Los hooks activados pueden ejecutarse sin conexión MCP. Si solo quieres usar el panel o la terminal, [elimina cualquier complemento de Mash que hayas instalado](#update-or-remove-the-plugin)
y [desconecta el agente](agents.md#disconnect-an-agent).
[Cuándo puede salir el historial de tu equipo](privacy.md#when-history-can-leave-your-machine)

<span id="update-or-remove-the-plugin"></span>

## Actualiza o elimina el complemento

El complemento y el programa `mash` se publican juntos. Un catálogo puede ofrecer una versión más reciente del complemento antes de que se publique el binario correspondiente. Ejecuta
`mash status --installation` para ver qué componente necesita actualizarse. Una versión más reciente del complemento puede funcionar con el binario actual si la interfaz de las herramientas es compatible y se superan las demás comprobaciones de instalación. Inicia una sesión nueva después de actualizar. Para eliminar el complemento, ejecuta el comando de tu agente.

```sh
claude plugin uninstall mash@mash
codex plugin remove mash@mash
copilot plugin uninstall mash@mash
```

Para retirar también las herramientas de búsqueda, [desconecta el agente](agents.md#disconnect-an-agent).

## GitHub Copilot CLI

Las versiones de desarrollo pueden conectar GitHub Copilot CLI desde `mash init` cuando
`copilot` está en tu `PATH`. La versión v0.1.1-alpha puede leer los chats guardados de Copilot, y puedes darle a Copilot acceso manual a la búsqueda de Mash. Después de [configurar Mash](quick-start.md), registra sus herramientas desde una terminal.

```sh
copilot mcp add mash -- mash mcp
```

Si Copilot no encuentra `mash`, sustituye el ejecutable que aparece después de `--` por la ruta completa a tu instalación, entre comillas. Si quieres, [instala el complemento de Copilot](#install-in-github-copilot-cli)
para disponer de las habilidades y los hooks. Abre una sesión nueva de Copilot y pídele que use Mash para encontrar un chat anterior. `copilot mcp list` muestra `mash`, y `copilot plugin list`
muestra `mash@mash` si instalaste el complemento.

Con la versión v0.1.1-alpha, elimina la conexión de Copilot antes de desinstalar Mash. Si instalaste el complemento, ejecuta también el comando para eliminarlo.

```sh
copilot plugin uninstall mash@mash
copilot mcp remove mash
```

* Copilot guarda esta configuración en `~/.copilot` (o en el directorio indicado por `COPILOT_HOME`). Guarda las herramientas en `mcp-config.json` y el complemento instalado en
  `config.json` y `installed-plugins`.
* `mash init --backfill-memory` no admite Copilot. Revisar chats antiguos de esa forma consumiría tus solicitudes prémium, así que la opción solo se ofrece para Claude Code y Codex.
* Para guardar en la [memoria](../records.md) desde una sesión de Copilot, necesitas una versión de `mash` capaz de leer la sesión activa de Copilot. `mash --version-info` muestra `copilot`
  en `session_citations` si admite esa función. Con una versión anterior de `mash` el complemento puede recuperar recuerdos, aunque omite la solicitud de guardado a Copilot.
* En Windows, los hooks de Copilot se ejecutan mediante PowerShell y llaman a `python`. Aún no hay constancia de que los hooks se hayan verificado en Windows.

<span id="check-that-it-is-working"></span>

## Comprueba que funciona

```sh
mash status --installation
```

El campo `connection` de cada agente indica `full` cuando están instalados tanto las herramientas como el complemento, y `mcp-only` cuando solo están conectadas las herramientas. `next_action` indica el siguiente paso cuando una integración requiere atención. Para Copilot con la versión v0.1.1-alpha, usa [las comprobaciones de Copilot](#github-copilot-cli).
[Usa Mash con tu agente](agents.md#confirm-the-connection)

<!-- mash-docs-source: 1e02f9b9b161736cf7cca776c5a49c6df0689c3d5dab9868d0a3291d2d334be9 -->
