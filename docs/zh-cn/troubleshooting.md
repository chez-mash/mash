# 故障排查

| 遇到的问题 | 从哪里开始 |
| --- | --- |
| 终端找不到 `mash`，或 PowerShell 提示无法识别该命令 | [找到已安装的命令](#mash-command-not-found) |
| 设置助手无法连接搜索服务 | [启动 Qdrant](#qdrant-connection-failed) |
| 网页面板的地址打不开 | [检查面板是否已启动](#the-dashboard-doesnt-open) |
| Mash 没找到历史记录，或索引尚未建立 | [检查历史记录设置](#nothing-is-indexed-yet) |
| 可以浏览对话，但搜索没有结果 | [检查查询、筛选条件和日期](#search-returns-no-results) |
| 编程代理找不到 Mash | [检查代理连接](#my-agent-cannot-find-mash) |
| 升级后提示索引格式不兼容 | [恢复或升级索引](#the-index-schema-is-older-or-newer-than-mash) |

问题还没解决，或者怀疑遇到了安全问题？[获取帮助](#get-help)。

<span id="check-what-mash-sees"></span>

## 查看 Mash 检测到的情况

如果不清楚问题出在哪里，可以先运行只读诊断。

```sh
mash doctor
```

在 `findings` 中查找 `"state": "problem"`。`summary` 说明问题。如有 `repair`，它会给出处理方法或命令。发现问题时，命令返回退出码 1。标为 `warning` 的项目提示你可以检查哪些地方。`mash doctor` 不会更改配置或历史记录。

要查看某一部分的状态，可以选用下面对应的命令。

```sh
mash status                  # index and search readiness
mash status --installation   # history locations and agent connections
mash status --shared         # the index shared with your other machines
```

即使索引还未就绪，也可以运行 `mash status --installation`。遇到错误时，在下文查找 `error.kind` 或 `error.message` 中的文字。

<span id="mash-command-not-found"></span>

## 终端找不到 mash 命令

使用安装程序显示的完整路径，或者把安装目录加入当前终端的 `PATH`。以下命令使用发布版安装程序的默认路径。

```sh
export PATH="$HOME/.local/bin:$PATH"
mash --version
```

在 Windows 上，请在当前 PowerShell 窗口运行以下命令。

```powershell
$env:Path = "$HOME\.local\bin;$env:Path"
mash --version
```

如果你选择了其他安装位置，请用实际路径替换 `.local/bin`。通过 `cargo install` 安装时，通常使用用户目录下的 `.cargo/bin`。如果 `mash --version` 显示了版本号，就可以在这个终端中继续。要让新打开的终端也使用这个路径，可将该目录添加到 shell 配置文件，或在 Windows 的环境变量中添加到用户的 **Path**。

<span id="the-dashboard-doesnt-open"></span>

## 网页面板打不开

保持 `mash dash` 运行，然后用同一台电脑上的浏览器打开它显示的地址。如果已经在另一个终端启动了 Mash，请打开那个终端里显示的地址。如果有其他程序占用地址，可以让 Mash 选择一个空闲的本地端口。

```sh
mash dash --listen 127.0.0.1:0
```

`0` 表示请系统选择空闲端口。请打开 Mash 实际显示的地址。如果需要固定端口，请使用一个可用地址，例如 `127.0.0.1:8791`。

如果页面提示未构建网页面板，请安装发布版压缩包，或[从源码构建时加入网页面板](install-methods.md#include-the-web-dashboard)。

<span id="qdrant-connection-failed"></span>

## 无法连接 Qdrant

运行 `mash doctor` 并查看 `search-service`，找出搜索无法使用的原因。

如果 `mash status` 中的 `local_qdrant` 显示 `rejected`，请先查看原因，再重试设置或索引。Mash 可能使用了错误的端口或 API 密钥，也可能连接到了较旧的 Qdrant。HTTP 和 gRPC 地址必须指向同一个 Qdrant 实例，版本需为 1.x 系列中的 1.20 或以上。请参阅[如何设置自定义地址和 API 密钥（英文）](../configuration.md#local-qdrant)。

如果看到 `qdrant-unavailable`、`qdrant-install` 或“Could not reach Qdrant at …”，或者 `local_qdrant` 显示 `unreachable`，请检查服务是否正在运行。

如果在 Linux 或 WSL 上遇到 `systemctl` 错误，请[检查当前用户会话并选择 Qdrant 的启动方式](local-search.md#linux-without-a-systemd-user-session)。

按你当初安装 Qdrant 的方式启动它。

- **`mash qdrant install`**。重新执行同一条命令，可以启动已停止的服务并修复缺失的文件。`mash status` 的 `local_qdrant.installed` 会显示服务状态与日志位置。在 Linux 上，可以用 `journalctl --user --unit mash-qdrant.service` 查看日志。
- **Docker**。启动已有容器并查看日志。

  ```sh
  docker start mash-local-qdrant
  docker logs mash-local-qdrant
  ```

- **自行运行的 Qdrant**。从原目录重启，并查看终端输出。

如果还未设置 Qdrant，请按照[安装步骤](installation.md#start-the-local-search-service)操作。[就绪检查](http://127.0.0.1:6333/readyz)显示“all shards are ready”时，服务就可以用了。

`qdrant-failure` 表示 Qdrant 返回了错误。可以检查日志中的存储、凭据或数据格式错误。

Qdrant 停止时仍可列出对话、阅读消息和导出备份。如果还未设置 Mash，可以选择 **Save configuration only**，或运行 `mash init --no-index`。接着运行 `mash index`，将对话保存到本机。命令会在需要 Qdrant 时以 `qdrant-unavailable` 停止，并报告保存了多少条对话。可以用 `mash list-chats` 和 `mash view` 阅读它们。等 Qdrant 启动后，再运行 `mash index` 更新搜索索引。

在 Windows 上直接运行 Qdrant 时，如果日志同时提到 `gridstore`、“path specified”和 `os error 3`，数据路径可能过长。请使用[Windows 设置说明](local-search.md#native-windows-without-docker)中的较短绝对路径。如果是已有的安装，请先停止 Qdrant 并备份，再移动数据。保留原有数据。仅更改配置中的路径会让 Qdrant 打开一个新的空目录，原数据仍在原处。

<span id="nothing-is-indexed-yet"></span>

## 索引尚未建立

常见错误代码为 `database-missing`。

| 错误消息 | 要运行的命令 |
| --- | --- |
| “Mash is not set up on this machine” | `mash init` |
| “Mash is configured, but nothing is indexed yet” | `mash index` |

如果看到“database does not exist”，首次设置时运行 `mash init`。如果已经保存设置，则运行 `mash index`。两条命令都需要 Qdrant 运行才能建立搜索索引。

如果助手提示未找到编程代理的对话，请先在本机受支持的代理中保存一段对话，再运行 `mash init`。如果本已有对话，可以用 `mash status --installation` 查看 Mash 找到的历史记录目录，并在[设置](installation.md#connect-your-histories)中核对选中的代理和目录。如果缺少较早的对话，请[检查历史记录日期范围（英文）](../indexing.md#add-older-history)。

<span id="my-agent-cannot-find-mash"></span>

## 代理找不到 Mash

检查你准备使用的代理。以 Claude Code 为例，运行以下命令。

```sh
mash status --installation --agent claude-code
```

如果使用 Codex，请将选项换成 `--agent codex`。如果使用 Copilot CLI，请参阅[适用于你的 Mash 版本的检查步骤](agents.md#confirm-the-connection)。如果报告在 `next_action` 中给出修复办法，请按提示操作。[设置代理连接](agents.md#connect-claude-code-codex-or-copilot)期间仍可在网页面板中搜索。

完成设置或修复后，或者升级 Mash 可执行文件或插件后，请重启代理。已经打开的对话会继续使用原来的连接。要测试一个新的连接，可以运行以下命令。

```sh
mash status --installation --agent claude-code --probe
```

测试通过表示新的 MCP 进程能够连接。[设置代理连接](agents.md)

<span id="search-returns-no-results"></span>

## 搜索没有结果

1. 查看[搜索模式](dashboard.md#choose-a-search-mode)。关键词搜索要求所有词出现在同一条消息里，可以先试一个有辨识度的词。按含义搜索时，试着用一个简短的英文问题。对于词与词之间没有空格的文本，可参阅[如何搜索其他语言的对话](search-languages.md)。
2. 如果网页面板显示筛选标签，选择 **Clear filters** 后会用同一个查询重试。如果面板回到对话列表，请重新输入查询。在终端中，可移除筛选选项后再次搜索。
3. 确认对话是否在列表中。选择 **Browse chats**，按当前筛选条件打开对话列表。如果记得标题的一部分，可以把它输入搜索框，在建议中的 **chats** 下查找。在终端中，可使用下文的命令。
4. 如果缺少新的对话，或记录过早结束，请运行 `mash index` 后再检查。若缺少较早的对话，可以[扩大历史记录范围（英文）](../indexing.md#add-older-history)。如果缺少某个代理的对话，请[检查历史记录目录](#nothing-is-indexed-yet)。

搜索范围包括用户和代理之间的对话。要查看推理过程和工具输出，请搜索相关的消息并打开它所在的对话。

要在终端中按标题查找，请把命令中的文字替换成你记得的标题片段。

```sh
mash list-chats --title-contains 'part of the title'
```

要从第一条消息开始阅读，请把结果行中的 `lookup.view.id` 交给 `mash view`。也可以用对话 ID 在该对话中搜索。

<span id="models-are-missing"></span>

## 缺少搜索模型

报错可能为“semantic models are not installed”（`model`），或“related-discussion needs semantic search, which this configuration turns off”（`usage`）。

按含义搜索是可选功能，需要下载模型。你可以用下面的命令搜索词语和短语。

```sh
mash search 'auth_token' --intent exact-mention
```

要按含义搜索，请[安装模型（英文）](../indexing.md#install-the-models)并[构建搜索数据（英文）](../indexing.md#build-search-data)。安装模型会保留现有设置。如果已在用 Mash，指南也说明了如何在现有配置中启用按含义搜索所需的索引。

<span id="embedding-failed-during-indexing"></span>

## 建立索引时未能计算向量

如果 `mash index` 返回 `embedding_error`，这次处理已结束，但需要重试。Mash 已保存新消息，却未能计算向量，例如模型未安装，或远程计算服务没有响应。新消息已能用关键词搜索（`exact-mention`）。要按含义查找它们（`related-discussion`），请先解决报告中的问题，再运行 `mash index`。

<span id="new-chats-are-missing-or-search-is-behind"></span>

## 找不到新对话，或者搜索落后于历史记录

Mash 会在建立索引时读取新的历史记录。如果找不到近期对话或看到 `search-not-ready`，可以先查看具体提示。

- “search is catching up with new history”表示正在建立索引，请稍后再试。
- “search is behind the stored history and nothing is refreshing it”表示需要运行下面的命令。

```sh
mash index
```

代理插件会在会话开始时触发一次增量索引，最多每十分钟一次。如需持续更新，请参阅[如何让历史记录及时可搜（英文）](../synchronization.md)。如果某个代理的对话从未出现，请[查看 Mash 支持哪些本地历史记录，以及如何选择](supported-agents.md#if-a-chat-is-missing)。

<span id="a-chat-still-shows-what-an-older-release-read"></span>

## 对话仍显示旧版 Mash 读取的内容

如果对话文件自上次读取后没有变化，Mash 会跳过它。升级后，如果 Mash 改变了读取方式，例如给对话分配的目录变了，已建立索引的对话会保留旧结果，直到这些文件更新。要立即重新读取，请运行下面的命令。

```sh
mash index --reread
```

如果重新读取后对话内容相同，Mash 会保留原来的搜索数据，因此通常比首次建立索引更快。要限制为一个代理，可以加上类似 `--provider codex` 的选项。

<span id="search-views-are-missing-or-stale"></span>

## 搜索数据缺失或过时

如果数据库格式兼容，且需要的历史记录已保存在本机，可以从这些记录重建本地搜索数据。

```sh
mash index --rebuild-local-views --full
```

此命令使用 Mash 已保存的历史记录。要读取新的代理文件或重试失败的来源，请运行 `mash index` 并查看结果。如果数据库格式不兼容，请参阅下文的升级步骤。

<span id="the-index-schema-is-older-or-newer-than-mash"></span>

## 索引格式比 Mash 所支持的版本更早或更新

如果看到 `schema-incompatible`，请停止当前操作并参阅[升级与备份（英文）](../upgrades.md)。保留旧的可执行文件和数据。用写入索引时的版本导出，然后使用新版本，在另一个路径验证导入结果。

保留数据库和现有的格式版本号。`index --full` 用于建立索引。要迁移数据库格式，请按升级指南操作。如果数据库有升级标记，请先完成升级或恢复数据，再创建新索引。

<span id="indexing-is-busy-incomplete-or-shared-pending"></span>

## 索引正忙、尚未完成或等待发布到共享服务器

- **正忙**（`index-busy`，“another Mash operation is updating this local index”）。另一个进程正在修改索引。等它结束，或停止你启动的 `mash index --watch` 进程，再重试。请保留锁文件。
- **尚未完成**。查看来源报错，排除问题后再运行 `mash index`。Mash 会保留已保存的历史记录。
- **等待共享**。本地索引可能已完成。检查连接、令牌配置和 `mash status --shared`，然后重新运行 `mash index`。

跨设备搜索可能返回本机结果，并附带 `shared_error`。可以用 `--local` 只搜索本机，或修复共享连接后，再依赖跨设备搜索结果。

<span id="get-help"></span>

## 获取帮助

安装、配置或日常使用遇到问题时，可以[在 GitHub 上报告问题](https://github.com/mashium/mash/issues/new?template=bug_report.yml)。请说明你的操作系统、尝试过的安装方式、出错的步骤以及预期结果。如果 `mash --version` 能运行，请附上它的输出。如果 Mash 没有安装成功，提供安装程序的报错就可以提交报告。

GitHub 上的问题报告是公开的。只需分享相关的报错或 [`mash doctor`](#check-what-mash-sees) 的一小段输出。请删去私人路径和地址、对话内容及凭据。如果需要举例，请用虚构的对话。

如果你怀疑 Mash 泄露了私人历史记录、允许绕过登录，或者发布了校验和不匹配的软件包，请使用 [GitHub 私密报告表单](https://github.com/mashium/mash/security/advisories/new)。请参阅[安全报告应包含哪些信息](../../SECURITY.md#report-a-vulnerability)（英文）。

<!-- mash-docs-source: f174be304d981e17d943dda57d7aeb769d7ac14235d03c73a7e4d55406066641 -->
