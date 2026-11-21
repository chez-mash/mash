# 使用 Mash 记忆

Mash 可以保存决定、偏好和事实，同时保留来自原始对话的引用。打开引用即可核对当时的上下文。

<span id="inspect-existing-records"></span>

## 查看保存的记忆

[打开演示中已保存的决定](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)，然后选择引用下方的 **open the message**。演示会跳转到讨论将刷新令牌存入设备钥匙串的原始消息。

要查看自己的历史记录，请运行 `mash dash`，打开命令显示的网址，然后选择 **memory**。打开一条记忆即可查看其来源、修订记录和关联的记忆。
[在网页面板中查找和筛选记忆](dashboard.md#inspect-conversation-records)

<span id="save-a-conclusion"></span>

## 保存结论

安装 [Mash 插件](plugin.md#start-with-the-plugin)后，可以在作出决定的那次对话中，请代理把决定保存下来，供以后的会话使用。

> 请记住，这个项目把刷新令牌存放在设备的钥匙串中。

在 Claude Code 中也可以使用 `/mash:remember`。在 Codex 或 Copilot 中，请代理使用 `remember` 技能。查看代理回复中保存的内容，再打开 **memory** 核对引用的上下文。

<span id="recall-records"></span>

## 找回记忆

以后在同一项目的对话中，可以问：“Mash 记得哪些关于刷新令牌的决定？给我看当时作出决定的消息。”在 Claude Code 中也可以使用 `/mash:recall refresh tokens`。

新会话开始或代理读取文件时，插件可以提供几条相关记忆。尚未保存为记忆的决定，代理也可以从过去的对话中搜索。[在代理中使用 Mash](agents.md#try-it)

<span id="how-saving-works"></span>

## Mash 如何保存记忆

Mash 会核对引用是否与原始对话一致，方便你查看来源。代理仍可能遗漏信息，或误解你的意思。

- 代理可以依据你的原话记录已经确定的决定和偏好。代理自己的其他解读可能作为**待确认记录**保存。
- 代理也可以保存自己作出的决定。你的决定优先。
- 如果已建立索引的对话发生变化，引用不再匹配，Mash 会标记来源已过时，常规回顾时会跳过它。

保存记忆使用当前运行的代理。插件可以在你工作时发现结论，也会提醒代理补存遗漏的内容。使用 Codex 时，请在 `/hooks` 中审核修改过的插件钩子以启用它们。
[插件会自动做什么](plugin.md#what-happens-without-asking)

设置 `MASH_RECORD_CAPTURE=off` 可以关闭自动保存，同时继续回顾记忆。设置 `MASH_RECORD_RECALL=off` 可以关闭自动回顾记忆，同时保留对话提示。
[插件设置](plugin.md#what-happens-without-asking)

<span id="correct-or-replace"></span>

## 更正或替换记忆

如果一条记忆有误，或者你改变了主意，请让代理根据你作出更正的对话更新记忆。可以用 `review` 技能检查待确认记录、过时的来源和相互冲突的决定。在 Claude Code 中可用 `/mash:review`。查看记忆不会改变它的状态。

你也可以让代理忘记一条记忆。这会删除它的修订记录和引用。如果你[从 Mash 中移除被引用的对话](privacy.md#remove-a-chat)，依赖该对话的记忆也会被删除。

<span id="backfill-earlier-memory"></span>

## 从旧对话中补录记忆

补录是可选的。它可以从旧对话中找出值得保存为记忆的内容。补录使用你已登录的 Codex 或 Claude Code，把选中的对话文字发送给该代理的服务并消耗其令牌额度。常规建立索引和 `mash init --yes` 不会启动补录。

你可以在交互式设置中选择补录最近三天内活跃的对话。要稍后检查已建立索引的对话，请运行：

```sh
mash init --yes --no-index --no-agents --backfill-memory codex
```

也可以把 `codex` 换成 `claude-code`。请先核对建议保存的记忆再使用。Mash 处理完一百条对话后，或遇到未能完成的审查时，会停止运行。每条对话最多处理十分钟。再次运行命令即可继续。只要已审查的对话没有发生变化，Mash 就会跳过它。

### 共享记忆

如果你将历史记录发布到[共享 Mash 服务器](shared-index.md)，就可以在网页面板中查看已发布的记忆。按本地项目路径保存的记忆只留在该设备上。
[查看共享记忆的来源](dashboard.md#inspect-conversation-records)

## 备份记忆

[可移植备份](backup.md#make-a-portable-backup)包含记忆、修订记录和引用的对话内容。请像保护原始对话一样妥善保管。

## 直接调用记忆工具

有关参数和示例，请参阅 [MCP 记忆工具（英文）](../record-tools.md)。

<!-- mash-docs-source: 36885e948f7746e5e39820ec03eac164d61f7874ac71b5ddbaf6c01efb02afdc -->
