# 比较编程代理对话历史工具

Mash 可以查找代理已经保存的对话，并打开记录决策依据的原始消息。你可以在浏览器、终端或已连接的代理中搜索。

在演示中，[搜索与发票导出有关的示例对话](https://getmash.io/demo/#/search?q=invoice+exports+queue)，
然后打开来自工作站的结果，阅读当时的讨论。接着打开[已保存的登录决策](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)，
选择 **open the message** 查看出处。

<span id="when-mash-fits"></span>

## 哪些情况适合用 Mash

- **你已经有值得保留的对话。** Mash 读取支持的代理保存在磁盘上的对话。搜索单词或短语，
  再打开匹配结果，就能阅读对应对话和工具输出。你可以使用网页面板、终端或自行选择连接的代理。
  [支持的代理](supported-agents.md)
- **你想核对已保存决策的依据。** 通过插件，代理可以保存决策和偏好，并附上经过核对的对话引文。
  网页面板可以打开引用的消息，让你阅读上下文。[Mash 如何保存记录](records.md#how-saving-works)
- **你在多台电脑上工作。** 每台电脑都可以离线搜索自己的索引。你也可以把历史发布到自己管理的
  Mash 服务器，从笔记本电脑上查找工作站里的对话。[了解共享方式](shared-index.md)

Mash 使用名为 Qdrant 的本地搜索服务。安装脚本可以在设置时启动该服务。按词搜索无需 Mash 账号，
也无需下载模型或连接代理。你可以随时添加本地模型来按含义搜索，也可以随时连接代理。
[安装](installation.md) · [隐私](privacy.md) · [按含义搜索](indexing.md#enable-semantic-search)

<span id="other-workflows"></span>

## 其他工作方式

| 如果你想… | 可以试试 | 工具提供什么 |
| --- | --- | --- |
| 搜索并继续之前的会话，或恢复先前的文件编辑 | Déjà Vu | 一个可执行文件就能在本地为已保存的历史建立索引并搜索。你可以在原代理中继续会话、把工作交给其他代理，或从记录的编辑中找回文本。[概览](https://github.com/vshulcz/deja-vu#overview) · [交互式搜索](https://github.com/vshulcz/deja-vu#interactive-search) |
| 用键盘搜索多种代理的对话历史 | CASS | 提供交互式终端搜索，并能为脚本或代理返回结构化结果。[CASS 仓库](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| 从一行代码或一次提交追溯到代理会话 | ctx | 对已保存在本机的历史建立索引，以便搜索。`ctx blame` 根据 Git 历史和已记录的代理活动，引用相应会话中的证据。[搜索](https://ctx.rs/search/) · [Blame](https://ctx.rs/blame/) |
| 在新会话中使用自动生成的工作摘要 | Claude-Mem | 通过代理钩子记录活动，并由观察代理生成摘要。下一次会话可以加载相关笔记。[了解 Claude-Mem 的工作原理](https://github.com/thedotmack/claude-mem#how-it-works) |
| 将对话保存为项目中的 Markdown 文件，或在代理中继续会话 | SpecStory | 扩展和命令行工具可以把已记录的会话保存为项目中的 Markdown 文件。命令行工具可以在原代理中重新打开会话，也可以重建会话，交给另一个受支持的代理。[保存对话](https://docs.specstory.com/specstory/features) · [搜索和继续会话](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[安装 Mash 并查找第一条对话](quick-start.md)。

<!-- mash-docs-source: e46b86ce4739cba4e7caa3de5458a8a8928d9850fd953a45ddffc1f1d8788368 -->
