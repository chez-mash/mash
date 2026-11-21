# 历史记录与隐私

## 哪些内容留在你的电脑上

默认设置下，Mash 在本机建立索引和搜索，代理保存的原始历史文件保持原样。Mash 在你自己的电脑上运行，无需 Mash 账号。它会保留一份已建立索引的对话副本，其中可能含有敏感文字。[公开演示](https://getmash.io/demo/) 使用示例对话，无法访问你的本地历史记录。

默认情况下，在同一台电脑上运行的程序可以直接访问本地网页面板和 Mash 安装的搜索服务，无需登录。请在只由可信人员使用的电脑上采用此设置。若要从另一台设备打开面板，请使用[需要登录的网页面板](dashboard.md#open-it-from-your-phone-or-another-computer)。

如果代理将历史保存在 SQLite 数据库中（例如 OpenCode），Mash 读取数据库时，数据库旁可能会出现
`-wal` 和 `-shm` 文件。SQLite 在任何程序读取数据库时都可能创建这些文件，数据库内容保持原样。

<span id="choose-agent-access"></span>

## 选择代理可以访问的内容

你可以直接在本地网页面板或终端搜索对话，无需连接代理。选择 Mash 读取哪些历史记录与选择连接哪些代理是两个独立的操作。

- **设置时**，取消选择所有代理即可跳过新的连接。现有的工具和插件仍会保留。[设置 Mash 时跳过代理连接](installation.md#skip-agent-connections)。
- **要移除现有集成**，请从代理中移除 Mash 工具，停用或卸载 Mash 插件，然后重启代理。单独安装的插件也需要处理。[移除代理连接](agents.md#disconnect-an-agent)。
- **要停止为某个来源建立索引**，打开 **settings**，在 **agent histories** 中关掉该来源，保存设置，然后重启正在运行的持续索引进程。已连接的代理仍保持连接，已索引的对话也仍然可以搜索。[从 Mash 中移除对话](#remove-a-chat)。

通过设置连接代理时，Mash 会注册工具并尝试安装插件。[连接代理时会安装什么](agents.md#what-the-connection-installs)。

<span id="when-history-can-leave-your-machine"></span>

## 历史记录何时可能离开你的电脑

| Mash 的使用方式 | 可以读取或发送的内容 |
| --- | --- |
| 在本地网页面板或终端搜索 | Mash 在你的电脑上查询索引并显示结果。 |
| 通过 MCP 将工具连接到代理 | 代理可以搜索已索引的对话，读取匹配的消息及其上下文，并可能将这些内容发给模型提供商。 |
| 启用钩子的 Mash 插件 | 会话开始时，插件可以向代理提供近期对话的概要信息和已保存的记忆。代理读取相关文件时，插件也可以加入对话摘录。即使未注册 MCP 连接，这些内容也可能被发送给模型提供商。[了解自动操作和停用方法](plugin.md#what-happens-without-asking)。 |
| 跨设备共享 | Mash 将已索引的对话和保留的附件上传到你的服务器。持有同组令牌的电脑可以读取这些内容。要停止后续上传，请打开 **settings**，在 **index into** 中选择 **this machine only**，保存设置，然后重启正在运行的持续索引进程。已上传的副本仍保留在服务器上。[查看会共享哪些内容](shared-index.md#before-sharing)。 |
| 网页面板中的 Mashy | 启用后，Mashy 会将你的问题、你选中的段落，以及它查询的对话文本和记忆发送给[你配置的模型服务（英文）](../configuration.md#the-dashboard-assistant)。要停止发送，请在 **settings** 中关闭 **Mashy** 并保存，或从配置文件中删除 `[assistant]` 部分。 |

- **远程向量生成服务**会收到为搜索准备的文本，其中可能含有敏感历史记录。默认在本机生成文本向量，查询的向量始终在本机生成。[向量生成（英文）](../configuration.md#embedding-inference)。
- **远程重排序服务**会收到每次查询以及待排序的摘录。默认情况下，重排序在本机进行。[远程重排序（英文）](../configuration.md#remote-reranking)。
- **从旧对话补录记忆**会将选定的历史对话发送给你所选且已登录的代理的模型提供商。Mash 会在设置时询问你是否执行此操作。[了解如何补录记忆](records.md#backfill-earlier-memory)。

<span id="where-mash-stores-data"></span>

## Mash 将数据存在哪里

默认情况下，已建立索引的对话保存在 `~/.mash/mash.sqlite3`，设置保存在
`~/.mash/config.toml`。在 Windows 上，`~` 指你的用户配置文件目录。Qdrant 会单独保存包括可搜索的对话文字在内的数据。在 Windows 上，Mash 为这些数据使用系统盘上的较短路径，通常是 `C:\mash-qdrant\storage`。
[各系统的搜索服务文件位置](local-search.md#where-qdrant-stores-data)

运行 `mash status` 可查看 `database_path`。如果 Qdrant 由 Mash 安装，还可查看
`local_qdrant.installed.storage`。运行 `mash uninstall` 可以预览要删除的内容，其中包括 Windows 上单独保存的 Qdrant 文件夹。准备好卸载后，再加上 `--yes`。

<span id="try-a-small-local-setup"></span>

## 用少量本地历史试用

可以先[在演示中搜索示例对话](https://getmash.io/demo/#/search?q=invoice+exports+queue)。
如果是全新安装，可以用自己的一份历史试用 Mash。

1. 运行[快速入门指南中的安装命令](quick-start.md#install-it-yourself)。在浏览器中打开显示的地址，并保持终端窗口打开。
2. 如果先看到设置摘要，请选择 **Customize setup**。从最近一周用过的代理中选一个，只保留它的历史记录，然后选择 **Next**。
3. 选择 **another number of days**，输入 **7**，筛选过去七天内更新的历史记录。Mash 根据对话中记录的活动时间筛选历史记录。如果没有活动时间，就按历史文件的最后修改时间筛选。纳入索引的对话可能包含更早的消息。选择 **Next**，关闭所有已选中的代理连接，然后进入设置摘要。
4. 确认摘要中显示 **this machine only** 和 **no agents**。选择 **Set up and index**。索引完成后，打开 **chats**，搜索那段对话中的一个词。

之后可以扩大日期范围或连接代理。[网页面板设置](dashboard.md#set-up-mash-from-the-dashboard)

如果决定卸载，`mash uninstall` 会在你确认前列出要删除的内容。卸载时，代理的原始对话保持原样。
[查看卸载会删除什么](uninstall.md)

<span id="shorten-the-history-range"></span>

## 缩短历史记录的时间范围

在网页面板中打开 **settings**。在 **history range** 下选择 **recent updates**（旧版本称为 **recent only**），输入天数。
选择 **Review changes**，再选择 **Save to file**。运行 `mash index` 以使用新的时间范围。
如果索引监听程序正在运行，也可以重启它，让它读取已保存的设置。

新的时间范围适用于之后的索引操作。比如，把范围从 90 天缩短到 7 天后，
即使再次运行索引，30 天前最后更新且已建立索引的对话仍然可以搜到。
最近更新的对话也可能含有更早的消息。要从 Mash 中移除某条对话，请参阅[从 Mash 中移除一条对话](#remove-a-chat)。

## 敏感信息

默认情况下，Mash 会遮盖搜索结果和对外发送的共享历史中能够识别的密钥等凭据。自动识别可能遗漏敏感内容。分享前请自行检查。

本地历史和可移植备份中仍可能含有凭据。请像保管原始对话一样，将备份存放在私密位置。

同一消息中，被 `<private>…</private>` 包裹的文字不会写入 Mash 保存的消息文本。起止标签必须同时存在。这些文字仍保留在代理自己的文件里，附件中的内容也仍可见。

<span id="remove-a-chat"></span>

## 从 Mash 中移除一条对话

先找到这条对话的 `chat_id`，方法是运行 `mash list-chats` 或 `mash search`，再运行以下命令。

```sh
mash forget CHAT_ID
```

Mash 会从本机移除这条对话以及引用过它的已保存记忆，并记住以后不要再导入这条对话。
启用共享后，Mash 也会尝试将这次移除同步到共享索引。请查看命令结果，确认是否还有操作待完成。

代理的原始文件、先前的备份和其他电脑独立建立的索引副本会保留。此操作在 Mash 中无法撤销。如果之后可能还要用到这条对话，请先
[备份](backup.md#make-a-portable-backup)。

<!-- mash-docs-source: 8ece8fc94aa9d9e86e4d6bb12c10445fe9ed0dbbac23a62ccb44e9bfd12a8496 -->
