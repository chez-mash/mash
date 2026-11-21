# 历史记录与隐私

## 哪些内容留在你的电脑上

默认设置下，Mash 在本机建立索引和搜索，代理保存的原始历史文件保持原样。Mash 在你自己的电脑上运行，无需 Mash 账号。它会保留一份已建立索引的对话副本，其中可能含有敏感文字。[公开演示](https://getmash.io/demo/) 使用示例对话，无法访问你的本地历史记录。

默认情况下，在同一台电脑上运行的程序可以直接访问本地网页面板和 Mash 安装的搜索服务，无需登录。请在只由可信人员使用的电脑上采用此设置。若要从另一台设备打开面板，请使用[需要登录的网页面板](dashboard.md#open-it-from-your-phone-or-another-computer)。

如果代理将历史保存在 SQLite 数据库中（例如 OpenCode），Mash 读取数据库时，数据库旁可能会出现
`-wal` 和 `-shm` 文件。SQLite 在任何程序读取数据库时都可能创建这些文件，数据库内容保持原样。

<span id="where-mash-stores-data"></span>

## Mash 将数据存在哪里

默认情况下，已建立索引的对话保存在 `~/.mash/mash.sqlite3`，设置保存在
`~/.mash/config.toml`。在 Windows 上，`~` 指你的用户配置文件目录。Qdrant 会单独保存包括可搜索的对话文字在内的数据。在 Windows 上，Mash 为这些数据使用系统盘上的较短路径，通常是 `C:\mash-qdrant\storage`。
[各系统的搜索服务文件位置](local-search.md#where-qdrant-stores-data)

运行 `mash status` 可查看 `database_path`。如果 Qdrant 由 Mash 安装，还可查看
`local_qdrant.installed.storage`。运行 `mash uninstall` 可以预览要删除的内容，其中包括 Windows 上单独保存的 Qdrant 文件夹。准备好卸载后，再加上 `--yes`。

<span id="when-history-can-leave-your-machine"></span>

## 历史记录何时可能离开你的电脑

* **连接代理** 后，代理可以向 Mash 请求搜索结果和摘录。这些内容会进入代理的上下文，并可能发送给它的模型提供商。即使没有 MCP 连接，[插件钩子](plugin.md#what-happens-without-asking)也可以把摘录加入代理的上下文。如果只想用本地网页面板或终端搜索，请保持代理未连接，并停用或移除已安装的 Mash 插件。
* **跨设备共享** 会把索引中保留的对话和附件上传到你配置的 Mash 服务器。持有同组令牌的电脑都可以读取这些内容。[发布前查看会共享哪些内容](shared-index.md#before-sharing)。
* **远程向量生成服务** 会收到为搜索准备的文本，其中可能含有敏感历史记录。默认在本机生成文本向量，查询的向量始终在本机生成。
* **远程重排序服务** 会收到每次查询以及待排序的摘录。启用远程服务之前，重排序在本机进行。
* **网页面板助手** 会把你的问题以及出于回答需要而读取的对话内容和记忆发送给
  [你配置的模型服务](../configuration.md#the-dashboard-assistant)。助手只有在配置后才可使用。
* **从旧对话补录记忆** 会将选定的历史对话发送给你所选且已登录的代理的模型提供商。Mash 会在设置时询问你是否执行此操作。
  [了解如何补录记忆](../records.md#backfill-earlier-memory)

## 用少量本地历史试用

可以先[在演示中搜索示例对话](https://getmash.io/demo/#/search?q=invoice+exports+queue)。
如果想用自己的一份历史试用 Mash，请按以下步骤操作。

1. 在终端中运行[快速入门指南中的安装命令](quick-start.md)。安装程序会先启动本地搜索服务，再打开设置向导。
2. 如果向导先显示设置摘要，请选择 **Customize setup**。询问搜索位置时，请选择
   **Just this computer (recommended)**。只保留一份包含近期对话的历史，
   选择 **Custom number of days** 并输入 7。若向导询问是否连接代理或从旧对话中补录记忆，
   请取消选中所有代理并跳过补录。
3. 选择 **Set up and index**，然后运行 `mash dash`。在浏览器中打开显示的地址，
   选择 **chats**，搜索那段对话中的一个词。

如果想使用所有建议的历史，请按步骤[跳过代理连接](installation.md#skip-agent-connections)。你也可以[在浏览器中选择这些设置](dashboard.md#set-up-mash-from-the-dashboard)。

如果决定卸载，`mash uninstall` 会在你确认前列出要删除的内容。卸载时，代理的原始对话保持原样。
[查看卸载会删除什么](installation.md#uninstall)

## 敏感信息

默认情况下，Mash 会遮盖搜索结果和对外发送的共享历史中能够识别的密钥等凭据。自动识别可能遗漏敏感内容。分享前请自行检查。

本地历史和可移植备份中仍可能含有凭据。请像保管原始对话一样，将备份存放在私密位置。

同一消息中，被 `<private>…</private>` 包裹的文字不会写入 Mash 保存的消息文本。起止标签必须同时存在。这些文字仍保留在代理自己的文件里，附件中的内容也仍可见。

## 从 Mash 中移除一条对话

先找到这条对话的 `chat_id`，方法是运行 `mash list-chats` 或 `mash search`，再运行以下命令。

```sh
mash forget CHAT_ID
```

Mash 会从本机移除这条对话，并记住以后不要再导入它。启用共享后，Mash 也会尝试同步这次移除。请查看命令结果，确认是否还有操作待完成。

代理的原始文件、先前的备份和其他电脑独立建立的索引副本会保留。此操作在 Mash 中无法撤销。如果之后可能还要用到这条对话，请先
[备份](backup.md#make-a-portable-backup)。

<!-- mash-docs-source: deb468c83a6fb401e2a46e29ce24c4351d4c94ba681b84df0e4dcd24a592bb8b -->
