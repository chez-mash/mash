# Usa Mash con tu agente

Conecta Claude Code, Codex o GitHub Copilot CLI para pedirle que busque
trabajos anteriores. Un agente conectado puede buscar en todo el historial
que hayas indexado. Los resultados que lea pasan a formar parte de su
contexto. [Agentes compatibles](supported-agents.md) · [Cómo trata Mash tu historial](privacy.md)

Si instalaste el complemento desde el catálogo de tu agente, [termina la configuración en el agente](plugin.md#finish-setup-in-the-agent).
Para otro agente compatible con servidores MCP locales, [configura la conexión manualmente](../mcp.md#connect-another-mcp-client).

<span id="connect-claude-code-codex-or-copilot"></span>

## Conecta un agente después de configurar Mash

Si es tu primera vez, sigue la [guía de inicio rápido](quick-start.md) y
elige los agentes que quieras conectar cuando se te pregunte. Una vez que
puedas buscar tus conversaciones, puedes añadir una conexión sin volver a indexar.

### En el navegador

1. En **settings**, elige **Connect an agent**. Si el enlace de tu versión
   se llama **go through setup**, elige ese.
2. Conserva los historiales y el intervalo de fechas. Abre **Choose agents to connect**
   y elige los agentes que quieras. Algunas versiones muestran esas opciones en
   **Customize setup** o después de elegir **Next** en las pantallas de historial.
   Desmarca los agentes que no quieras configurar.
3. Elige **Set up without indexing**. Inicia una sesión nueva en el agente
   y [prueba una búsqueda](#try-it).

Mash busca el comando del agente en el equipo donde se ejecuta el panel. Si
acabas de instalar el agente o de añadirlo a `PATH`, reinicia el panel desde
una terminal donde funcione ese comando. Si el panel es de solo lectura,
sigue los pasos de terminal en ese equipo.

### En la terminal

Abre el asistente con la indexación desactivada.

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

Las versiones de desarrollo incluyen la configuración guiada de GitHub Copilot CLI.
Con v0.1.1-alpha, [conecta Copilot CLI manualmente](plugin.md#github-copilot-cli).

<span id="what-the-connection-installs"></span>

### Qué instala la conexión

Mash utiliza la CLI de cada agente elegido (`claude`, `codex` o `copilot`)
para instalar dos componentes:

- Un **servidor MCP** que pone las herramientas de búsqueda de Mash a
  disposición del agente. MCP (Model Context Protocol) permite a los agentes
  utilizar herramientas externas.
- El **[complemento de Mash](plugin.md)**, con la función `mash-history`
  que indica al agente cuándo y cómo buscar, la función `remember` que guarda
  algo en la [memoria](records.md) cuando se lo pides (`/mash:remember` en
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

Inicia una sesión nueva en el agente que conectaste. Pídele que use Mash
para buscar una conversación que ya encuentres en el panel. Si el agente no
ve Mash, comprueba qué está registrado en ese equipo.

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
```

Usa `--agent copilot` en las versiones que ofrecen esa opción. Con v0.1.1-alpha,
[comprueba la conexión desde Copilot CLI](plugin.md#github-copilot-cli).

La prueba comprueba una conexión nueva. Para que los cambios se apliquen
a tus conversaciones, reinicia el agente después de configurar, reparar o
actualizar Mash.
Si la prueba indica que la configuración del proyecto puede modificar la
conexión, ejecútala fuera del proyecto para verificar el registro de usuario.
Comprueba las conexiones del proyecto desde el propio agente.

<span id="keep-new-chats-searchable"></span>

## Actualiza el historial de conversaciones

El complemento indexa las novedades en segundo plano al iniciar una sesión
del agente, como máximo una vez cada diez minutos. Para desactivar esta tarea,
`MASH_AUTO_INDEX=off` en el entorno del agente.

Ejecuta `mash index` para actualizar ahora, o deja `mash index --watch` en
otra terminal para actualizar de forma continua.
[Más sobre la actualización del historial](synchronization.md)

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
[cómo desinstalarlo](uninstall.md). Si vas a seguir usando
Mash, quita el agente de [`[agents] connected`](../configuration.md#connected-agents)
para que Mash deje de comprobar esa conexión.

## Otros clientes MCP y configuración manual

Cualquier cliente que admita servidores MCP por entrada y salida estándar
puede ejecutar `mash mcp`. Sigue el
[ejemplo de configuración de MCP](../mcp.md#connect-another-mcp-client)
e incluye las variables de entorno que necesites.

[Solución de problemas de conexión](troubleshooting.md#my-agent-cannot-find-mash)

<!-- mash-docs-source: 9eefd5390c868ddde79cb2de929c06967a270e9fc40f875d4b1e1587129f6907 -->
