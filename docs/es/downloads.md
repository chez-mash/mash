# Descarga y verifica Mash

Descarga los archivos listos para usar, comprueba la suma publicada y abre
el panel web desde la carpeta donde los extraigas. Así puedes omitir el
script de instalación. También puedes [revisar el script](#inspect-the-installer)
o [instalar con un gestor de paquetes](install-methods.md).

<span id="choose-your-download"></span>

## Elige la descarga

Los enlaces llevan a los archivos de la última versión publicada. Cada
archivo incluye el programa de terminal y el panel web.

| Equipo | Descarga |
| --- | --- |
| Mac con Apple silicon | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/chez-mash/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/chez-mash/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/chez-mash/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/chez-mash/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

Consulta las [notas de la versión y el archivo de sumas en GitHub](https://github.com/chez-mash/mash/releases/latest).
Para comprobar una descarga, usa el archivo y el `SHA256SUMS` de la misma
versión. Consulta los [requisitos de Linux](installation.md#recommended-release-installers).
En plataformas como Mac con Intel, puedes [compilar desde el código fuente](install-methods.md#build-from-source).

<span id="verify-the-checksum"></span>

## Comprueba la suma

Desde la carpeta de la descarga, ejecuta el comando de tu sistema.
En Linux ARM64, sustituye el nombre por el de tu archivo.

**Linux**

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

**macOS**

```sh
shasum -a 256 mash-aarch64-apple-darwin.tar.gz
```

**PowerShell de Windows**

```powershell
Get-FileHash -Algorithm SHA256 .\mash-x86_64-pc-windows-msvc.zip
```

Busca el nombre exacto del archivo en `SHA256SUMS` y compara la suma de
esa línea con la que muestra el comando. Las mayúsculas y minúsculas son
equivalentes en la suma. Si coincide, el archivo es igual al que se publicó.

<span id="use-the-archive"></span>

## Usa los archivos descargados

Extrae `mash` y `mash-server` (o los dos archivos `.exe` en Windows) a una
carpeta tuya. En una terminal abierta en esa carpeta, ejecuta
`./mash --version` en macOS o Linux, o `.\mash.exe --version` en PowerShell.
El comando muestra la versión que descargaste.

Desde esa misma carpeta, inicia la búsqueda local y abre el panel web.
El primer comando descarga Qdrant e inicia el servicio con tu cuenta.
Omítelo si ya tienes un [Qdrant local compatible](local-search.md#run-qdrant-yourself).
En Linux o WSL, el comando de instalación requiere una
[sesión de usuario de systemd](local-search.md#linux-without-a-systemd-user-session).

**macOS o Linux**

```sh
./mash qdrant install
./mash dash
```

**Windows PowerShell**

```powershell
.\mash.exe qdrant install
.\mash.exe dash
```

Deja abierta la terminal y abre en el navegador la dirección que muestra,
por lo general [http://127.0.0.1:8790](http://127.0.0.1:8790).
Revisa qué conversaciones puede leer Mash y el intervalo de fechas.
Si prefieres buscar por tu cuenta, deja los agentes sin conectar y elige
**Set up and index**. [Pasos de configuración en el navegador](dashboard.md#set-up-mash-from-the-dashboard)

<span id="inspect-the-installer"></span>

## Revisa el instalador

Puedes leer el [instalador de macOS y Linux](https://github.com/chez-mash/mash/blob/main/install.sh)
o el [instalador de Windows](https://github.com/chez-mash/mash/blob/main/install.ps1)
antes de ejecutarlo. Los scripts comprueban la suma del archivo y colocan
Mash en tu carpeta de usuario sin privilegios de administrador.
[Qué cambia al configurar Mash](installation.md#what-setup-changes)

<span id="check-how-it-was-built"></span>

## Comprueba cómo se compiló

El [proceso público de publicación](https://github.com/chez-mash/mash/actions/workflows/release.yml)
prepara los archivos tras comprobar que las pruebas de Linux y Windows
pasaron para el código de la versión. Las
[pruebas públicas de instalación](https://github.com/chez-mash/mash/actions/workflows/release-check.yml)
muestran qué instaladores se ejecutaron y confirmaron la versión esperada.

Para las versiones que incluyen una certificación de compilación, GitHub
CLI puede verificar qué proceso y etiqueta de origen produjeron el archivo.
Inicia sesión en GitHub CLI y sustituye `ARCHIVE` por la ruta de la descarga
y `RELEASE_TAG` por su etiqueta, incluida la `v` inicial.

```sh
gh attestation verify "ARCHIVE" --repo chez-mash/mash --signer-workflow chez-mash/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

El proceso de publicación comprueba las certificaciones antes de publicar
esos archivos. Hasta `v0.1.1-alpha`, usa la comprobación de suma anterior.

<!-- mash-docs-source: d089547ce72f1abd31d192459b684cc330176a9a9a11aeeadbb18065c6b8f2c4 -->
