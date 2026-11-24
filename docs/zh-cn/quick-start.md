# 快速入门

<span id="quick-start"></span>

<span id="install-it-yourself"></span>

## 自行安装

运行适合你系统的命令，下载 Mash 并在浏览器中开始设置。你需要至少一条来自[受支持代理](supported-agents.md)的已保存对话。运行前可以了解[设置会更改什么](installation.md#what-setup-changes)和[如何校验下载文件](downloads.md#verify-the-checksum)。

**macOS（Apple 芯片）或 Linux（x86-64 或 ARM64）**

Linux 和 WSL 上需要 [systemd 用户会话](local-search.md#linux-without-a-systemd-user-session)。

```bash
curl -fsSL https://raw.githubusercontent.com/chez-mash/mash/main/install.sh | sh -s -- --dashboard
```

**Windows x64（PowerShell）**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/chez-mash/mash/main/install.ps1))) -Dashboard
```

打开安装程序显示的链接，保持终端窗口打开。按浏览器中的提示操作，然后选择 **Set up and index**。你可以跳过连接代理的步骤，或取消勾选已选中的代理。[查看网页面板的设置步骤](dashboard.md#set-up-mash-from-the-dashboard)

<span id="try-it"></span>
<span id="find-your-first-chat"></span>

## 查找第一条对话

搜索就绪后，打开 **chats**，输入记得的关键词，例如报错文字或函数名，然后按回车。打开一项结果，即可在对话中查看原始消息。

以后要重新打开网页面板，运行 `mash dash`，再打开命令显示的地址。
如果终端找不到 `mash`，请使用安装时显示的完整路径，或[将 Mash 添加到命令搜索路径](troubleshooting.md#mash-command-not-found)。
[在网页面板中查找和阅读对话](dashboard.md#find-and-read-a-chat) ·
[在终端中搜索](search.md#your-first-search) ·
[搜索无结果时该检查什么](troubleshooting.md#search-returns-no-results)

<span id="ask-your-agent"></span>

## 让代理帮你设置

将以下提示粘贴到 Claude Code、Codex 或其他编程代理中。

> 请阅读 <https://getmash.io/setup.md>，并在这台电脑上配置 Mash。开始前先和我核对设置选项。

如果已安装 [Mash 插件](plugin.md#start-with-the-plugin)，可以让代理使用其中的 `setup` 技能。如果是自行安装的 Mash，也可以[稍后再连接代理](agents.md#connect-claude-code-codex-or-copilot)。

如需使用软件包管理器、从源代码构建或在终端中设置，请参阅[安装指南](installation.md)。如果想先用较少的历史记录试用，请参阅 [本地试用指南](privacy.md#用少量本地历史试用)。

<!-- mash-docs-source: d57d2515612c6e32065f147541df1dc3c18acd5e701eed2c2fd739454faeaf4a -->
