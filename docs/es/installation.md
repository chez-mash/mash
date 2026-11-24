# Instala Mash

Instala Mash, inicia el servicio de búsqueda y elige qué conversaciones
quieres buscar. Con la configuración predeterminada, el índice y las
búsquedas se quedan en tu equipo. [Cómo trata Mash tu historial](privacy.md)

Puedes [buscar en conversaciones de ejemplo en la demo](https://getmash.io/demo/).
Para probarlo con tus propios datos, [elige un historial y usa el panel local](privacy.md#prueba-con-menos-historial).

La [guía de inicio rápido](quick-start.md) incluye un solo comando que
instala Mash e inicia el servicio de búsqueda y el panel para configurar Mash
en el navegador. Si ya instalaste Mash, [inicia el servicio de búsqueda](#start-the-local-search-service).
Si ya está en marcha, [configura tus historiales](#connect-your-histories).

Para pasar a una versión nueva, sigue los [pasos de actualización](upgrades.md#upgrade-an-existing-installation).

<span id="what-setup-changes"></span>

## Qué cambia al configurar Mash

1. **Instala los programas.** El [instalador básico](#recommended-release-installers)
   coloca `mash` y `mash-server` en tu carpeta de usuario. También puedes
   [verificar y descomprimir un archivo de la versión](downloads.md).
2. **Inicia la búsqueda local.** `mash qdrant install` descarga Qdrant y lo
   configura para que se ejecute con tu cuenta al iniciar sesión. Mash solo
   permite conexiones desde este equipo y desactiva la telemetría de Qdrant.
   [Archivos del servicio de búsqueda](local-search.md#where-qdrant-stores-data)
3. **Elige qué puede leer Mash.** La configuración en el panel o en la terminal
   muestra los historiales que encontró Mash antes de guardar la
   configuración o indexar tus conversaciones. Mash lee las conversaciones
   originales donde están y, por defecto, guarda una copia que puedes buscar
   en este equipo. Si eliges conectar un agente, el asistente actualiza su
   configuración para que pueda pedir fragmentos de esa copia. También puedes
   [omitir las conexiones a agentes](#skip-agent-connections) y buscar por tu cuenta.
   [Dónde guarda Mash tus datos](privacy.md#where-mash-stores-data)

El comando de inicio rápido instala los programas e inicia Qdrant antes de que
abras el panel. Si omites la configuración, `mash uninstall` muestra qué se
instaló y cómo retirarlo. [Vista previa de la desinstalación](uninstall.md)

<span id="install-the-cli"></span>

<span id="get-mash-for-your-system"></span>

## Descarga Mash para tu sistema

La versión precompilada incluye el panel web. Cuando el servicio de búsqueda
esté en marcha, puedes [configurar Mash en el navegador](#set-up-in-your-browser).
También puedes [conectar tus agentes](agents.md) para que busquen en tus
conversaciones por ti.

<span id="recommended-release-installers"></span>

### Instaladores recomendados

Hay versiones precompiladas para Linux x86-64 y ARM64, macOS con Apple
silicon y Windows x64. Elige el comando para tu sistema. Puedes
[revisar el instalador o verificar una descarga](downloads.md) antes.

En Linux hace falta glibc 2.34 o posterior, disponible en Ubuntu 22.04,
Debian 12, RHEL 9 y versiones posteriores. Alpine y otros sistemas con
musl quedan fuera de las plataformas compatibles.

<span id="macos-or-linux"></span>

#### macOS o Linux

```sh
curl -fsSL https://raw.githubusercontent.com/grep-stack/mash/main/install.sh | sh
```

Usa el mismo comando en WSL.

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/grep-stack/mash/main/install.ps1 | iex
```

Ambos scripts comprueban la suma de verificación e instalan Mash en
`.local/bin` dentro de tu carpeta personal. Muestran los siguientes comandos
con la ruta completa, así que puedes usarlos antes de añadir el directorio
a `PATH`. Usa `MASH_VERSION` para fijar una etiqueta publicada o
`MASH_INSTALL_DIR` para elegir otro directorio. Los scripts dejan intacto el
perfil de tu terminal. Si el comando `mash` no aparece, consulta
[cómo encontrarlo](troubleshooting.md#mash-command-not-found).

<span id="package-manager-alternatives"></span>

### Instala con un gestor de paquetes

Puedes usar [Homebrew, apt, Scoop, mise o cargo-binstall](install-methods.md#install-with-a-package-manager).
Después de instalar Mash, [inicia el servicio de búsqueda](#start-the-local-search-service).

<span id="build-from-source"></span>

### Compila a partir del código fuente

Para buscar desde el navegador, [compila Mash con el panel web](install-methods.md#include-the-web-dashboard).
Si quieres [buscar en la terminal y usar las herramientas de agente](install-methods.md#cli-and-agent-tools),
usa el comando de Cargo más corto.

<span id="start-the-local-search-service"></span>

## Inicia el servicio de búsqueda local

En Linux o WSL, el comando de abajo requiere una sesión de usuario de systemd.
[Comprueba tu sesión y consulta las alternativas](local-search.md#linux-without-a-systemd-user-session).
Si ya tienes un [Qdrant local compatible](local-search.md#run-qdrant-yourself),
puedes pasar a [configurar tus historiales](#connect-your-histories).

En Windows, Mash guarda los datos de búsqueda de Qdrant en un directorio con
una ruta corta en la unidad del sistema, normalmente `C:\mash-qdrant\storage`.
[Archivos y registros de Qdrant](local-search.md#where-qdrant-stores-data)

```sh
mash qdrant install
```

Mash verifica la descarga de Qdrant (unos 30 MB) e inicia el servicio en
segundo plano con tu usuario. Qdrant escucha en el equipo local y vuelve a
iniciarse al entrar en tu sesión. En Windows requiere iniciar sesión en el
escritorio. Si el servicio no arranca, consulta
[cómo revisar los registros y reiniciarlo](local-search.md#check-or-restart-the-service).

<span id="run-qdrant-yourself"></span>

### Ejecuta Qdrant por tu cuenta

Puedes [reutilizar un Qdrant 1.20 o posterior en tu equipo](local-search.md#run-qdrant-yourself).
Para iniciar uno, elige la guía para tu entorno.

- <span id="optional-qdrant-docker-image"></span>[Qdrant en Docker](local-search.md#optional-qdrant-docker-image)
- <span id="native-windows-without-docker"></span>[Windows sin Docker](local-search.md#native-windows-without-docker)
- <span id="native-macos-or-linux-without-docker"></span>[macOS o Linux sin Docker](local-search.md#native-macos-or-linux-without-docker)

<span id="connect-your-histories"></span>

## Configura tus historiales

Cuando el servicio de búsqueda local esté en marcha, puedes elegir qué
conversaciones lee Mash en el panel del navegador o en la terminal.

<span id="set-up-in-your-browser"></span>

### Configura Mash en el navegador

Si instalaste una versión publicada o [compilaste con el panel](install-methods.md#include-the-web-dashboard),
ejecuta este comando en una terminal:

```sh
mash dash
```

Abre el enlace que te muestra Mash y deja el terminal abierto. En una
instalación nueva, sigue los pasos en el navegador y elige **Set up and index**.
Puedes omitir la conexión de agentes o desmarcar los que aparezcan seleccionados.
Cuando la búsqueda esté lista, abre **chats** y busca una palabra que recuerdes.
[Consulta los pasos de configuración en el panel](dashboard.md#set-up-mash-from-the-dashboard)

<span id="set-up-in-a-terminal"></span>

### Configura Mash en la terminal

Para instalar Mash y abrir el asistente de terminal con un solo comando,
usa el [script de inicio rápido](quick-start.md#install-it-yourself) con
`--setup` en vez de `--dashboard` en macOS o Linux, o con `-Setup` en vez de
`-Dashboard` en Windows.

Usa el asistente de la terminal si compilaste Mash sin el panel, si quieres
elegir una carpeta de historial o si vas a conectarte a un servidor compartido.

```sh
mash init
```

En una instalación nueva, el asistente de la terminal muestra las carpetas
que encontró, el periodo que va a leer (archivos actualizados en los últimos
90 días) y los agentes que puede conectar. Si el asistente pregunta dónde
buscar, elige **Just this computer (recommended)** para guardar el índice en
este equipo. **Set up and index** aplica las opciones de la revisión, incluidas
las conexiones a los agentes seleccionados. Mash puede leer conversaciones
guardadas aunque falte el comando del agente.

Elige **Customize setup** para cambiar el periodo o la carpeta de un
historial, elegir integraciones concretas, conectar un servidor compartido
existente o guardar la configuración sin indexar. Esta opción también
permite [extraer memoria de conversaciones anteriores](records.md#backfill-earlier-memory)
después de indexar. Para ello, Mash usa tu sesión de Claude Code o Codex y
envía texto seleccionado de tus conversaciones al servicio de modelos del
agente.

Pulsa Esc o Ctrl-C para salir antes de cambiar la configuración o los ajustes
de tus agentes. Si vas a indexar, el asistente comprueba la conexión HTTP con
el servicio de búsqueda local y, si elegiste compartir, la autenticación
con el servidor. Si falla una conexión, puedes guardar la configuración y
terminar más adelante.

Si conectaste un agente, reinícialo para que cargue Mash. Los hooks del plugin
de historial necesitan Python 3.9 o posterior. [Usa Mash con tu agente](agents.md)

<span id="skip-agent-connections"></span>

### Omite las conexiones a agentes

Puedes buscar tus conversaciones en el panel o la terminal sin dar acceso a
la búsqueda de Mash a un agente de programación. Sigue la opción que muestre
el asistente.

- Si la primera revisión ofrece **Set up and index without connecting agents**,
  elígela para indexar los historiales que aparecen.
- Si el asistente pregunta **Which agents should get access to Mash search?**,
  usa la barra espaciadora para desmarcar todos los agentes y pulsa Intro. Si
  pregunta si quieres extraer recuerdos de conversaciones anteriores, elige que no.
- Si la revisión indica que no se conectará ningún agente, elige **Set up and index**.

Cuando la búsqueda esté lista, [abre el panel](#try-your-first-search). Puedes
[conectar un agente más adelante](agents.md#connect-claude-code-codex-or-copilot).

<span id="try-your-first-search"></span>

## Haz tu primera búsqueda

Si instalaste el panel, ejecuta `mash dash` y abre la dirección que muestra.
En **chats**, busca una palabra que recuerdes de una conversación anterior y
abre un resultado para leer el mensaje original.

También puedes [buscar en la terminal](search.md#your-first-search)
o [pedirle a un agente conectado que busque](agents.md#try-it). Si falta una
conversación antigua, aprende a [incluir más historial](indexing.md#add-older-history).

La búsqueda por palabras funciona sin modelos. Para buscar por significado,
sigue los [pasos para instalar los modelos opcionales](indexing.md#enable-semantic-search).

<span id="setup-options"></span>

## Opciones de configuración

<span id="without-the-wizard"></span>

### Sin el asistente

`mash init --yes` acepta las opciones detectadas. Incluye los archivos
modificados en los últimos 90 días y las sesiones actuales de todos los
agentes compatibles que encontró, incluido Copilot. Aplica las integraciones
disponibles y omite la extracción de memoria de conversaciones anteriores.
Cuando la entrada o la salida de errores no está conectada a una terminal,
el comando también se ejecuta sin interacción. El resultado JSON enumera las
carpetas seleccionadas.

Sin el asistente, `mash init` comprueba Qdrant antes de cambiar nada. Si no
responde, muestra los comandos para iniciarlo y conserva la configuración y
los ajustes de los agentes.

| Opción | Efecto |
| --- | --- |
| `--no-index` | Guarda la configuración sin indexar. Omite la comprobación de Qdrant e incluye cómo iniciarlo en los pasos siguientes. Para elegir carpetas a mano, edita la [configuración (en inglés)](../configuration.md) y ejecuta `mash index`. |
| `--no-agents` | Busca en el panel o la terminal sin conectar agentes. |
| `--force` | Sin esta opción, la configuración no interactiva conserva los ajustes existentes. Con `--yes`, vuelve a generar la configuración a partir de los historiales detectados. En el asistente, abre directamente la revisión. |

Sin `--force`, el asistente te deja conservar los ajustes existentes o
revisarlos. La revisión conserva los ajustes que no aparecen en el asistente
y guarda una copia del archivo anterior.

Para leer el historial antes de iniciar Qdrant, ejecuta `mash init --no-index`
y luego `mash index`. Mash guarda los chats en este equipo antes de necesitar
Qdrant. Al detenerse, indica cuántos guardó y cómo iniciar el servicio.
Mientras tanto, puedes leerlos con `mash list-chats` y `mash view`. Cuando
Qdrant esté en marcha, ejecuta `mash index` otra vez para activar la búsqueda.

<span id="read-the-result"></span>

### Lee el resultado

El asistente de la terminal termina con un informe breve y el siguiente paso.
Cuando la búsqueda esté lista, verás cómo abrir el panel o buscar en la
terminal, según la versión instalada. Para obtener el informe JSON completo,
ejecuta `mash init --yes` o redirige la salida estándar del asistente con
`mash init > setup.json`. Las preguntas y el progreso van a la salida de
errores.

- `"agents": []` indica que Mash no conectó ningún agente aunque la indexación
  haya terminado. Consulta `next` para ver el motivo y sigue los pasos para
  [conectar un agente](agents.md). Puedes buscar desde la terminal por tu cuenta.
- Para cada agente conectado, `files` enumera los archivos que su comando
  editó y `undo` indica cómo quitar la conexión.
- `index_error` indica que la primera indexación falló después de aplicar
  la configuración. Puede ocurrir si se detiene Qdrant o si su puerto gRPC
  es inaccesible. El comando informa del error y conserva la configuración,
  las conexiones y el historial que ya guardó. Corrige el problema y
  ejecuta `mash index`.
- `written: false` en una segunda ejecución indica que se reutilizó la
  configuración existente.
- El recuento de mensajes que se pueden buscar solo incluye el diálogo
  entre la persona y su agente. El razonamiento y la salida de herramientas
  se pueden leer en cada conversación, por lo que el recuento de búsqueda
  puede ser mucho menor.

<span id="check-the-release-download"></span>

## Verifica la descarga

[Elige un archivo y comprueba su suma](downloads.md) o
[revisa el instalador antes de ejecutarlo](downloads.md#inspect-the-installer).

<span id="uninstall"></span>

## Desinstala Mash

Ejecuta `mash uninstall` para ver qué quitaría Mash. [Consulta qué se conserva y cómo desinstalar Mash](uninstall.md).

<!-- mash-docs-source: ee4a2a27f23c403ecd60868d3802d5825da71d2845b9b858c965efd527ca32f3 -->
