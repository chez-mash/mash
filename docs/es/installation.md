# Instala Mash

Instala Mash, inicia el servicio de búsqueda y elige qué conversaciones
quieres buscar. Con la configuración predeterminada, el índice y las
búsquedas se quedan en tu equipo. [Cómo trata Mash tu historial](privacy.md)

Puedes [buscar en conversaciones de ejemplo en la demo](https://getmash.io/demo/).
Para probarlo con tus propios datos, [elige un historial y usa el panel local](privacy.md#prueba-con-menos-historial).

La [guía de inicio rápido](quick-start.md) incluye un solo comando que
instala Mash, inicia el servicio de búsqueda y abre el asistente para
revisar la configuración. Si ya instalaste Mash, [inicia el servicio de búsqueda](#start-the-local-search-service).
Si ya está en marcha, [configura tus historiales](#connect-your-histories).

Para pasar a una versión nueva, sigue los [pasos de actualización (en inglés)](../upgrades.md#upgrade-an-existing-installation).

<span id="install-the-cli"></span>

<span id="get-mash-for-your-system"></span>

## Descarga Mash para tu sistema

La versión precompilada incluye el panel web. Una vez configurado Mash,
ejecuta `mash dash` y abre la dirección que muestra. Durante la configuración
también puedes [conectar tus agentes](agents.md) para que busquen en tus
conversaciones por ti.

<span id="recommended-release-installers"></span>

### Instaladores recomendados

Hay versiones precompiladas para Linux x86-64 y ARM64, macOS con Apple
silicon y Windows x64. Elige el comando para tu sistema. Puedes
[revisar el instalador o verificar una descarga](#check-the-release-download) antes.

En Linux hace falta glibc 2.34 o posterior, disponible en Ubuntu 22.04,
Debian 12, RHEL 9 y versiones posteriores. Alpine y otros sistemas con
musl quedan fuera de las plataformas compatibles.

<span id="macos-or-linux"></span>

#### macOS o Linux

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

Usa el mismo comando en WSL.

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
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

Puedes usar [Homebrew, apt, Scoop, mise o cargo-binstall (en inglés)](../install-methods.md#install-with-a-package-manager).
Después de instalar Mash, [inicia el servicio de búsqueda](#start-the-local-search-service).

<span id="build-from-source"></span>

### Compila a partir del código fuente

Para buscar desde el navegador, [compila Mash con el panel web (en inglés)](../install-methods.md#include-the-web-dashboard).
Si quieres [buscar en la terminal y usar las herramientas de agente (en inglés)](../install-methods.md#cli-and-agent-tools),
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
permite [extraer memoria de conversaciones anteriores (en inglés)](../records.md#backfill-earlier-memory)
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

También puedes [buscar en la terminal (en inglés)](../search.md#your-first-search)
o [pedirle a un agente conectado que busque](agents.md#try-it). Si falta una
conversación antigua, aprende a [incluir más historial (en inglés)](../indexing.md#add-older-history).

La búsqueda por palabras funciona sin modelos. Para buscar por significado,
sigue los [pasos para instalar los modelos opcionales (en inglés)](../indexing.md#enable-semantic-search).

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

Antes de ejecutarlo, puedes leer el [instalador de macOS y Linux](https://github.com/mashium/mash/blob/main/install.sh)
o el [instalador de Windows](https://github.com/mashium/mash/blob/main/install.ps1).
Los scripts instalan los archivos en tu carpeta de usuario sin permisos
de administrador. Con la instalación en un solo comando, el script instala
e inicia Qdrant antes de abrir el asistente. Allí puedes revisar los
historiales y las conexiones antes de aplicarlos. Si cancelas en el
asistente, usa la [vista previa de desinstalación](#uninstall) para ver qué
llegó a instalarse.

El [proceso público de publicación](https://github.com/mashium/mash/actions/workflows/release.yml)
crea los archivos de cada plataforma después de verificar que las pruebas
de Linux y Windows pasaron para esa versión del código. Las
[pruebas públicas de instalación](https://github.com/mashium/mash/actions/workflows/release-check.yml)
indican qué instaladores pasaron para tu versión.

Para instalar sin ejecutar un script descargado, obtén el archivo de tu
plataforma y `SHA256SUMS` de la misma
[versión publicada en GitHub](https://github.com/mashium/mash/releases).
Antes de extraerlo, compara su suma SHA-256 con la línea que lleva su nombre
exacto. En Linux x86-64 puedes usar este comando.

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

En macOS usa `shasum -a 256` con el archivo de macOS. En Windows usa
`Get-FileHash -Algorithm SHA256` con el ZIP de Windows en PowerShell.
Si la suma coincide con la publicada, extrae `mash` y `mash-server` (o los
dos archivos `.exe`) a un directorio tuyo. Después
[inicia el servicio de búsqueda](#start-the-local-search-service).

Si la versión incluye una certificación de compilación, GitHub CLI también
puede verificar el proceso y la etiqueta de origen del archivo.
Sustituye `ARCHIVE` por el archivo descargado y `RELEASE_TAG` por su etiqueta,
incluida la `v` inicial.

```sh
gh attestation verify ARCHIVE --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

El proceso de publicación comprueba esas certificaciones antes de publicar
los archivos. Hasta `v0.1.1-alpha`, usa la comprobación SHA-256 de arriba.

<span id="uninstall"></span>

## Desinstala Mash

```sh
mash uninstall          # lists what would be removed and changes nothing
mash uninstall --yes    # removes it
```

Lee la vista previa en la terminal. Enumera los servicios, las conexiones y
los archivos locales que se quitarían, incluido el tamaño de la copia de tus
conversaciones. Si rediriges la salida o usas una versión anterior que
muestra JSON en la terminal, consulta `paths`, `kept` y `next`.

Con `--yes`, Mash hace lo siguiente.

1. Detiene y quita los servicios que configuró, incluidos el servicio de
   búsqueda y sus datos, el [panel remoto](dashboard.md) y el servicio que
   actualiza el índice si está instalado.
2. Desconecta Claude Code, Codex y GitHub Copilot con los comandos de cada
   agente descritos en [cómo desconectar un agente](agents.md#disconnect-an-agent).
3. Borra su índice (la copia de tus conversaciones), los modelos, la
   configuración y los tokens guardados.

Sigue la última instrucción del informe para quitar el programa. En JSON
aparece en `binary.remove`. Puede indicar `brew uninstall mash`,
`scoop uninstall mash`, `sudo apt remove mash-cli`, `cargo uninstall mash`
o los archivos que debes borrar si usaste el script.

Los historiales originales de tus agentes y los datos publicados en un
[índice compartido](shared-index.md) permanecen donde estaban.
Mash enumera y conserva los archivos de `~/.mash` que no creó. Si administras
tu propio Qdrant, su servicio y los datos de búsqueda que contiene siguen
ahí. Gestiona esos datos por separado.

Para una reinstalación futura, `--keep-history` guarda la copia consultable de
tus conversaciones, la configuración, los tokens y los datos de búsqueda del
Qdrant gestionado por Mash. Aun con esta opción, la desinstalación quita los
servicios, las conexiones con agentes y los modelos descargados. Si prefieres
una copia de seguridad independiente, usa [`mash export DIRECTORY` (en inglés)](../upgrades.md).

Revisa primero con `mash uninstall --keep-history` y luego confirma con
`mash uninstall --yes --keep-history`.

El informe enumera los pasos que fallaron y sus motivos. Si falla alguno, el
comando devuelve el código 1. Repite el comando con las mismas opciones para
completar lo que quede pendiente. Después reinicia tus agentes para que
dejen de cargar Mash.

Para quitar solo el servicio de búsqueda, ejecuta `mash qdrant uninstall`.
Conserva los datos de Qdrant salvo que añadas `--delete-storage`. Esos datos
contienen lo necesario para las búsquedas. Puedes regenerarlos a partir del
índice con `mash index --rebuild-local-views --full`.

<!-- mash-docs-source: e9fdc82da63b901e9fbc86e2c87b4aa88f83b1670d47a01b3094284023646838 -->
