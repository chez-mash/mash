# Servicio de búsqueda local (Qdrant)

Mash usa Qdrant para crear el índice y buscar en este equipo. Puedes dejar
que Mash lo instale y lo inicie al entrar en tu sesión, o usar tu propio
Qdrant. Cuando esté en marcha, [configura tus historiales](installation.md#connect-your-histories).

<span id="let-mash-manage-qdrant"></span>

## Deja que Mash gestione Qdrant

```sh
mash qdrant install
```

Este comando descarga [Qdrant 1.20.0 para tu plataforma](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)
(unos 30 MB), compara el archivo con la suma SHA-256 incluida en Mash e
inicia el servicio en segundo plano con tu usuario. Funciona sin Docker ni
permisos de administrador. Solo escucha en este equipo (`127.0.0.1`, puertos
6333 y 6334) y vuelve a iniciarse al entrar en tu sesión. El resultado JSON
de una instalación nueva incluye `"outcome": "installed"`.

<span id="where-qdrant-stores-data"></span>

### Dónde guarda Qdrant los datos

Estas son las rutas del servicio instalado por Mash. Guarda el texto de las
conversaciones que se puede buscar y los vectores que usa la búsqueda por
significado, si está habilitada.

| Sistema | Cómo se ejecuta | Registros de Qdrant | Datos |
| --- | --- | --- | --- |
| Linux | Servicio de usuario de systemd `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant/storage` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant/storage` |
| Windows | Tarea de inicio de sesión `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `%SystemDrive%\mash-qdrant\storage` |

Qdrant también usa la carpeta `snapshots` junto a `storage`. En Windows
estas carpetas suelen guardarse en `C:\mash-qdrant` para evitar fallos por
rutas largas. El programa, la configuración y el registro de Qdrant siguen en
`%USERPROFILE%\.mash\qdrant`.

En `mash status` puedes ver la ruta, la versión y el estado del servicio en
`local_qdrant.installed`. En el resultado de `mash qdrant install`, consulta
la ruta en `service.storage`. [Dónde guarda Mash los demás archivos](privacy.md#where-mash-stores-data)

<span id="check-or-restart-the-service"></span>

### Comprueba o reinicia el servicio

Si el servicio falla al iniciar, consulta [el comando o el archivo de registro de tu sistema](#where-qdrant-stores-data).

En Windows, la tarea se ejecuta mientras tengas una sesión de escritorio
abierta. Si solo accedes por SSH o nadie inicia sesión en el escritorio,
el comando indicará que Qdrant no llegó a estar listo. En ese caso,
[ejecútalo directamente en Windows](#native-windows-without-docker).

Puedes volver a ejecutar `mash qdrant install`. Reinicia el servicio si se
detuvo, restaura archivos que falten y actualiza Qdrant cuando una versión
nueva de Mash fija una versión más reciente de Qdrant. Conserva sus datos.
El resultado es `current` si el servicio gestionado ya funciona y está al día. Si
otro Qdrant compatible ya responde en esos puertos, por ejemplo desde el
contenedor de Docker, el resultado es `already_running` y Mash lo reutiliza.
Si otro programa ocupa un puerto, el comando se detiene e indica qué puerto.

- **Linux.** El comando necesita una [sesión de usuario de systemd](#linux-without-a-systemd-user-session).
  El servicio funciona mientras tu sesión esté abierta. Si nadie inicia
  sesión en ese equipo, puedes mantenerlo en marcha con `loginctl enable-linger`.
  Algunos sistemas reservan esta opción a administradores.
- **Otros puertos.** Antes de instalar, ajusta `url` y `grpc_url` a
  direcciones de `127.0.0.1` en la [configuración de Qdrant (en inglés)](../configuration.md#local-qdrant).

`mash init` solo comprueba si Qdrant responde. La descarga y el arranque se
hacen con `mash qdrant install` o al iniciarlo por tu cuenta.

<span id="linux-without-a-systemd-user-session"></span>

### Linux sin sesión de usuario de systemd

El instalador de Mash usa tu sesión de usuario de Linux para mantener
Qdrant en segundo plano. Haz esta comprobación de solo lectura en la misma
terminal de Linux o WSL que usarás para configurar Mash.

```sh
systemctl --user show-environment >/dev/null
```

Si termina sin errores, usa `mash qdrant install`. Si indica que no puede
conectarse al bus de usuario o no encuentra `systemctl`,
[inicia Qdrant en Docker](#optional-qdrant-docker-image) o
[ejecuta Qdrant directamente en una terminal de Linux](#native-macos-or-linux-without-docker).
Si prefieres habilitar systemd en WSL 2, sigue la
[guía de Microsoft](https://learn.microsoft.com/es-es/windows/wsl/systemd)
y repite la comprobación en tu terminal de WSL.

Si el instalador rápido se detuvo después de mostrar «Installed», los
ejecutables de Mash ya están instalados. Inicia Qdrant y ejecuta `mash dash`
para configurar Mash en el navegador o `mash init` para hacerlo en la terminal.
Usa la ruta de instalación si hace falta. En un equipo nuevo, usa el
[instalador que solo instala los ejecutables](installation.md#recommended-release-installers)
antes de iniciar Qdrant por tu cuenta.

<span id="run-qdrant-yourself"></span>

## Ejecuta Qdrant por tu cuenta

Usa la [imagen oficial de Docker](#optional-qdrant-docker-image), inicia
Qdrant en una terminal con las instrucciones de abajo o reutiliza un servicio
Qdrant 1.20 o posterior. Los puertos HTTP y gRPC deben apuntar a la misma
instancia. Para cambiar las direcciones, consulta la
[configuración de Qdrant (en inglés)](../configuration.md#local-qdrant).
Mash y el panel funcionan directamente en tu equipo.

Cada instancia de Qdrant guarda sus propios datos. Para cambiar entre Qdrant
nativo, Docker y `mash qdrant install`, detén primero el servicio anterior
para liberar los puertos 6333 y 6334. Llena el nuevo con
`mash index --rebuild-local-views --full`.

<span id="native-windows-without-docker"></span>

### Windows sin Docker

En Windows x64, descarga [Qdrant 1.20.0 para Windows](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip)
y extráelo en `C:\mash-qdrant` o en otro directorio corto donde puedas
escribir. Una ruta larga en tu perfil de usuario puede impedir que Qdrant
cree los índices de sus datos incluso si el servicio indica que ha
arrancado. Ejecuta estos comandos en PowerShell.

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

Deja esta ventana abierta. Vuelve a usar los mismos ajustes la próxima vez
para reutilizar los datos. Antes de ejecutar `mash init`, comprueba en otra
terminal si el servicio está listo con `curl.exe --fail http://127.0.0.1:6333/readyz`.

<span id="native-macos-or-linux-without-docker"></span>

### macOS o Linux sin Docker

Descarga el archivo para tu equipo de la [versión 1.20.0 de Qdrant](https://github.com/qdrant/qdrant/releases/tag/v1.20.0).

| Equipo | Archivo |
| --- | --- |
| macOS, Apple silicon | [Descargar](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| macOS, Intel | [Descargar](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux, x86_64 | [Descargar](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux, ARM64 | [Descargar](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

Extráelo en una carpeta propia, por ejemplo `~/qdrant`, e inícialo desde allí.

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

Deja la terminal abierta y vuelve a iniciar Qdrant en la misma carpeta la
próxima vez para reutilizar los datos. Antes de ejecutar `mash init`,
comprueba en otra terminal si está listo con `curl --fail http://127.0.0.1:6333/readyz`.

<span id="optional-qdrant-docker-image"></span>

### Qdrant en Docker

Instala e inicia [Docker Desktop (en inglés)](https://www.docker.com/products/docker-desktop/)
en macOS o Windows, o [Docker Engine (en inglés)](https://docs.docker.com/engine/install/)
en Linux. En Windows, la imagen requiere contenedores Linux. Si tu instalación
de Windows Server o Docker Engine no los admite,
[ejecuta Qdrant directamente en Windows](#native-windows-without-docker).
Con Docker en marcha, ejecuta este comando en Bash o PowerShell.

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

Solo se puede acceder al servicio desde este equipo. Abre la
[comprobación de disponibilidad](http://127.0.0.1:6333/readyz) y espera a
que muestre «all shards are ready» antes de ejecutar `mash init`.

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

Qdrant vuelve a iniciarse con Docker salvo que lo hayas detenido a mano.
Si el contenedor ya existe, reanúdalo con `docker start`. Los datos siguen
en el volumen `mash-qdrant-data` al parar el contenedor. Elimina ese volumen
solo si quieres borrar esos datos.

<!-- mash-docs-source: 5506852e48e6f2fe5fb713326fe5b9ea939c430cd352dd42c470fb85bf1aab9f -->
