# 继续之前的工作

打开之前的对话，找出接下来该做什么。在演示中，打开已保存的登录方案决策，然后选择 **open the message**。对话会显示采用短时有效的访问令牌这一决定，并列出移动端尚未完成的工作。 [用示例对话试一试](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)

<span id="find-the-next-step-in-your-chats"></span>

## 在对话中找到下一步

1. 在网页面板中打开 **chats**。搜索与这项工作相关的文件名、错误信息或短语。如果多个项目中都有类似的对话，可按项目筛选。
2. 打开一条搜索结果，阅读最后几条消息。看看哪些工作已经完成、你同意了哪些决定，以及还有哪些测试或任务没有完成。展开相关的工具输出，查看执行结果。
3. 你可以自己着手做下一步，也可以把简短的回顾带到新的代理会话中。写明目标、仍然适用的决定，以及剩余工作。

要查找相关对话，请参阅 [网页面板搜索](dashboard.md#find-and-read-a-chat)
和 [找不到对话时该检查什么](troubleshooting.md#search-returns-no-results)。

<span id="reopen-the-original-session"></span>

## 重新打开原会话

打开一条 Codex 或 Claude Code 的对话，选择 **Copy session ID**。在保存了原会话的电脑上，进入项目目录并打开终端，用复制的 ID 替换对应命令中的 `SESSION_ID`。

```sh
codex resume SESSION_ID
claude --resume SESSION_ID
```

如果代理无法用这个 ID 打开会话，可在代理的会话选择器中按标题查找，也可以新建会话并附上简短的回顾。

<span id="ask-an-agent-for-a-recap"></span>

## 请代理回顾进度

配置好 Mash 插件和搜索工具后，在要继续处理的项目中启动会话。如果使用 Claude Code，请运行 `/mash:resume`。如果使用 Codex 或 Copilot，可以说“请用 Mash 的 `resume` 技能告诉我这个项目上次做到哪儿了。”
[在代理中设置 Mash](plugin.md#finish-setup-in-the-agent)

该技能会尝试刷新索引，然后检查是否有交接说明，并读取项目最近的对话。它会概述之前做了什么、还有哪些工作没有完成，以及接下来该做什么。要核对某项决定，可以让它显示作出该决定时的原始消息。

<span id="leave-a-handoff-for-the-next-session"></span>

## 为下次会话留下交接说明

1. 结束会话前，可以在 Claude Code 中运行 `/mash:handoff`，也可以让你的代理使用 Mash 的 `handoff` 技能。该技能会在当前对话中写一份简短的交接说明。确认其中写明了涉及的文件或分支、已经确定的事项、未完成的工作和下一步。
2. 代理写完交接说明后，在保存这段对话的电脑上运行 `mash index`。下次回到这台电脑时，`resume` 技能也可以刷新索引。
3. 在下次会话中打开项目，并让代理使用 `resume`。你也可以在网页面板中搜索 `"Mash handoff"`，然后自行阅读交接说明。

如果要换电脑，请先设置共享，再在写下交接说明的电脑上更新索引，以便发布这段对话。若要手动交接，请将交接说明复制到新会话中。只分享你希望下一个代理看到的内容。 [跨电脑搜索](shared-index.md) · [代理能看到哪些内容](privacy.md#when-history-can-leave-your-machine)

<!-- mash-docs-source: 6248ed759dbb495e792ddf0cdb080fb54a4a6d188e6948c31407a5aed576c670 -->
