# 快速入门

[在演示中试搜](https://getmash.io/demo/#/search?q=invoice+exports+queue)，打开一条结果即可查看原始对话。按下面的步骤设置 Mash，即可在浏览器或终端中搜索自己的历史记录，也可以让你连接的代理帮忙搜索。

运行命令前，可以先了解[设置会更改什么](installation.md#what-setup-changes)以及[如何校验下载文件](installation.md#check-the-release-download)。如果只想先用一个代理的近期记录试用，请参阅 [本地试用指南](privacy.md#用少量本地历史试用)。

## 让代理帮你设置

将以下提示粘贴到 Claude Code、Codex 或其他编程代理中。

> 请阅读 <https://getmash.io/setup.md>，并在这台电脑上配置 Mash。

你可以先用网页面板，再决定是否连接代理。

如果已安装 [Mash 插件](plugin.md#start-with-the-plugin)，可以让代理使用其中的 `setup` 技能。

## 自行安装

你需要至少一条来自[受支持代理](supported-agents.md)的已保存对话。以下脚本会安装 Mash 和网页面板，启动本地搜索服务，并打开设置向导。如果想用 Homebrew、apt、Scoop 或其他软件包管理器，请参阅[软件包安装命令](install-methods.md#install-with-a-package-manager)。

在搭载 Apple 芯片的 macOS 或 Linux（x86-64 或 ARM64）上运行以下命令。在 Linux 或 WSL 上，此命令需要 systemd 用户会话来启动 Qdrant。[检查当前会话或查看其他启动方式](local-search.md#linux-without-a-systemd-user-session)。

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

在 Windows x64 上，安装程序会将搜索数据放在系统盘上路径较短的文件夹中，通常是
`C:\mash-qdrant\storage`。[Mash 将数据存在哪里](privacy.md#where-mash-stores-data)。请使用 PowerShell。

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

这些脚本会校验下载内容，并将 Mash 安装到你的 `.local/bin` 目录。Qdrant（约 30 MB）以你的用户身份运行，登录时自动启动。关键词搜索不需要下载模型。

开始前先核对要读取的历史记录。默认范围为最近 90 天。如果设置向导询问在哪里搜索，请选择 **Just this computer (recommended)**。

- **自己在网页面板中搜索。** [跳过代理连接](installation.md#skip-agent-connections)，为对话建立索引。之后也可以再连接代理。
- **让代理帮你搜索。** 选择 **Set up and index** 之前，先核对摘要中列出的代理。

如果你已运行自己的搜索服务，或只想安装可执行文件，请参阅 [安装指南](installation.md)。你也可以 [在网页面板中核对设置](dashboard.md#set-up-mash-from-the-dashboard)。

<span id="try-it"></span>

## 查找第一条对话

设置完成并提示搜索已就绪后，启动网页面板，保持终端窗口打开。

```bash
mash dash
```

1. 在浏览器中打开命令显示的地址，通常是 <http://127.0.0.1:8790>。
2. 打开 **chats**，输入记得的关键词，例如报错文字或函数名，然后按回车。
3. 打开一项结果，即可在对话中查看原始消息。

如果终端找不到 `mash`，请使用安装时显示的完整路径。你也可以[将 Mash 添加到命令搜索路径](troubleshooting.md#mash-command-not-found)。
如果找不到某条对话，请参阅[搜索无结果时该检查什么](troubleshooting.md#search-returns-no-results)。
参阅 [网页面板指南](dashboard.md#find-and-read-a-chat)，或了解如何
[在终端中搜索](search.md#your-first-search)。

**让代理帮你搜索。** 重新启动在设置时连接的代理。例如，可以对 Codex 说：“请用 Mash 找出我们在 Claude Code 中对身份验证做过什么决定，并给我看作为依据的原始消息。”
[检查代理连接](agents.md#confirm-the-connection)，确认代理能找到 Mash。
如果设置时跳过了连接，也可以[稍后再连接代理](agents.md#connect-claude-code-codex-or-copilot)。

要让新对话也能被搜到，可以使用 [`mash index` 或后台监听程序](../synchronization.md)。你还可以 [导出并备份历史记录](../upgrades.md)。

Mash 的界面目前使用英文。要按原文查找中文，请参阅[如何搜索其他语言的对话](search-languages.md)。

<!-- mash-docs-source: a9c68be8a1b7921e08e9b2c39d59ef4330baecf985e1d0aa9a3832de160133e8 -->
