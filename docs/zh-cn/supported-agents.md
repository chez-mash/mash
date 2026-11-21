# 支持的代理

运行 `mash dash`，在[网页面板](dashboard.md)中搜索已保存的对话，也可以在[终端](search.md)中搜索。
下面列出的历史都可以用这两种方式搜索。

你也可以连接代理，让它帮你查找之前做过的工作。
连接代理是可选的。连接后，代理可以搜索所有已建立索引的历史，包括其他代理的对话。
[代理连接指南](agents.md#connect-claude-code-codex-or-copilot)

[设置 Mash 并找到你的第一段对话](quick-start.md)。

| 代理 | Mash 可以读取的已保存对话 | 能否用 `mash init` 连接 |
| --- | --- | --- |
| Claude Code | 本机保存的 Claude Code 对话 | 可以 |
| Codex | 本机保存的 Codex 对话 | 可以，通过 Codex CLI |
| GitHub Copilot CLI | 本机保存的命令行会话 | 开发版支持。[v0.1.1-alpha 手动连接步骤](plugin.md#github-copilot-cli) |
| VS Code 中的 GitHub Copilot Chat | VS Code 保存在本机的对话 | — |
| OpenCode | OpenCode 本地数据库中的对话 | — |
| Cursor | 命令行对话，以及编辑器以受支持格式保存的对话 | — |
| Antigravity | 应用和 IDE 保存在本地数据库中的对话 | — |

如果你的客户端支持通过标准输入和输出来运行本地 MCP 服务器，请参阅
[手动连接指南（英文）](../mcp.md#connect-another-mcp-client)。

<span id="if-a-chat-is-missing"></span>

## 找不到某段对话

运行 `mash init`，查看 Mash 检测到了哪些代理、将读取哪些历史。如果已经设置过 Mash，请选择
**Customize setup** 来修改选择。如果历史保存在其他位置，请按照
[配置指南（英文）](../configuration.md#choose-histories-and-an-age-limit)添加该路径。

运行 `mash index` 读取新保存的对话。Mash 默认从最近 90 天开始建立索引。参阅
[如何纳入更早的对话](indexing.md#add-older-history)以及
[VS Code 远程会话的保存位置](indexing.md#copilot-in-vs-code-remote-windows)。

<span id="tested-with"></span>

## 各操作系统上的测试情况

下表列出了各代理历史的索引和搜索测试情况。

| 代理 | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Claude Code | 日常使用 | 日常使用 | 少量使用 |
| Codex | 日常使用 | 日常使用 | 少量使用 |
| GitHub Copilot | 少量使用 | 少量使用 | 仅用记录测试 |
| OpenCode | 仅用记录测试 | 少量使用 | CI 模拟测试 |
| Antigravity | 仅用记录测试 | 少量使用 | 仅用记录测试 |
| Cursor | 仅用记录测试 | 部分验证 | 仅用记录测试 |

- **日常使用**覆盖该系统中实际安装的代理所产生的数百段真实对话。
- **少量使用**覆盖实际安装的代理所产生的不到二十段真实对话。
- **CI 模拟测试**由自动化测试按代理的方式写入历史，同时让 Mash 建立索引。
- **部分验证**表示 Mash 已读取实际安装的 Cursor CLI 的对话。编辑器的部分旧格式尚不受支持。
- **仅用记录测试**表示使用符合代理格式的记录进行测试，还没有在该系统上用实际安装的代理验证。

每次推送代码都会运行 [Linux 测试](https://github.com/mashium/mash/actions/workflows/ci.yml)和
[Windows 测试](https://github.com/mashium/mash/actions/workflows/windows.yml)。
[Apple 芯片上的构建和基本运行检查](https://github.com/mashium/mash/actions/workflows/macos-build.yml)按需运行。
如果找不到对话，或显示内容有误，可以[提交问题](https://github.com/mashium/mash/issues)，
并注明代理名称、版本和操作系统。

<!-- mash-docs-source: 103ac8fd4f6f882272bc1d643a5e8f893430be72a0b0625ebac257e648edca39 -->
