# 快速入门

[在演示中试搜](https://getmash.io/demo/#/search?q=invoice+exports+queue)，打开一条结果即可查看原始对话。要搜索自己的对话，请按下面的步骤安装 Mash。

运行命令前，你可以先 [查看安装脚本并校验下载文件](../installation.md#check-the-release-download)。如果只想先用一个代理的近期记录试用，请参阅 [本地试用指南](privacy.md#用少量本地历史试用)。

**让代理帮你设置。** 将以下提示粘贴到 Claude Code、Codex 或其他编程代理中。

> 请阅读 <https://getmash.io/setup.md>，并在这台电脑上配置 Mash。

如果已安装 [Mash 插件](../plugin.md#start-with-the-plugin)，可以让代理使用其中的 `setup` 技能。

**自己动手设置。** 首次在本机安装时，在终端中运行下面适用于你的系统的命令。它会安装 Mash 和本地搜索服务，然后请你在建立索引之前确认要读取哪些历史记录和连接哪些代理。

在搭载 Apple 芯片的 macOS 或 Linux（x86-64 或 ARM64）上运行以下命令。

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

在 Windows x64 上，安装程序会将搜索数据放在系统盘上路径较短的文件夹中，通常是
`C:\mash-qdrant\storage`。[Mash 将数据存在哪里](privacy.md#where-mash-stores-data)。请使用 PowerShell。

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

这些脚本会校验下载内容，并将 Mash 安装到你的 `.local/bin` 目录。Qdrant（约 30 MB）以你的用户身份运行，登录时自动启动。关键词搜索不需要下载模型。Linux 和 WSL 上的这种安装方式需要 systemd 用户会话。

默认读取最近 90 天的历史。核对设置后，选择 **Set up and index** 继续。
如果设置助手先询问在哪里搜索，请选择 **Just this computer (recommended)**。
如果 WSL 没有 systemd、你已运行自己的搜索服务，或你只想安装可执行文件，请参阅 [安装指南](../installation.md)。你也可以 [在网页面板中核对设置](../dashboard.md#set-up-mash-from-the-dashboard)。

<span id="try-it"></span>

## 查找第一条对话

设置完成并提示搜索已就绪后，启动网页面板，保持终端窗口打开。

```bash
mash dash
```

1. 在浏览器中打开命令显示的地址，通常是 <http://127.0.0.1:8790>。
2. 打开 **chats**，输入记得的关键词，例如报错文字或函数名，然后按回车。
3. 打开一项结果，即可在对话中查看原始消息。

如果终端找不到 `mash`，请使用安装时显示的完整路径。你也可以[将 Mash 添加到命令搜索路径（英文）](../troubleshooting.md#mash-command-not-found)。
如果找不到某条对话，请参阅[搜索无结果时该检查什么（英文）](../troubleshooting.md#search-returns-no-results)。
参阅 [网页面板指南](../dashboard.md#find-and-read-a-chat)，或了解如何
[在终端中搜索](../search.md#your-first-search)。

**让代理帮你搜索。** 重新启动在设置时连接的代理。例如，可以对 Codex 说：“请用 Mash 找出我们在 Claude Code 中对身份验证做过什么决定，并给我看作为依据的原始消息。”
[检查代理连接](../agents.md#confirm-the-connection)，确认代理能找到 Mash。

要让新对话也能被搜到，可以使用 [`mash index` 或后台监听程序](../synchronization.md)。你还可以 [导出并备份历史记录](../upgrades.md)。

Mash 的界面目前使用英文。你可以按原文查找中文。如果短词找不到，可尝试输入原文中由空格或标点分隔的完整片段。按含义搜索需要使用英文查询。[搜索指南（英文）](../search.md#search-in-other-languages)

<!-- mash-docs-source: 09d0e677e3d50b3f0916f91eff96b3e5660f3879b4a2a13f29cc7515dbf01781 -->
