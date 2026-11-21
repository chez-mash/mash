# Local search service (Qdrant)

Mash uses Qdrant for local indexing and search. Use the built-in installer
for a service that starts when you log in, or run Qdrant yourself if you
already have it or need a different setup. After it starts,
[connect your histories](installation.md#connect-your-histories).

## Let Mash manage Qdrant

```sh
mash qdrant install
```

Indexing and every search use a local [Qdrant](https://qdrant.tech) service.
This command downloads Qdrant 1.20.0 for your platform (about 30 MB) from
[Qdrant's GitHub releases](https://github.com/qdrant/qdrant/releases/tag/v1.20.0),
checks it against a SHA-256 checksum built into Mash, and starts it in the
background as your user. It needs no administrator rights or Docker, listens
only on this machine (`127.0.0.1`, ports 6333 and 6334), and starts again when
you log in. The JSON result has `"outcome": "installed"`.

| Platform | Runs as | Qdrant's output | Data |
| --- | --- | --- | --- |
| Linux | systemd user service `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant` |
| Windows | logon task `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `C:\mash-qdrant`, because Qdrant fails with longer data paths there |

`mash status` shows the installed version and whether the service is running,
under `local_qdrant.installed`.

On Windows the logon task runs only while you are signed in at the desktop.
On a machine you reach only over SSH, or a server nobody signs in to, the
command reports that Qdrant did not become ready; start Qdrant yourself there,
as [native Windows without Docker](#native-windows-without-docker) describes.

Rerun the command at any time. It restarts a stopped service, restores missing
files, and upgrades Qdrant when a newer Mash pins a newer release, without
changing Qdrant's data. The outcome is `current` when nothing needs
doing, and `already_running` (nothing installed) when a compatible Qdrant
already answers on those ports, such as the Docker container below. If another
program holds a port, the command stops and says which.

- **Linux:** the command needs a systemd user session. Where there is none, as
  in some containers and WSL setups, [run Qdrant yourself](#run-qdrant-yourself).
  The service runs while you are logged in; to keep it running on a machine
  nobody logs in to, enable lingering with `loginctl enable-linger` (some
  systems allow this only for administrators).
- **Other ports:** set `url` and `grpc_url` in the
  [Qdrant configuration](configuration.md#local-qdrant) to `127.0.0.1`
  addresses before installing.

`mash init` checks that Qdrant answers, but never downloads or starts it.

## Run Qdrant yourself

Use the [official Docker image](#optional-qdrant-docker-image), start Qdrant
by hand as below, or use an existing Qdrant 1.20+ service. Both the HTTP and
gRPC ports must reach the same instance. For nondefault endpoints, see
[Qdrant configuration](configuration.md#local-qdrant). Mash and the dashboard
still run directly on your machine.

Each Qdrant has its own storage. When you switch between a native Qdrant, the
Docker container and `mash qdrant install`, stop the old one first to free
ports 6333 and 6334, then populate the new one with
`mash index --rebuild-local-views --full`.

### Native Windows without Docker

On Windows x64, download [Qdrant 1.20.0 for Windows](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip)
and extract it into `C:\mash-qdrant` (or another short, writable folder).
Longer paths under your user profile can make Qdrant's payload indexes fail
even when its startup health check succeeds. In PowerShell, run:

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

Keep this window open and use the same settings next time to reuse the data.
In a second terminal, check readiness with
`curl.exe --fail http://127.0.0.1:6333/readyz` before running `mash init`.

### Native macOS or Linux without Docker

Download the archive for your machine from the
[Qdrant 1.20.0 release](https://github.com/qdrant/qdrant/releases/tag/v1.20.0):

| Machine | Archive |
| --- | --- |
| macOS, Apple Silicon | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| macOS, Intel | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux, x86_64 | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux, ARM64 | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

Extract it into a dedicated folder, such as `~/qdrant`, and start it there:

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

Keep that terminal open and start Qdrant from the same folder next time to
reuse its data. In a second terminal, check readiness with
`curl --fail http://127.0.0.1:6333/readyz` before running `mash init`.

### Optional Qdrant Docker image

Install and start [Docker Desktop](https://www.docker.com/products/docker-desktop/)
on macOS/Windows, or [Docker Engine](https://docs.docker.com/engine/install/)
on Linux. Windows needs Linux containers; Windows Server/Engine-only hosts may
not support them. On those hosts, use the [native Windows instructions](#native-windows-without-docker).
Run this in Bash or PowerShell once Docker is running.

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

Only this machine can reach it. Open
[the readiness check](http://127.0.0.1:6333/readyz) and wait for “all shards
are ready” before running `mash init`.

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

Qdrant restarts with Docker unless you explicitly stopped it. If the container
already exists, use `docker start` rather than `docker run`. Data lives in the
`mash-qdrant-data` volume and survives stopping; delete the volume only to
discard it.
