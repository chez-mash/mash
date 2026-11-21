# 本地搜索服务（Qdrant）

Mash 使用 Qdrant 在本机建立索引和搜索。可以用 Mash 内置的安装命令设置登录时自动启动的搜索服务，也可以自行运行 Qdrant 或复用已有的服务。服务就绪后，[设置要读取的历史记录](installation.md#connect-your-histories)。

<span id="let-mash-manage-qdrant"></span>

## 让 Mash 管理 Qdrant

```sh
mash qdrant install
```

此命令下载适合你的系统的 [Qdrant 1.20.0](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)（约 30 MB），用 Mash 内置的 SHA-256 校验值核对文件，并以你的用户身份在后台启动服务。整个过程无需管理员权限或 Docker。服务只在本机监听（`127.0.0.1`，端口 6333 和 6334），并在你登录时重新启动。首次安装成功时，JSON 结果会包含 `"outcome": "installed"`。

<span id="where-qdrant-stores-data"></span>

### Qdrant 将数据存在哪里

下表适用于由 Mash 安装的 Qdrant。它保存可搜索的对话文字，也保存按含义搜索所需的向量（如果启用了此功能）。

| 系统 | 运行方式 | Qdrant 日志 | 数据目录 |
| --- | --- | --- | --- |
| Linux | systemd 用户服务 `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant/storage` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant/storage` |
| Windows | 登录任务 `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `%SystemDrive%\mash-qdrant\storage` |

Qdrant 还会在 `storage` 旁使用 `snapshots` 目录。在 Windows 上，这两个数据目录通常放在 `C:\mash-qdrant` 下，以免路径过长导致 Qdrant 出错。Qdrant 的程序、配置和日志仍在 `%USERPROFILE%\.mash\qdrant` 下。

运行 `mash status`，可以在 `local_qdrant.installed` 中查看实际路径、版本和服务状态。`mash qdrant install` 的结果也会在 `service.storage` 中显示路径。[查看 Mash 的其他文件位置](privacy.md#where-mash-stores-data)

<span id="check-or-restart-the-service"></span>

### 检查或重启服务

如果启动失败，请查看[对应系统的日志命令或文件](#where-qdrant-stores-data)。

在 Windows 上，登录任务只在你登录桌面时运行。如果只能通过 SSH 访问电脑，或者服务器上没人登录桌面，安装命令会报告 Qdrant 未能就绪。这时可以[在 Windows 上直接运行 Qdrant](#native-windows-without-docker)。

可以随时重新运行 `mash qdrant install`。它会重启已停止的服务、恢复缺失的文件，并在新版 Mash 指定了更新的 Qdrant 版本时完成升级。已有的 Qdrant 数据会保留。如果 Mash 管理的服务已在运行且版本符合要求，结果为 `current`。如果这些端口上已经有兼容的 Qdrant（例如下文的 Docker 容器），结果为 `already_running`，Mash 会直接使用该服务。如果其他程序占用了端口，命令会停止并指出该端口。

- **Linux。** 此命令需要 [systemd 用户会话](#linux-without-a-systemd-user-session)。服务会在你的登录会话期间运行。要让服务在无人登录时继续运行，可以用 `loginctl enable-linger` 启用用户服务常驻。部分系统只允许管理员这样做。
- **其他端口。** 安装前，在 [Qdrant 配置（英文）](../configuration.md#local-qdrant)中将 `url` 和 `grpc_url` 设为相应的 `127.0.0.1` 地址。

`mash init` 只检查 Qdrant 是否响应。请使用 `mash qdrant install` 安装并启动 Qdrant，或自行启动它。

<span id="linux-without-a-systemd-user-session"></span>

### Linux 或 WSL 缺少 systemd 用户会话

Mash 的安装命令依靠 Linux 用户会话让 Qdrant 在后台运行。请在打算用来设置 Mash 的同一个 Linux 或 WSL 终端中，运行以下只读检查。

```sh
systemctl --user show-environment >/dev/null
```

如果检查顺利完成，可使用 `mash qdrant install`。如果终端提示无法连接用户总线或找不到 `systemctl`，可以[在 Docker 中启动 Qdrant](#optional-qdrant-docker-image)，也可以[在 Linux 终端中直接运行 Qdrant](#native-macos-or-linux-without-docker)。如果想在 WSL 2 上启用 systemd，请参阅 [Microsoft 的 WSL 指南](https://learn.microsoft.com/zh-cn/windows/wsl/systemd)，然后在 WSL 终端中再次检查。

如果快速安装脚本在显示“Installed”后停止，Mash 的可执行文件已经装好。请先启动 Qdrant，然后运行 `mash init`，必要时使用 Mash 的完整安装路径。如果还没安装 Mash，请先用[仅安装可执行文件的脚本](installation.md#recommended-release-installers)，再自行启动 Qdrant。

<span id="run-qdrant-yourself"></span>

## 自行运行 Qdrant

可以使用[官方 Docker 镜像](#optional-qdrant-docker-image)，按下文说明直接启动 Qdrant，或复用已有的 Qdrant 1.20 或更高版本。HTTP 和 gRPC 端口必须连接到同一个实例。如果使用非默认地址，请查看 [Qdrant 配置（英文）](../configuration.md#local-qdrant)。Mash 和网页面板仍直接运行在你的电脑上。

每个 Qdrant 实例各自保存数据。在直接运行的 Qdrant、Docker 容器和 `mash qdrant install` 之间切换时，请先停止原来的服务，腾出端口 6333 和 6334。然后运行 `mash index --rebuild-local-views --full`，在新实例上重建搜索数据。

<span id="native-windows-without-docker"></span>

### 在 Windows 上直接运行

在 Windows x64 上下载 [Qdrant 1.20.0 Windows 版](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip)，解压到 `C:\mash-qdrant`，或其他路径较短且可写的目录。用户目录下的路径如果太长，即使 Qdrant 通过启动检查，也可能无法建立数据索引。在 PowerShell 中运行以下命令。

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

保持这个窗口打开。下次使用相同的设置即可复用数据。运行 `mash init` 前，请在另一个终端中用 `curl.exe --fail http://127.0.0.1:6333/readyz` 检查服务是否就绪。

<span id="native-macos-or-linux-without-docker"></span>

### 在 macOS 或 Linux 上直接运行

请从 [Qdrant 1.20.0 的发布页面](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)下载适合你电脑的压缩包。

| 系统 | 压缩包 |
| --- | --- |
| macOS，Apple 芯片 | [下载](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| macOS，Intel 芯片 | [下载](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux，x86_64 | [下载](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux，ARM64 | [下载](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

解压到专用目录，例如 `~/qdrant`，再从该目录启动 Qdrant。

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

保持这个终端打开。下次从同一目录启动 Qdrant 即可复用数据。运行 `mash init` 前，请在另一个终端中用 `curl --fail http://127.0.0.1:6333/readyz` 检查服务是否就绪。

<span id="optional-qdrant-docker-image"></span>

### 在 Docker 中运行 Qdrant

在 macOS 或 Windows 上安装并启动 [Docker Desktop（英文）](https://www.docker.com/products/docker-desktop/)，在 Linux 上使用 [Docker Engine（英文）](https://docs.docker.com/engine/install/)。Windows 上运行此镜像需要支持 Linux 容器。如果你的 Windows Server 或 Docker Engine 环境不支持，请[在 Windows 上直接运行 Qdrant](#native-windows-without-docker)。Docker 启动后，在 Bash 或 PowerShell 中执行下面的命令。

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

只有本机能访问该 Qdrant 服务。打开[服务就绪检查](http://127.0.0.1:6333/readyz)，看到“all shards are ready”后再运行 `mash init`。

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

Qdrant 会随 Docker 重启，手动停止容器时除外。如果容器已经存在，用 `docker start` 启动它。停止容器后，数据仍保存在 `mash-qdrant-data` 数据卷中。只有打算删除这些数据时，才需要删除该数据卷。

<!-- mash-docs-source: 545085b01bb01a6e80ed969f8e815ec8bf83993ba7a8f8575957f023963237a0 -->
