# Otras formas de instalar Mash

Usa esta guía para instalar Mash con un gestor de paquetes o compilarlo a partir del
código fuente. Después de instalarlo, [inicia el servicio de búsqueda local](installation.md#start-the-local-search-service)
y [elige qué historiales leer](installation.md#connect-your-histories). Si ya tienes Mash instalado,
sigue los [pasos para actualizarlo](upgrades.md#upgrade-an-existing-installation).

<span id="install-with-a-package-manager"></span>

## Instala con un gestor de paquetes

Los paquetes de esta sección usan las versiones precompiladas de Mash. Incluyen el
panel web, la búsqueda en la terminal y las herramientas que puede usar un agente
conectado. Cuando la búsqueda local esté en marcha, puedes [terminar la configuración en el navegador](installation.md#set-up-in-your-browser).
Antes de instalar, puedes [verificar las descargas publicadas](downloads.md)
y [ver qué instaladores pasaron las comprobaciones de cada versión](https://github.com/mashhq/mash/actions/workflows/release-check.yml).

<span id="homebrew-macos-or-linux"></span>

### Homebrew en macOS o Linux

En macOS con Apple silicon o en Linux x86-64 o ARM64, instala Mash con la
[fórmula de Homebrew de MashHQ](https://github.com/mashhq/homebrew-tap/blob/main/Formula/mash.rb). El nombre completo indica a Homebrew qué repositorio usar y selecciona esa fórmula para el [sistema de confianza de taps de Homebrew](https://docs.brew.sh/Tap-Trust).

```sh
brew install mashhq/tap/mash
```

<span id="apt-debian-or-ubuntu"></span>

### apt en Debian o Ubuntu

Usa este método en Debian 12 o posterior, o Ubuntu 22.04 o posterior, en x86-64 o ARM64.
El repositorio apt de MashHQ publica metadatos firmados de sus paquetes. La opción
`signed-by` usa la [clave de firma de MashHQ](https://getmash.io/apt/mash-archive-keyring.gpg)
para este repositorio. Puedes [revisar los metadatos firmados](https://getmash.io/apt/dists/stable/InRelease)
antes de añadir el repositorio.

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

El paquete se llama `mash-cli` porque Debian y Ubuntu ya ofrecen otro paquete llamado `mash`. El comando que instala el paquete sigue siendo `mash`. Los dos paquetes no se pueden instalar a la vez. apt ofrece quitar el otro.

<span id="scoop-windows"></span>

### Scoop en Windows

En Windows x64, con Scoop ya instalado, abre PowerShell, añade el repositorio de MashHQ e instala su
[paquete de Mash](https://github.com/mashhq/mash/blob/main/bucket/mash.json). El manifiesto indica el archivo de Windows y su suma SHA-256.

```powershell
scoop bucket add mashhq https://github.com/mashhq/mash
scoop install mashhq/mash
```

### mise

Con mise instalado, usa su integración con GitHub para instalar Mash a partir de los
[archivos de las versiones de MashHQ](https://github.com/mashhq/mash/releases).

```sh
mise use -g github:mashhq/mash
```

### cargo-binstall

Con cargo-binstall instalado, indica el proyecto de GitHub para usar el archivo publicado por MashHQ.

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/mashhq/mash mash
```

Si no hay un archivo para tu plataforma, consulta cómo
[compilar a partir del código fuente](#build-from-source).

<span id="build-from-source"></span>

## Compila a partir del código fuente

Sigue estos pasos si no hay una versión precompilada para tu equipo o si quieres probar una revisión que todavía no se ha publicado. [Incluye el panel web](#include-the-web-dashboard)
para buscar desde el navegador. Para usar la [búsqueda en la terminal y las herramientas para agentes](#cli-and-agent-tools), usa el comando de Cargo más corto.

<span id="include-the-web-dashboard"></span>

### Incluye el panel web

Para incluir el panel, genera sus archivos web antes de compilar Mash. Con Git,
[Node.js 24 o posterior](https://nodejs.org/en/download) y una versión estable de Rust,
ejecuta estos comandos en la terminal.

```sh
git clone https://github.com/mashhq/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo install --locked --path .
```

Si quieres añadir el panel a una instalación existente, cambia a la revisión que instalaste antes de compilar. Después, [inicia la búsqueda local e indexa tus conversaciones](installation.md#start-the-local-search-service), ejecuta `mash dash`
y abre la dirección que muestra.

<span id="cli-and-agent-tools"></span>

### Búsqueda en la terminal y herramientas para agentes

Este comando compila la búsqueda en la terminal y las herramientas que puede usar un agente conectado. Para incluir el panel web, sigue los pasos de compilación anteriores.

```sh
cargo +stable install --locked --git https://github.com/mashhq/mash mash
mash --version
```

Después de compilar, [inicia la búsqueda local e indexa tus conversaciones](installation.md#start-the-local-search-service).
Luego [haz tu primera búsqueda en la terminal](search.md#your-first-search).

Si la terminal no encuentra Mash después de la compilación, consulta
[cómo añadir el directorio de ejecutables de Cargo a la ruta de comandos](troubleshooting.md#mash-command-not-found).

<!-- mash-docs-source: 400da6d93d5805098a5628678b37ade9f61dc9fdc1eedfb23cf768d2c4e84eec -->
