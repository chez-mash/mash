# Guía de inicio rápido

<span id="quick-start"></span>

<span id="install-it-yourself"></span>

## Instálalo por tu cuenta

Ejecuta el comando de tu sistema para descargar Mash y empezar a configurarlo
en el navegador. Necesitas al menos una conversación guardada con un
[agente compatible](supported-agents.md). Consulta
[qué cambia al configurar Mash](installation.md#what-setup-changes) y
[cómo verificar la descarga](downloads.md#verify-the-checksum).

**macOS (Apple silicon) o Linux (x86-64 o ARM64)**

En Linux o WSL necesitas una [sesión de usuario de systemd](local-search.md#linux-without-a-systemd-user-session).

```sh
curl -fsSL https://raw.githubusercontent.com/chez-mash/mash/main/install.sh | sh -s -- --dashboard
```

**Windows x64 (PowerShell)**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/chez-mash/mash/main/install.ps1))) -Dashboard
```

Abre el enlace que te muestra el instalador y deja el terminal abierto.
Sigue los pasos en el navegador y elige **Set up and index**. Puedes omitir
la conexión de agentes o desmarcar los que aparezcan seleccionados.
[Pasos de configuración en el panel](dashboard.md#set-up-mash-from-the-dashboard)

<span id="try-it"></span>
<span id="find-your-first-chat"></span>

## Encuentra una conversación anterior

Cuando la búsqueda esté lista, entra en **chats**. Escribe una palabra, un
código de error o el nombre de una función que recuerdes y pulsa Intro.
Abre un resultado para ver el mensaje original en su conversación.

Para volver a abrir el panel, ejecuta `mash dash` y abre la dirección que muestra.
Si el terminal no encuentra `mash`, usa la ruta completa que mostró el instalador
o [añade Mash a la ruta de comandos](troubleshooting.md#mash-command-not-found).
[Busca y lee una conversación en el panel](dashboard.md#find-and-read-a-chat) ·
[Busca desde el terminal](search.md#your-first-search) ·
[Qué hacer cuando no hay resultados](troubleshooting.md#search-returns-no-results)

<span id="ask-your-agent"></span>

## Pide ayuda a tu agente

Pega esta instrucción en Claude Code, Codex u otro agente de programación.

> Lee https://getmash.io/setup.md y configura Mash en este equipo. Revisa
> conmigo las opciones en español antes de empezar.

Si ya instalaste el [plugin de Mash](plugin.md#start-with-the-plugin),
pídele a tu agente que use la skill `setup`. Si instalaste Mash por tu cuenta,
puedes [conectar un agente más tarde](agents.md#connect-claude-code-codex-or-copilot).

Para usar gestores de paquetes, compilar desde el código fuente o configurar
Mash en el terminal, consulta la [guía de instalación](installation.md).
Para empezar con menos historial, sigue la
[guía de prueba local](privacy.md#prueba-con-menos-historial).

<!-- mash-docs-source: d57d2515612c6e32065f147541df1dc3c18acd5e701eed2c2fd739454faeaf4a -->
