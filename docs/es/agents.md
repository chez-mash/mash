# Usa Mash con tu agente

Conecta Claude Code, Codex o GitHub Copilot CLI para pedirle que busque
trabajos anteriores. Un agente conectado puede buscar en todo el historial
que hayas indexado. Los resultados que lea pasan a formar parte de su
contexto. [Agentes compatibles](supported-agents.md) · [Cómo trata Mash tu historial](privacy.md)

<span id="connect-claude-code-codex-or-copilot"></span>

## Conecta un agente después de configurar Mash

Si es tu primera vez, sigue la [guía de inicio rápido](quick-start.md) y
elige los agentes que quieras conectar cuando se te pregunte. Si ya puedes
buscar tus conversaciones en Mash, ejecuta este comando en una terminal
para añadir una conexión sin volver a indexar:

```bash
mash init --no-index
```

1. Elige **Use existing settings** para conservar tu selección de historiales.
2. En **Which agents should get access to Mash search?**, usa la barra
   espaciadora para dejar seleccionados solo los agentes que quieras conectar.
   Si se te pregunta si quieres crear recuerdos a partir de conversaciones
   anteriores, puedes rechazar esa opción.
3. Elige **Set up without indexing** y reinicia el agente para que cargue
   la conexión. [Prueba una búsqueda](#try-it).

Si tu agente no aparece, comprueba si puedes ejecutar su comando en esa
terminal, por ejemplo, `codex --version`. Instala la CLI del agente o añádela
a `PATH` y vuelve a ejecutar el comando anterior.

La configuración guiada de GitHub Copilot CLI está disponible en la rama
de desarrollo. Si tu versión solo ofrece Claude Code y Codex, puedes
[configurar MCP manualmente](../mcp.md#connect-another-mcp-client).

Si ya instalaste el complemento desde el catálogo de tu agente, usa su
[función `setup`](../plugin.md#start-with-the-plugin) para terminar la configuración.

### Qué instala la conexión

Mash utiliza la CLI de cada agente elegido (`claude`, `codex` o `copilot`)
para instalar dos componentes:

- Un **servidor MCP** que pone las herramientas de búsqueda de Mash a
  disposición del agente. MCP (Model Context Protocol) permite a los agentes
  utilizar herramientas externas.
- El **[complemento de Mash](../plugin.md)**, con la función `mash-history`
  que indica al agente cuándo y cómo buscar, la función `remember` que guarda
  algo en la [memoria](../records.md) cuando se lo pides (`/mash:remember` en
  Claude Code) y las acciones automáticas que
  [mantienen el historial al día](#keep-new-chats-searchable).

Los comandos actualizan la configuración de usuario del agente
(`~/.claude.json`, `~/.claude/settings.json` y `~/.claude/plugins` para
Claude Code, `~/.codex/config.toml` y `~/.codex/plugins` para Codex, y
`~/.copilot/mcp-config.json`, `~/.copilot/config.json` y
`~/.copilot/installed-plugins` para GitHub Copilot CLI). Mash también anota
qué agentes están conectados en su propio `config.toml` para que
[`mash status --installation`](#confirm-the-connection) pueda avisarte si
alguno pierde la conexión.

El complemento requiere acceso a Internet para instalarse. Sus acciones
automáticas necesitan Python 3.9 o superior (`python3`, `python` o el
lanzador `py` en Windows).

Si el complemento no se instala, el agente aún puede usar el servidor MCP
para buscar cuando se lo pidas. Las funciones del complemento y las acciones
automáticas no estarán disponibles.

Vuelve a ejecutar `mash init` para reparar una conexión que apunta a un
ejecutable de Mash que falta o que no es el correcto.

<span id="try-it"></span>

## Pídele a tu agente que encuentre trabajos anteriores

- «Usa Mash para buscar qué decidimos sobre la autenticación».
- «Encuentra la conversación donde hablamos del despliegue de este repositorio».
- «¿En qué trabajamos la semana pasada? Muéstrame los mensajes originales».
- «Encuentra la conversación anterior sobre este error».

El agente busca, lee los mensajes que encontró y cita el título, la fecha
o un enlace a la conversación. Si quieres ver los identificadores de sesión
o de mensaje, pídeselos.

Para invocar la función directamente, usa `/mash:mash-history` en Claude Code.
En Codex o Copilot, selecciona la función `mash-history` del complemento o
pídele al agente que use Mash.

<span id="confirm-the-connection"></span>

## Comprueba la conexión del agente

```bash
mash status --installation
```

En el informe JSON, `summary.clients` indica el estado `connection` de
cada agente.

| `connection` | Qué significa |
| --- | --- |
| `full` | La conexión funciona. |
| `mcp-only` | Las herramientas de Mash están conectadas. Revisa el complemento, la función o las acciones automáticas. |
| `none` | Mash no tiene una conexión utilizable con el agente. Puede que no lo seleccionaras durante la configuración. |

Si hay algo que reparar, `next_action` indica cómo hacerlo. Volver a indexar
no repara la conexión. Si el complemento del catálogo es más nuevo que tu
CLI, el informe comprueba si ambos declaran la misma interfaz de
herramientas. Puedes seguir usando herramientas compatibles si los archivos
del complemento están intactos. Si la interfaz cambia, necesitas una
versión de la CLI que coincida.

Un recuento `unconfigured` con `next_action: null` indica que Mash encontró
datos de agentes sin conexión y no sugiere ninguna reparación. Puedes buscar
en el historial indexado desde el panel web o la terminal.

El comando muestra el informe y sale con código 1 si requiere atención
un agente conectado por `mash init`, configurado para Mash o seleccionado con
`--agent`. Puedes dejar sin conectar los demás agentes.

Para comprobar un agente concreto e iniciar su servidor MCP en un proceso
nuevo, elige el comando correspondiente:

```bash
mash status --installation --agent codex --probe
mash status --installation --agent claude-code --probe
mash status --installation --agent copilot --probe
```

La prueba comprueba una conexión nueva. Para que los cambios se apliquen
a tus conversaciones, reinicia el agente después de configurar, reparar o
actualizar Mash.

<span id="keep-new-chats-searchable"></span>

## Actualiza el historial de conversaciones

El complemento indexa las novedades en segundo plano al iniciar una sesión
del agente, como máximo una vez cada diez minutos. Para desactivar esta tarea,
`MASH_AUTO_INDEX=off` en el entorno del agente.

Ejecuta `mash index` para actualizar ahora, o deja `mash index --watch` en
otra terminal para actualizar de forma continua.
[Más sobre la actualización del historial](../synchronization.md)

<span id="disconnect-an-agent"></span>

## Quita la conexión de un agente

Usa los comandos correspondientes a tu agente. `mash init` los muestra
en el campo `undo`.

```bash
claude mcp remove --scope user mash
claude plugin uninstall mash@mash
codex mcp remove mash
codex plugin remove mash@mash
copilot mcp remove mash
copilot plugin uninstall mash@mash
```

Si quieres quitar también Mash del equipo, consulta
[cómo desinstalarlo](installation.md#uninstall). Si vas a seguir usando
Mash, quita el agente de [`[agents] connected`](../configuration.md#connected-agents)
para que Mash deje de comprobar esa conexión.

## Otros clientes MCP y configuración manual

Cualquier cliente que admita servidores MCP por entrada y salida estándar
puede ejecutar `mash mcp`. Sigue el
[ejemplo de configuración de MCP](../mcp.md#connect-another-mcp-client)
e incluye las variables de entorno que necesites.

[Solución de problemas de conexión](troubleshooting.md#my-agent-cannot-find-mash)

<!-- mash-docs-source: d890419b26152a4e78be6e843a0c4789881de03d39205a6b214101d802a2e9a2 -->
