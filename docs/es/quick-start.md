# Guía de inicio rápido

<span id="quick-start"></span>

Mash busca en las conversaciones que tus agentes de programación ya guardaron
en este equipo. Es de [código abierto](https://github.com/mashium/mash) y se
ejecuta en tu propio equipo. Antes de instalarlo, puedes
[probar una búsqueda de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue)
y abrir un resultado para ver la conversación original.

Puedes buscar en el panel web o en la terminal, o conectar un agente para que
busque por ti.

[Qué cambia al configurar Mash](installation.md#what-setup-changes) · [Cómo verificar la descarga](downloads.md#verify-the-checksum) · [Privacidad del historial](privacy.md) · [Quick start in English](../../README.md#quick-start)

Si quieres empezar con un solo agente y los últimos siete días, sigue la
[guía de prueba local](privacy.md#prueba-con-menos-historial).

## Pide ayuda a tu agente

Pega esta instrucción en Claude Code, Codex u otro agente de programación.

> Lee https://getmash.io/setup.md y configura Mash en este equipo. Revisa
> conmigo las opciones en español antes de empezar.

Puedes empezar con el panel y elegir si quieres conectar agentes.

Si ya instalaste el [plugin de Mash](plugin.md#start-with-the-plugin),
pídele a tu agente que use la skill `setup`.

<span id="install-it-yourself"></span>

## Instálalo por tu cuenta

Necesitas al menos una conversación guardada con un [agente compatible](supported-agents.md).
Los comandos de abajo instalan Mash, inician la búsqueda local y ponen en
marcha el panel para que configures Mash en tu navegador. Si prefieres Homebrew,
apt, Scoop u otro gestor de paquetes, consulta los
[comandos de instalación](install-methods.md#install-with-a-package-manager).
Si prefieres el [asistente de terminal](installation.md#set-up-in-a-terminal), usa sus pasos de configuración.

En un terminal de macOS con Apple silicon o de Linux (x86-64 o ARM64), ejecuta
este comando para una instalación local nueva. En Linux o WSL necesitas una
sesión de usuario de systemd para iniciar Qdrant.
[Comprueba tu sesión o consulta las alternativas](local-search.md#linux-without-a-systemd-user-session).

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --dashboard
```

En Windows x64, los datos de búsqueda se guardan en una carpeta con una ruta
corta en la unidad del sistema, normalmente `C:\mash-qdrant\storage`.
[Dónde guarda Mash los datos](privacy.md#where-mash-stores-data). Usa PowerShell.

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Dashboard
```

El instalador verifica las descargas, guarda Mash en `.local/bin`, inicia el
servicio de búsqueda local (Qdrant, unos 30 MB).
El servicio funciona con tu usuario y se inicia cuando entras en tu sesión.
La búsqueda por palabras funciona sin descargar modelos.

Abre en tu navegador la dirección que muestra el instalador, normalmente
<http://127.0.0.1:8790>, y deja el terminal abierto. Revisa los historiales
detectados y el período. El período predeterminado es de 90 días. Si primero
aparece la lista de historiales, pulsa **Next** hasta llegar al resumen.

- **Busca por tu cuenta en el panel.** Elige **Skip agent connections** o desactiva cada conexión en la pantalla de agentes. Puedes conectar agentes más adelante.
- **Deja que un agente busque por ti.** Selecciona los agentes que quieras conectar y comprueba cuáles figuran en el resumen.

Elige **Set up and index**. [Pasos de configuración en el panel](dashboard.md#set-up-mash-from-the-dashboard)

Para instalar por separado o usar tu propio Qdrant, consulta la
[guía de instalación](installation.md).

<span id="try-it"></span>

## Encuentra una conversación anterior

Cuando la búsqueda esté lista, entra en **chats**. Escribe una palabra, un
código de error o el nombre de una función que recuerdes y pulsa Intro.
Abre un resultado para ver el mensaje original en su conversación.

Para volver a abrir el panel, ejecuta `mash dash` y abre la dirección que muestra.
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

Para que los chats nuevos aparezcan en tus búsquedas, ejecuta `mash index` o
[mantén el índice al día en segundo plano](synchronization.md).
También puedes [hacer una copia de seguridad](backup.md).

<!-- mash-docs-source: d399b6683cab2014c1981e29b038db64aec33965aa8cc955be372b7e16785d31 -->
