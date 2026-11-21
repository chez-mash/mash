# 在你的代理中使用 Mash

连接 Claude Code、Codex 或 GitHub Copilot CLI 后，就可以让代理帮你查找之前做过的工作。
连接的代理可以搜索你已建立索引的所有历史，它读取的结果会进入当前对话的上下文。
[支持的代理](supported-agents.md) · [Mash 如何处理对话历史](privacy.md)

如果已在代理的插件市场中安装了 Mash，请[在代理中完成设置](plugin.md#finish-setup-in-the-agent)。
如果使用其他支持本地 MCP 服务器的代理，请[手动连接](../mcp.md#connect-another-mcp-client)。

<span id="connect-claude-code-codex-or-copilot"></span>

## 设置后再连接代理

第一次使用 Mash 时，请按[快速入门](quick-start.md)操作，在提示时选择想连接的代理。
如果已经能在 Mash 中搜索对话，请在终端中运行以下命令，即可在无须重新索引的情况下连接代理。

```bash
mash init --no-index
```

1. 选择 **Use existing settings**，保留当前选定的对话历史。
2. 看到 **Which agents should get access to Mash search?** 时，按空格键切换勾选状态，只保留你想连接的代理。
   如果提示要从旧对话中提取记忆，可以跳过这一步。
3. 选择 **Set up without indexing**，然后重启代理以加载连接。[试着搜索一下](#try-it)。

如果列表中没有你的代理，请确认能否在同一个终端运行它的命令，例如 `codex --version`。
安装代理的命令行工具或将其添加到 `PATH` 后，再运行上述命令。

开发版提供 GitHub Copilot CLI 的引导设置。使用 v0.1.1-alpha 时，请[手动连接 Copilot CLI](plugin.md#github-copilot-cli)。

### 连接时会安装什么

Mash 会调用所选代理自己的命令（`claude`、`codex` 或 `copilot`）来安装两个组件。

- **MCP 服务器**向代理提供 Mash 搜索工具。MCP（Model Context Protocol，模型上下文协议）让代理可以调用外部工具。
- **[Mash 插件](plugin.md)**提供 `mash-history` 技能，告诉代理何时以及如何搜索。
  `remember` 技能可以按你的要求[保存记忆](../records.md)，在 Claude Code 中可用 `/mash:remember` 调用。
  插件也包含[自动更新对话历史](#keep-new-chats-searchable)的钩子。

这些命令会修改代理的用户设置。Claude Code 使用 `~/.claude.json`、
`~/.claude/settings.json` 和 `~/.claude/plugins`。
Codex 使用 `~/.codex/config.toml` 和 `~/.codex/plugins`。
GitHub Copilot CLI 使用 `~/.copilot/mcp-config.json`、`~/.copilot/config.json`
和 `~/.copilot/installed-plugins`。Mash 还会将连接的代理记录在自己的 `config.toml` 中，
以便用 [`mash status --installation`](#confirm-the-connection)发现后来失效的连接。

安装插件需要联网。插件的钩子需要 Python 3.9 或更高版本，可通过
`python3`、`python` 或 Windows 上的 `py` 启动。

如果插件安装失败，代理仍然可以在你提出要求时通过 MCP 服务器搜索。
插件的技能和钩子暂时无法使用。

如果连接指向的 Mash 可执行文件已被移走或替换，重新运行 `mash init` 即可修复。

<span id="try-it"></span>

## 让代理查找之前的工作

- “用 Mash 查查我们之前是怎么决定身份验证方案的。”
- “找到我们讨论如何部署这个仓库的那段对话。”
- “我们上周都做了什么？把相关的原始消息给我看看。”
- “找到之前讨论这个错误的对话。”

代理会搜索并阅读相关消息，回答时引用对话的标题、日期或链接。
如果需要会话或消息 ID，可以直接向代理索取。

在 Claude Code 中可以使用 `/mash:mash-history` 直接调用技能。
在 Codex 或 Copilot 中，可以选择插件的 `mash-history` 技能，或直接让代理使用 Mash。

<span id="confirm-the-connection"></span>

## 检查代理是否已连接

```bash
mash status --installation
```

JSON 报告的 `summary.clients` 会为每个代理列出 `connection` 状态。

| `connection` | 含义 |
| --- | --- |
| `full` | 连接正常。 |
| `mcp-only` | Mash 工具已连接。请检查插件、技能或钩子。 |
| `none` | Mash 没有与该代理建立可用连接。你可能在设置时跳过了它。 |

如果需要修复，`next_action` 会给出下一步。重新索引无法修复连接。
如果插件市场中的版本比本地 CLI 新，报告会检查两者声明的工具接口是否一致。
接口兼容且插件文件完整时可以继续使用。接口不同则需要安装与插件匹配的 CLI 版本。

如果报告显示有 `unconfigured` 项且 `next_action: null`，
说明 Mash 发现了未连接的代理的数据，但没有需要修复的项目。
你仍可在网页面板或终端搜索已建立索引的对话。

如果由 `mash init` 连接、为 Mash 配置或通过 `--agent` 指定的代理存在问题，
命令会打印报告并以状态码 1 退出。其他代理可以保持未连接状态。

要检查某个代理，并在新的进程里启动其 MCP 服务器，请运行对应命令。

```bash
mash status --installation --agent codex --probe
mash status --installation --agent claude-code --probe
```

在支持的版本中可使用 `--agent copilot`。使用 v0.1.1-alpha 时，请[在 Copilot CLI 中检查连接](plugin.md#github-copilot-cli)。

此检查验证的是新进程中的连接。设置、修复或升级完成后，请重启代理，让连接在对话中生效。

<span id="keep-new-chats-searchable"></span>

## 更新对话历史

插件的钩子会在代理会话开始时在后台建立索引，最多每十分钟运行一次。
如果想关闭它，请在代理的环境中设置 `MASH_AUTO_INDEX=off`。

运行 `mash index` 可以立即更新索引。在另一个终端中保持
`mash index --watch` 运行，可以持续更新索引。
[如何保持对话历史最新](../synchronization.md)

<span id="disconnect-an-agent"></span>

## 移除代理连接

运行对应代理的移除命令。`mash init` 会将这些命令显示在 `undo` 字段。

```bash
claude mcp remove --scope user mash
claude plugin uninstall mash@mash
codex mcp remove mash
codex plugin remove mash@mash
copilot mcp remove mash
copilot plugin uninstall mash@mash
```

如果还想从电脑上移除 Mash，请参阅[卸载说明](installation.md#uninstall)。
如果继续使用 Mash，请把该代理从 [`[agents] connected`](../configuration.md#connected-agents)
中移除，让 Mash 停止检查这条连接。

## 其他 MCP 客户端和手动设置

任何支持标准输入输出（stdio）MCP 服务器的客户端都可以运行 `mash mcp`。
请参照 [MCP 配置示例](../mcp.md#connect-another-mcp-client)，并按需设置环境变量。

[解决常见连接问题](troubleshooting.md#my-agent-cannot-find-mash)

<!-- mash-docs-source: d13297cbda95f0ed2a881c85c68f64a7582974147ff379a99ca977396ab0d265 -->
