# 使用 Mash 网页面板

在演示中[搜索发票导出相关的对话](https://getmash.io/demo/#/search?q=invoice+exports+queue)，打开一项结果就能看到原始对话。也可以[打开一项已保存的决定](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)，选择 **open the message** 查看决定的出处。演示使用示例对话。

Mash 的预编译版本包含网页面板。如果自行编译，请按步骤[将面板编译进去](../install-methods.md#include-the-web-dashboard)。[安装 Mash](quick-start.md) 后，在终端运行以下命令。

```sh
mash dash
```

在浏览器打开命令显示的地址，通常是 **[http://127.0.0.1:8790](http://127.0.0.1:8790)**。完成设置后，概览页会显示最近的对话和记忆。请让终端保持运行，按 Ctrl-C 可以停止面板。如果打不开页面，请参阅[面板启动排查（英文）](../troubleshooting.md#the-dashboard-doesnt-open)。

安装 [Mash 插件](../plugin.md)后，可以在 Claude Code 中运行 `/mash:dashboard`，也可以让你的代理打开 Mash 网页面板。

默认的本机面板无需登录。更改监听地址前请参阅[如何限制访问](#keep-access-private)。面板上的按钮和菜单使用英文。

<span id="set-up-mash-from-the-dashboard"></span>

## 在网页面板中设置 Mash

第一次设置可以在浏览器完成。运行 `mash dash` 并打开网页面板。没有配置文件时，面板会直接显示设置页面，此时不会显示侧栏和搜索框。选择 **Skip to the dashboard** 即可进入面板，之后可在 **settings** 中回到设置。也可以在终端用 `mash init` 设置。

核对 Mash 找到的历史、要索引的时间范围，以及可连接的代理。**Show history folders** 会列出历史文件夹。首次设置默认会索引本机过去 90 天的历史。选择 **Set up and index** 应用当前选项，或选择 **Customize setup** 修改历史范围、来源和代理连接。

在首次确认页面，**Skip agent connections** 会保留相同的历史索引，方便你自己在面板或终端搜索。如果你的版本先显示历史选项，选择 **Next** 进入确认页面。

在确认页面选择 **Set up and index** 或 **Set up without indexing** 后，Mash 才会写入配置。它会通过所选代理自己的命令行工具进行连接，并按你的选择在后台建立索引。页面会显示已保存的配置、各代理修改的文件以及撤销连接的命令，还会显示索引进度，直到对话出现在面板中。

- 索引需要搜索服务。如果服务未运行，请执行 `mash qdrant install` 后选择 **Check search service again**，或选择 **Set up without indexing**，稍后再索引。搜索服务可用时，自定义设置的确认页也允许保存配置并稍后索引。
- 要在网页面板中按语义搜索，请按[模型设置指南](../indexing.md#enable-semantic-search)安装模型，并将语义搜索设为默认模式。
- 重新设置已配置的机器时，Mash 会按你的选项更新配置并在同一位置保留旧文件。
- 要跨机器共享、从旧对话提取记忆，或读取存放在其他位置的历史，请使用 `mash init` 或编辑配置文件。
- 如果其他机器能访问网页面板，启动时需要提供服务配置，才能在浏览器中设置 Mash 或修改设置。

<span id="the-rail"></span>

## 使用侧栏

在手机上通过菜单按钮打开侧栏。

- **overview** 显示最近的对话和记忆，以及各台机器和各个代理的对话数量。
- **chats** 按时间从新到旧列出对话，并提供搜索框。**viewing history** 列出在此浏览器会话中打开过的对话，再次点击会回到上次读到的位置。
- **agents**、**machines** 和 **projects** 各有自己的页面，显示近期对话以及这些对话在另外两个类别中的分布。
- **memory** 显示已保存的决定、偏好和事实。
- **services** 和 **fleet manager** 显示进程、数据库、各机器上的代理设置及可用的管理操作。配置多台机器的集中管理后才会显示 **fleet manager**。[服务管理](../services.md)
- **tool explorer** 显示代理可以调用的工具及参数。运行只读工具可以看到代理收到的响应。会写入数据的工具仅供查看。
- **settings** 显示本机配置和索引状态。你可以选择要索引的代理、时间范围、是否发布到共享索引、是否提取记忆及如何遮盖凭据。如果配置了 [Mashy](#ask-mashy)，也可以开关它或选择模型。保存时，Mash 会把上一份配置保留为 `config.toml.previous`。请重启索引监听程序并重新连接代理来应用设置。Mashy 的设置会立即生效。搜索和存储设置仅供查看。如果面板可从其他机器访问，启动时需要加上 `--services-config` 才能修改设置。

右上角的按钮可以切换浅色和深色主题，浏览器会记住你的选择。

<span id="find-and-read-a-chat"></span>

## 查找并阅读对话

打开 **chats**，在 **Search your history** 输入一个记得的词并按回车。打开结果即可跳到原始对话中的匹配消息。

搜索框为空时，页面会列出对话。可以按代理、机器、项目和日期筛选对话列表或搜索结果。修改后的筛选项显示虚线边框，选择 **Apply filters** 后才会生效。已生效的筛选条件会以标签显示在搜索框里，点击标签即可移除。

如果筛选后没有结果，选择 **Clear filters**，用相同的查询搜索所有对话。如果你的版本回到了对话列表，请重新提交查询。如果没有筛选条件还是搜不到，请参阅[如何查找缺失的对话（英文）](../troubleshooting.md#search-returns-no-results)。

对话页面左边是代理的回复，右边是你的消息。工具调用和推理过程折叠为一行，展开即可查看每一步及其输出。侧边栏显示当前页中你发出的请求、同一项目的其他对话，以及根据此对话保存的记忆。**Copy session ID** 可以复制会话 ID，方便你在原代理中继续对话。

对话和记忆的查看页面均为只读。

<span id="choose-a-search-mode"></span>

### 选择搜索模式

提交搜索后，在筛选项旁打开 **search: default**，选择 **words** 或 **meaning** 后点击 **Apply filters**。**words** 会匹配对话中的词，**meaning** 依赖[可选模型](../indexing.md#enable-semantic-search)，接收英文查询。翻页时会沿用当前查询选择的模式。重新输入查询时会使用[默认模式](../configuration.md#search-defaults)。

如果你的面板没有此菜单，可以用 [`mash search --intent`](../search.md)选择模式。

<span id="inspect-conversation-records"></span>

## 查看已保存的记忆

从侧栏打开 **memory** 或其中一个类别。使用顶部按钮切换类别，在 **find a statement** 中搜索内容。可以按状态、审核状态和来源筛选。来源可以选本机记忆或已发布到共享组的记忆。选择 **Apply filters** 搜索，用 **Next page** 翻页。

打开记忆后可以查看修订记录、引用的原始消息和相关记忆。引用下方的 **open the message** 会打开原始消息。对话里的 **memory from this chat** 列出在任一保留版本中引用过这段对话的记忆。被引用的消息下方会出现 **Saved as**。

列表也会显示待审核或已停用的记忆。使用前，请检查状态、到期时间、证据警告以及已标明的冲突。要确认或修改记忆，请让代理使用 Mash 的[记忆工具](../records.md)。如果共享记忆的来源不可访问，打开链接会显示错误。

<span id="keep-your-place"></span>

## 更新时保留阅读位置

面板会跟进本机以及向共享索引发布历史的其他机器的更新。阅读位置不会自行改变。

- **对话和记忆列表**会在几秒内更新，包括概览、侧栏、近期对话、阅读对话时旁边的会话列表，以及代理、机器、项目和记忆各页。
- **正在阅读的对话**会在你位于底部时追加新消息并自动跟随。在其他位置阅读时，会出现 **N new messages** 按钮，点击即可跳到新消息。如果 Mash 重新读取了已展示的消息，选择 **Show updated messages** 才会应用变更。
- **已提交的对话或记忆搜索**在你再次提交前会保留结果和筛选条件。打开结果后用浏览器的后退按钮返回，也会保留原结果。
- **已打开的记忆**在选择 **check for updates** 后更新。

其他机器发布历史后，面板才能看到对应的对话和记忆。共享服务器的版本至少要和面板一样新。使用较旧的服务器或同时打开超过 16 个面板标签页时，请用侧栏底部的 **reload chat list** 或对话里的刷新按钮手动更新。

如果浏览器或共享服务器的连接中断，面板会自动重连并补上更新。

新对话会在建立索引后出现。运行 `mash index` 手动更新，或在另一个终端保持 `mash index --watch` 运行。两种方式的更新都会显示在面板上。

如果请求失败，错误提示会说明哪部分无法加载，并提供 **Retry**。连接错误表示浏览器无法访问 Mash。请检查连接，并确认 `mash dash` 仍在运行，然后选择 **Retry**。

<span id="ask-mashy"></span>

## 向 Mashy 提问

如果你[配置了模型](../configuration.md#the-dashboard-assistant)，每个页面右下角都会出现带 Mash 吉祥物的圆形按钮。点击后，可以直接用自然语言询问自己的历史。

- “我昨天做了什么？”
- “我们之前怎么决定做重启测试？”
- 在某段对话页面上可以问：“总结这段对话。”
- “为什么我找不到今天上午的对话？”

在宽屏上可以拖动标题栏，把窗口移开以免挡住阅读内容。双击标题栏可以恢复位置。在手机上，窗口会占满屏幕。

Mashy 会在对话和记忆中搜索并简要回答。回答引用的内容可以在背后的面板中打开。

- 对话和记忆会显示为标签。如果答案引用了一条消息，标签会直接打开对话中的那条消息。
- 引用卡片会显示说话人和日期，你说过的话会加上彩色边框。
- 项目、代理和机器各自链接到对应页面。
- 如果回答只涵盖部分匹配项，**See all** 会用原词打开完整搜索。
- `mash index` 等命令有 **Copy** 按钮。

回答下方会出现后续问题建议。如果搜索不到内容，Mashy 会告诉你。点击停止按钮可以中断回答。

要针对正在阅读的内容提问，选中一段文字后选择 **Ask Mashy**。该段文字会固定在聊天上方，随每次提问发送，直到你移除它。

Mashy 也可以告诉你搜索是否可用、索引中的最新对话有多新、哪些代理已连接，以及其他机器上次同步的时间。发现问题时，它会指出可以运行的命令。

它还可以参考内置指南回答 Mash 的使用问题，比如“如何与笔记本共享索引？”或“`mash init` 会改动什么？”。回答中引用的指南会在本站打开。

日期和时间以浏览器时钟为准。询问上周、某个月等时间范围时，Mashy 会查找在这段时间内有消息的对话。长对话会分段读取，Mashy 会说明回答覆盖了哪一段。

Mashy 使用只读工具查询你的历史和内置指南。它无法建立索引、删除历史、保存记忆、运行命令或更改设置。Mashy 对话只在当前标签页中保留，不会写入对话历史。

每次提问时，Mashy 读取的文本会发送给你配置的模型，其中包括你选中的段落。配置模型后才会显示 Mashy 并发送文本。

如果 Mashy 出错或回答不理想，可用 `MASH_TRACE=1` 启动面板后再问一次。日志会记录查询步骤、结果量、模型用时、令牌用量和停止原因。问题、回答及历史正文都不会写入日志。

<span id="save-a-transcript"></span>

## 保存单段对话

在对话中打开 **Export** 菜单，可以下载包含工具输出的完整对话，格式可选 Markdown 或 HTML。**Export to PDF** 会打开适合打印的页面，在浏览器打印窗口选择 **Save as PDF** 即可。

建议等索引完成后再下载，以免导出时内容发生变化。下载内容可能包含敏感信息。

每段对话都有自己的地址，格式为 `http://127.0.0.1:8790/#/chat/<chat id>`。有 [Mash 插件](../plugin.md)时，运行 `/mash:export` 会查找你说的对话，并在该地址打开它。

<span id="export-many-chats-at-once"></span>

## 一次导出多段对话

打开 **chats**，选择 **export chats**，按代理、机器、项目和时间筛选。所有页面上匹配的对话都会导出，取消勾选的除外。选择 **Markdown** 便于阅读，或选择 **JSON** 供脚本使用，然后点击 **Export**。浏览器会下载一个 `.zip` 文件，每个项目有一个文件夹，每段对话有一个文件。

保持页面打开直到下载开始。一旦有对话无法读取，导出会停止且不会保存文件。导出包含完整的对话文本，包括可能出现的敏感内容，附件不在其中。

如果要把历史恢复到 Mash 中，请使用[可移植备份](../upgrades.md#make-a-portable-backup)。网页面板导出的文件用于阅读和分享。要在面板查看其他机器的对话，请参阅[共享历史指南](../shared-index.md)。

<span id="open-it-from-your-phone-or-another-computer"></span>

## 从手机或另一台电脑打开面板

默认只有本机可以访问面板。在 Linux 或 macOS 上连接 [Tailscale](https://tailscale.com) 后，运行以下命令，让你的其他设备可以在同一私有网络内访问。

```sh
mash dash remote
```

面板会以用户服务运行并要求登录。Linux 使用 systemd 用户服务，macOS 使用 LaunchAgent。命令会显示私有网络内的 HTTPS 地址、一次性验证码和二维码。

```json
{
  "dashboard": "https://laptop.your-tailnet.ts.net:8790/",
  "service": "mash-dash.service",
  "code": "K7M-2QX",
  "code_minutes": 10
}
```

在已连接 Tailscale 的手机上扫描二维码，或打开地址并输入验证码。浏览器会保持登录 30 天。

| 要做什么 | 命令 |
| --- | --- |
| 登录另一台设备 | `mash dash remote --code` |
| 再次显示二维码 | `mash dash remote --qr` |
| 使用其他端口 | `mash dash remote --port 8791`（每次运行都带上该参数） |
| 在没有本机历史的机器上托管面板 | `mash dash remote --shared-only` |
| 从没有 Tailscale 的设备访问 | `mash dash remote --public --port 8443` |
| 停止服务 | `mash dash remote --remove` |

验证码只能使用一次，十分钟内有效。二维码把长期令牌放在地址的 `#` 后面。浏览器加载网页时不会把这一段地址发送给服务器。面板会用令牌登录，再立即从地址中移除。请像保护密码一样保护二维码图片。

面板可以搜索本机历史。如果配置了[共享索引](../shared-index.md)，也可以搜索其他机器的历史。使用共享索引时，请在已设置本机令牌环境变量的终端中运行 `mash dash remote`。Mash 会将该变量保存到仅你可读的 `~/.mash/dashboard.env` 中。在 macOS 上，`~/Library/LaunchAgents/dev.mash.dash.plist` 同样受访问权限保护，内含相同令牌。输出写到 `~/.mash/dashboard.log`。

没有本机代理历史的机器也可托管面板，例如保存共享索引的服务器。加上 `--shared-only` 后，对话、搜索结果和记忆都来自共享索引。

`--public` 通过 [Tailscale Funnel](https://tailscale.com/kb/1223/funnel) 把地址发布到互联网，方便私有网络外的设备访问。Tailscale 必须为这台机器开放 Funnel，而且只能使用 443、8443 或 10000 端口。任何人都能访问登录页面，令牌和一次性验证码是历史记录的访问凭证。一分钟内出现十次错误尝试后，所有人都会暂时无法登录。能用私有网络时请优先使用。

<span id="set-it-up-by-hand"></span>

### 手动设置远程访问

在 Windows 上，或在 Caddy 等其他 HTTPS 反向代理后面运行时，可以按以下步骤设置。

1. 生成至少 32 个字符的令牌，放入环境变量，例如 `MASH_DASH_TOKEN`。
2. 为面板设置 HTTPS 地址。使用 Tailscale 时可运行 `tailscale serve --bg 8790`。
3. 用面板地址中的主机名和令牌环境变量启动面板。

   ```sh
   mash dash --allow-host laptop.your-tailnet.ts.net --access-token-env MASH_DASH_TOKEN
   ```

`--allow-host` 指定浏览器使用的名称。如果地址里有端口，也要带上端口。需要添加多个名称时，可以重复使用此参数。更改令牌会让所有浏览器退出登录。

<span id="keep-access-private"></span>

## 限制面板访问

设置 `--access-token-env` 后，查看历史记录的页面都要求登录。一分钟内输错十次令牌后，所有人都需要等这一分钟过去才能登录。

使用默认地址时，只有本机可以访问面板。如果用 `mash dash --listen` 换成其他地址且未配置登录，能访问面板的人就能读取历史。Mash 会在启动时提示这一点。`--allow-host` 需要配置令牌。

请优先使用私有网络。面板发布到互联网后，令牌保护所有对话历史。代你处理 HTTPS 连接的服务，例如 Cloudflare Tunnel，可以读取经过它的数据。

<!-- mash-docs-source: 34c4d23e51e6a4bbe1de4267f7f240413d4340148d117de79378275a88462e2f -->
