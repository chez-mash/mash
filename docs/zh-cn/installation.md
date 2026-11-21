# 安装 Mash

安装 Mash，启动搜索服务，再选择要搜索哪些对话。默认设置会在你的电脑上建立索引和搜索。[Mash 如何处理历史记录](privacy.md)

你可以先[搜索演示中的示例对话](https://getmash.io/demo/)。想用自己的数据试一试，可以[选择一份历史记录，在本地网页面板中试用](privacy.md#用少量本地历史试用)。

[快速入门](quick-start.md)提供一条命令，可安装 Mash、启动搜索服务并打开设置助手供你核对选项。如果已经安装 Mash，请[启动搜索服务](#start-the-local-search-service)。如果服务也已运行，请直接[设置要读取的历史记录](#connect-your-histories)。

如果已有 Mash，现在要更新到新版本，请参阅[升级步骤](upgrades.md#upgrade-an-existing-installation)。

<span id="what-setup-changes"></span>

## 设置会更改什么

1. **安装程序。** [基本安装脚本](#recommended-release-installers)会将 `mash` 和 `mash-server` 放到你的用户目录。也可以自行[校验并解压发布归档](downloads.md)。
2. **启动本地搜索。** `mash qdrant install` 会下载 Qdrant，并将它设置为在你登录时以你的账号运行。Mash 只允许本机连接该服务，并关闭 Qdrant 的遥测。[搜索服务文件](local-search.md#where-qdrant-stores-data)
3. **选择 Mash 可以读取的内容。** 交互式 `mash init` 助手会先展示找到的历史记录目录，再保存设置或为对话建立索引。Mash 从原位置读取对话，默认在本机保存可搜索的副本。如果选择连接代理，设置助手会更新该代理的设置，让它可以从副本中请求对话片段。你也可以[跳过连接代理](#skip-agent-connections)，自行搜索。[Mash 的数据存放位置](privacy.md#where-mash-stores-data)

快速入门命令会先安装程序并启动 Qdrant，再打开设置确认页。如果在这里取消，可以运行 `mash uninstall` 查看已安装的内容和移除方法。[预览卸载](uninstall.md)

<span id="install-the-cli"></span>

<span id="get-mash-for-your-system"></span>

## 下载适合你的系统的 Mash

预编译版本包含网页面板。搜索服务启动后，可以[在浏览器中设置 Mash](#set-up-in-your-browser)。也可以[连接编程代理](agents.md)，让它们帮你搜索历史记录。

<span id="recommended-release-installers"></span>

### 推荐的安装脚本

预编译版本支持 Linux x86-64 和 ARM64、搭载 Apple 芯片的 macOS，以及 Windows x64。选择适用于你系统的命令。运行前可以先[查看脚本或校验下载文件](downloads.md)。

Linux 可执行文件需要 glibc 2.34 或更高版本。Ubuntu 22.04、Debian 12、RHEL 9 及其后续版本符合要求。Alpine 等使用 musl 的系统暂不支持。

<span id="macos-or-linux"></span>

#### macOS 或 Linux

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

WSL 也使用这条命令。

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

两个脚本都会校验下载文件，并将 Mash 安装到用户目录下的 `.local/bin`。脚本会显示后续命令和可执行文件的完整路径，即使目录尚未加入 `PATH` 也能运行。通过 `MASH_VERSION` 可以指定已发布的版本标签，通过 `MASH_INSTALL_DIR` 可以选择安装目录。脚本会保留你的终端配置。如果终端找不到 `mash`，请参阅[如何找到命令](troubleshooting.md#mash-command-not-found)。

<span id="package-manager-alternatives"></span>

### 使用包管理器安装

可以用 [Homebrew、apt、Scoop、mise 或 cargo-binstall 安装](install-methods.md#install-with-a-package-manager)。安装后请[启动搜索服务](#start-the-local-search-service)。

<span id="build-from-source"></span>

### 从源代码构建

要在浏览器中搜索，请[构建包含网页面板的版本](install-methods.md#include-the-web-dashboard)。如果只需要[终端搜索和代理工具](install-methods.md#cli-and-agent-tools)，可以用较短的 Cargo 命令。

<span id="start-the-local-search-service"></span>

## 启动本地搜索服务

在 Linux 或 WSL 上，下面的命令需要 systemd 用户会话。[检查当前会话或查看其他启动方式](local-search.md#linux-without-a-systemd-user-session)。如果已运行[兼容的本地 Qdrant](local-search.md#run-qdrant-yourself)，可以直接[设置要读取的历史记录](#connect-your-histories)。

在 Windows 上，Mash 会把 Qdrant 的搜索数据放在系统盘上路径较短的目录中，通常是 `C:\mash-qdrant\storage`。[Qdrant 的文件和日志位置](local-search.md#where-qdrant-stores-data)

```sh
mash qdrant install
```

Mash 校验下载的 Qdrant（约 30 MB），再以你的用户身份在后台启动服务。Qdrant 只在本机监听，并在你登录时重新启动。在 Windows 上，需要登录桌面。如果服务启动失败，请参阅[如何查看日志并重启服务](local-search.md#check-or-restart-the-service)。

<span id="run-qdrant-yourself"></span>

### 自行运行 Qdrant

你可以[复用本机的 Qdrant 1.20 或更高版本](local-search.md#run-qdrant-yourself)。如需启动一个新实例，请选用适合你环境的说明。

- <span id="optional-qdrant-docker-image"></span>[在 Docker 中运行 Qdrant](local-search.md#optional-qdrant-docker-image)
- <span id="native-windows-without-docker"></span>[在 Windows 上直接运行](local-search.md#native-windows-without-docker)
- <span id="native-macos-or-linux-without-docker"></span>[在 macOS 或 Linux 上直接运行](local-search.md#native-macos-or-linux-without-docker)

<span id="connect-your-histories"></span>

## 设置要读取的历史记录

本地搜索服务启动后，可以在网页面板或终端里选择 Mash 要读取哪些对话。

<span id="set-up-in-your-browser"></span>

### 在浏览器中设置 Mash

如果你安装了已发布的版本，或[构建了带网页面板的 Mash](install-methods.md#include-the-web-dashboard)，请在终端运行：

```sh
mash dash
```

保持终端运行，在浏览器中打开终端显示的网址。首次安装时，网页面板会打开设置页。核对历史记录和日期范围。如果只想自己搜索，请保持代理连接关闭。选择 **Set up and index**。索引就绪后，打开 **chats**，搜索一个你记得的词。
[查看网页面板的设置步骤](dashboard.md#set-up-mash-from-the-dashboard)

### 在终端中设置 Mash

如果你的构建没有网页面板，或你想自行选择历史记录目录、连接共享服务器，请使用终端设置助手。

```sh
mash init
```

首次设置时，终端助手会显示找到的历史记录目录、读取范围（最近 90 天内更新过的文件），以及可连接的代理。如果向导询问在哪里搜索，请选择 **Just this computer (recommended)**，将索引保存在本机。**Set up and index** 会应用摘要中的选项，包括连接已选中的代理。即使代理的命令不在本机，Mash 也能读取它已保存的对话。

选择 **Customize setup** 可以修改读取范围或历史记录目录，选择要接入的代理，连接已有的共享服务器，或先保存设置，稍后再建立索引。这个选项还允许在建立索引后[从旧对话中补录记忆](records.md#backfill-earlier-memory)。补录时，Mash 会使用你已登录的 Claude Code 或 Codex，将选中的对话文字发送给该代理的模型服务。

在应用设置前按 Esc 或 Ctrl-C，可以取消对 Mash 配置和代理设置的更改。如果选择建立索引，助手会检查本地搜索服务的 HTTP 接口。如果选择共享，还会检查能否通过服务器的身份验证。如果连接失败，可以先保存设置，稍后再完成配置。

如果连接了代理，请重启代理以加载 Mash。历史记录插件的钩子需要 Python 3.9 或更高版本。[在代理中使用 Mash](agents.md)

<span id="skip-agent-connections"></span>

### 跳过代理连接

你可以自己在网页面板或终端搜索对话，无需授权代理使用 Mash 搜索。根据向导显示的选项操作。

- 如果首次核对设置时看到 **Set up and index without connecting agents**，选择它，为列出的历史记录建立索引。
- 如果向导询问 **Which agents should get access to Mash search?**，用空格键取消选中所有代理，然后按回车键。如果向导询问是否从旧对话中补录记忆，请跳过补录。
- 如果摘要显示没有要连接的代理，选择 **Set up and index**。

搜索就绪后，[打开网页面板](#try-your-first-search)。你也可以[稍后再连接代理](agents.md#connect-claude-code-codex-or-copilot)。

<span id="try-your-first-search"></span>

## 开始第一次搜索

如果安装了网页面板，请运行 `mash dash`，然后在浏览器中打开显示的地址。在 **chats** 中输入记得的关键词，打开搜索结果即可阅读原始消息。

你也可以[在终端中搜索](search.md#your-first-search)，或者[让已连接的代理搜索](agents.md#try-it)。如果找不到较早的对话，可以[扩大历史记录范围](indexing.md#add-older-history)。

关键词搜索不需要下载模型。要按含义搜索，请[配置可选的搜索模型](indexing.md#enable-semantic-search)。

<span id="setup-options"></span>

## 配置选项

<span id="without-the-wizard"></span>

### 跳过设置助手

`mash init --yes` 接受检测到的默认选项。它会读取最近 90 天内修改过的文件，包括找到的所有受支持代理（含 Copilot）的当前会话，并应用可用的代理连接，但不会补录旧对话中的记忆。如果标准输入或标准错误输出未连接到终端，命令也会以非交互方式运行。JSON 结果会列出选中的历史记录目录。

跳过助手时，`mash init` 会在更改任何设置之前检查 Qdrant。如果 Qdrant 没有响应，命令会给出启动方法，并保留 Mash 的配置和代理设置。

| 选项 | 用途 |
| --- | --- |
| `--no-index` | 保存配置，稍后再建立索引。跳过 Qdrant 检查，并在后续步骤中提示如何启动 Qdrant。要手动选择目录，可编辑[配置文件（英文）](../configuration.md)，然后运行 `mash index`。 |
| `--no-agents` | 使用网页面板或终端搜索，无需连接代理。 |
| `--force` | 非交互设置默认保留现有配置。配合 `--yes` 使用此选项时，Mash 会根据找到的历史记录重新生成配置。在助手中，此选项会直接打开设置确认页。 |

未指定 `--force` 时，助手会让你选择保留或核对现有设置。在核对后保存时，Mash 会保留助手未提供的设置，并备份原配置文件。

Qdrant 启动前也可以阅读历史记录。先运行 `mash init --no-index`，再运行 `mash index`。Mash 会先将对话保存到本机，然后在需要 Qdrant 时停止，报告保存了多少条对话并给出服务启动方法。此时可以用 `mash list-chats` 和 `mash view` 阅读已保存的对话。Qdrant 就绪后，再运行一次 `mash index` 即可更新搜索索引。

<span id="read-the-result"></span>

### 查看设置结果

终端助手会显示简要报告和下一步。搜索就绪后，会根据已安装的版本提示如何打开面板或在终端搜索。要获取完整的 JSON 报告，可以运行 `mash init --yes`，或用 `mash init > setup.json` 重定向助手的标准输出。提示和进度写入标准错误输出。

- `"agents": []` 表示没有连接代理，索引仍可能已成功建立。查看 `next` 了解原因，或参阅[如何连接代理](agents.md)。你可以直接在终端搜索。
- 对于每个已连接的代理，`files` 列出该代理的命令修改过的设置文件，`undo` 列出移除连接的命令。
- `index_error` 表示设置已保存，但首次建立索引失败。例如，Qdrant 停止运行或无法访问其 gRPC 端口。Mash 会报告错误，并保留已保存的配置、代理连接和历史记录。解决问题后，运行 `mash index`。
- 再次运行时出现 `written: false`，表示 Mash 复用了已有的配置。
- 可搜索的消息数只包含用户和代理之间的对话。推理过程和工具输出仍可在对话中阅读，所以可搜索的消息数可能比对话中的消息总数少很多。

<span id="check-the-release-download"></span>

## 校验下载文件

[选择安装包并核对校验值](downloads.md)，或在运行前[查看安装脚本](downloads.md#inspect-the-installer)。

<span id="uninstall"></span>

## 卸载 Mash

运行 `mash uninstall` 可以预览将移除的内容。[查看哪些内容会保留，以及如何卸载 Mash](uninstall.md)。

<!-- mash-docs-source: 52ce3de441d218c0570a52571fd86f47587a6a60e2bdd2d55cf2b6fe726eef61 -->
