# Guía de inicio rápido

Mash busca en las conversaciones que tus agentes de programación ya guardaron
en este equipo. Es de [código abierto](https://github.com/mashium/mash) y se
ejecuta en tu propio equipo. Antes de instalarlo, puedes
[probar una búsqueda de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue)
y abrir un resultado para ver la conversación original.

Después de configurar Mash, ejecuta `mash dash` y abre en tu navegador la
dirección que muestra. También puedes buscar en la terminal o conectar un
agente para que busque por ti.

[Qué cambia al configurar Mash](installation.md#what-setup-changes) · [Cómo verificar la descarga](installation.md#check-the-release-download) · [Privacidad del historial](privacy.md) · [Quick start in English](../../README.md#quick-start)

Si quieres empezar con un solo agente y los últimos siete días, sigue la
[guía de prueba local](privacy.md#prueba-con-menos-historial).

## Pide ayuda a tu agente

Pega esta instrucción en Claude Code, Codex u otro agente de programación.

> Lee https://getmash.io/setup.md y configura Mash en este equipo. Revisa
> conmigo las opciones en español antes de empezar.

Puedes empezar con el panel y elegir si quieres conectar agentes.

Si ya instalaste el [plugin de Mash](plugin.md#start-with-the-plugin),
pídele a tu agente que use la skill `setup`.

## Instálalo por tu cuenta

Necesitas al menos una conversación guardada con un [agente compatible](supported-agents.md).
El instalador incluye el panel web. Si prefieres Homebrew, apt, Scoop u otro
gestor de paquetes, consulta los [comandos de instalación](install-methods.md#install-with-a-package-manager).

En un terminal de macOS con Apple silicon o de Linux (x86-64 o ARM64), ejecuta
este comando para una instalación local nueva. En Linux o WSL necesitas una
sesión de usuario de systemd para iniciar Qdrant.
[Comprueba tu sesión o consulta las alternativas](local-search.md#linux-without-a-systemd-user-session).

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

En Windows x64, los datos de búsqueda se guardan en una carpeta con una ruta
corta en la unidad del sistema, normalmente `C:\mash-qdrant\storage`.
[Dónde guarda Mash los datos](privacy.md#where-mash-stores-data). Usa PowerShell.

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

El instalador verifica las descargas, guarda Mash en `.local/bin`, inicia el
servicio de búsqueda local (Qdrant, unos 30 MB) y abre el asistente de Mash.
El servicio funciona con tu usuario y se inicia cuando entras en tu sesión.
La búsqueda por palabras funciona sin descargar modelos.

Revisa los historiales antes de empezar. Por defecto se incluyen los archivos
actualizados en los últimos 90 días. Si el asistente pregunta dónde buscar,
elige **Just this computer (recommended)**.

- **Busca por tu cuenta en el panel.** [Omite las conexiones a agentes](installation.md#skip-agent-connections) e indexa tus conversaciones. Puedes conectar agentes más adelante.
- **Deja que un agente busque por ti.** Comprueba qué agentes aparecen en la revisión antes de elegir **Set up and index**.

Para instalar por separado o usar tu propio Qdrant, consulta la
[guía de instalación](installation.md).
También puedes [usar el asistente del panel local](dashboard.md#set-up-mash-from-the-dashboard).

## Encuentra una conversación anterior

Cuando el asistente indique que la búsqueda está lista, inicia el panel local
y deja el terminal abierto.

```sh
mash dash
```

1. Abre en el navegador la dirección que muestra el comando, normalmente [http://127.0.0.1:8790](http://127.0.0.1:8790).
2. Entra en **chats**, escribe una palabra, un código de error o el nombre de una función que recuerdes y pulsa Intro.
3. Abre un resultado para ver el mensaje original en su conversación.

Si el terminal no encuentra `mash`, usa la ruta completa que muestra el instalador.
También puedes [añadir Mash a la ruta de comandos](troubleshooting.md#mash-command-not-found).
Si no aparece una conversación, consulta [qué hacer cuando no hay resultados](troubleshooting.md#search-returns-no-results).
Consulta la [guía del panel web](dashboard.md#find-and-read-a-chat) o
[busca desde el terminal](search.md#your-first-search).

**Busca desde un agente.** Reinicia los agentes que conectaste. Por ejemplo,
puedes pedirle a Codex «Usa Mash para buscar qué decidimos sobre la
autenticación en Claude Code y muéstrame el mensaje original».
[Comprueba la conexión](agents.md#confirm-the-connection) si el agente
no encuentra Mash. Si omitiste este paso durante la configuración, puedes
[conectar un agente más tarde](agents.md#connect-claude-code-codex-or-copilot).

Los menús de Mash están en inglés. Para buscar palabras de tus conversaciones,
consulta [cómo buscar en otros idiomas](search-languages.md).
