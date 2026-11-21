# Guía de inicio rápido

Mash busca en las conversaciones que tus agentes de programación ya guardaron
en este equipo. Es de [código abierto](https://github.com/mashium/mash) y se
ejecuta en tu propio equipo. Antes de instalarlo, puedes
[probar una búsqueda de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue)
y abrir un resultado para ver la conversación original.

Después de configurar Mash, ejecuta `mash dash` y abre en tu navegador la
dirección que muestra. También puedes buscar en la terminal o conectar un
agente para que busque por ti.

[Privacidad del historial](privacy.md) · [Revisa y verifica el instalador](installation.md#check-the-release-download) · [Quick start in English](../../README.md#quick-start)

Si quieres empezar con un solo agente y los últimos siete días, sigue la
[guía de prueba local](privacy.md#prueba-con-menos-historial).

## Pide ayuda a tu agente

Pega esta instrucción en Claude Code, Codex u otro agente de programación.

> Lee https://getmash.io/setup.md y configura Mash en este equipo. Explícame
> en español qué va a cambiar antes de empezar.

Puedes empezar con el panel y elegir si quieres conectar agentes.

Si ya instalaste el [plugin de Mash](../plugin.md#start-with-the-plugin),
pídele a tu agente que use la skill `setup`.

## Instálalo por tu cuenta

Necesitas al menos una conversación guardada con un agente compatible.

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

Revisa las carpetas de historial y los agentes detectados. Elige
**Set up and index** para crear el índice en este equipo. La configuración
predeterminada incluye archivos actualizados en los últimos 90 días.
Si el asistente te pregunta antes dónde buscar, elige **Just this computer**.
Para elegir carpetas, fechas o agentes, usa **Customize setup** si está
disponible.

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
También puedes [añadir Mash a la ruta de comandos (en inglés)](../troubleshooting.md#mash-command-not-found).
Si no aparece una conversación, consulta [qué hacer cuando no hay resultados (en inglés)](../troubleshooting.md#search-returns-no-results).
Consulta la [guía del panel web](dashboard.md#find-and-read-a-chat) o
[busca desde el terminal (en inglés)](../search.md#your-first-search).

**Busca desde un agente.** Reinicia los agentes que conectaste. Por ejemplo,
puedes pedirle a Codex «Usa Mash para buscar qué decidimos sobre la
autenticación en Claude Code y muéstrame el mensaje original».
[Comprueba la conexión](agents.md#confirm-the-connection) si el agente
no encuentra Mash. Si omitiste este paso durante la configuración, puedes
[conectar un agente más tarde](agents.md#connect-claude-code-codex-or-copilot).

Los menús de Mash están en inglés. Para buscar palabras de tus conversaciones,
consulta [cómo buscar en otros idiomas](search-languages.md).
