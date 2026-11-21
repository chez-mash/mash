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
| 搜索并继续之前的会话，或恢复先前的文件编辑 | Déjà Vu | 一个可执行文件就能在本地为已保存的历史建立索引并搜索。它可以把工作交给另一个代理，也可以从记录的编辑中找回先前的文本。[Déjà Vu README](https://github.com/vshulcz/deja-vu#overview) |
| 用键盘搜索多种代理的对话历史 | CASS | 提供交互式终端搜索，并能为脚本或代理返回结构化结果。[CASS 仓库](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| 在新会话中使用自动生成的工作摘要 | Claude-Mem | 通过代理钩子记录活动，并由观察代理生成摘要。下一次会话可以加载相关笔记。[了解 Claude-Mem 的工作原理](https://github.com/thedotmack/claude-mem#how-it-works) |
| 将对话保存为项目中的 Markdown 文件，或在代理中继续会话 | SpecStory | 扩展和命令行工具会在你工作时写入 Markdown 文件。命令行工具可以重新打开已保存的会话。[保存对话](https://docs.specstory.com/specstory/features) · [搜索和继续会话](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[安装 Mash 并查找第一条对话](quick-start.md)。

<!-- mash-docs-source: 93cb840d39d49b21496d27385d3b09d3c74fe1f6c462801f5a43717fbbfbc6c9 -->
