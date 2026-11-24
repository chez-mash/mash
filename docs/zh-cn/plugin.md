# 适用于 Claude Code、Codex 和 Copilot 的 Mash 插件

使用 Mash 插件，Claude Code、Codex 或 GitHub Copilot CLI 可以在对话时查找以往的工作、按你的要求保存决定，以及打开 Mash 网页面板。插件添加了技能和钩子。Mash 的 [MCP 服务器](../mcp.md) 为代理提供这些技能所用的搜索工具。你也可以单独使用 MCP，直接让代理用 Mash 搜索。

* **已经在用 Mash 网页面板或终端？** 可以直接[连接代理](agents.md#connect-claude-code-codex-or-copilot)，无需重新建立索引。设置时会为连接的代理安装插件。
* **已经从代理的插件市场安装了插件？** [在代理中完成设置](#finish-setup-in-the-agent)。
* **第一次使用 Mash？** 按[快速入门](quick-start.md)操作，或[先安装插件](#start-with-the-plugin)，再让代理一步步带你完成设置。

<span id="install-update-and-remove"></span>

<span id="install-the-plugin"></span>

## 安装插件

<span id="requirements"></span>

安装需要联网。钩子需要 Python 3.9 或更高版本（`python3`、`python`，或 Windows 上的 `py` 启动器）。

<span id="start-with-the-plugin"></span>

### 从安装插件开始

在终端运行适用于你的代理的两条命令。如果已经添加了 Mash 插件市场，只需运行第二条。

#### Claude Code

```sh
claude plugin marketplace add grep-stack/mash
claude plugin install mash@mash
```

#### Codex

```sh
codex plugin marketplace add grep-stack/mash
codex plugin add mash@mash
```

<span id="install-in-github-copilot-cli"></span>

#### 在 GitHub Copilot CLI 中安装

```sh
copilot plugin marketplace add grep-stack/mash
copilot plugin install mash@mash
```

<span id="finish-setup-in-the-agent"></span>

### 在代理中完成设置

启动新会话。在 Claude Code 中，运行 `/mash:setup`。在 Codex 或 Copilot 中，说“用 Mash 的 `setup` 技能在这台电脑上设置 Mash”。代理会与你确认要安装的内容和要建立索引的对话，然后告诉你如何打开网页面板。是否连接代理由你决定。如果你使用 Mash v0.1.1-alpha 并选择连接 Copilot CLI，请在设置结束后[完成 Copilot 连接](#github-copilot-cli)。

如果你连接了代理，请重启它，然后说“用 Mash 找一段之前讨论身份验证的对话，并给我看答案所依据的消息”。请选择你之前讨论过的主题。要启用 Codex 钩子，请在 Codex 中打开 `/hooks`，查看 Mash 的钩子并将其设为受信任。如果代理找不到 Mash，请[检查连接](agents.md#confirm-the-connection)。

<span id="skills-and-commands"></span>

## 技能和命令

| 技能 | 在 Claude Code 中 | 在 Codex 中 | 功能 |
| --- | --- | --- | --- |
| `setup` | `/mash:setup` | 选择 `setup` 技能，或说“在这台电脑上设置 Mash” | 经你同意后安装 Mash，接入你的对话历史，并检查能否找到之前的对话。 |
| `remember` | `/mash:remember what to keep` | 选择 `remember` 技能，或说“记住这件事” | 以你的原话为依据，将决定、偏好或事实保存到 [记忆](records.md)中。如果命令后没有附加内容，就保存当前对话中最新的决定。 |
| `resume` | `/mash:resume` | 选择 `resume` 技能，或问“上次做到哪儿了？” | 读取本项目在所有代理和电脑上的近期对话，告诉你哪些工作正在进行、哪些事项还没解决，以及已经做了哪些决定。 |
| `handoff` | `/mash:handoff` | 选择 `handoff` 技能，或说“把这项工作交接出去” | 在对话中写一份会话简报，涵盖目标、已完成的事项、做了一半的工作、已作出的决定和下一步。 `resume` 可以在对话建立索引后，从任何代理或电脑找到这份简报。 |
| `recall` | `/mash:recall a topic` | 选择 `recall` 技能，或问“Mash 对……记得些什么？” | 显示某个主题或当前项目的 [记忆](records.md) 记录，并标明每条记录的来源。有记录需要审查时，也会提醒你。 |
| `review` | `/mash:review` | 选择 `review` 技能，或说“审查 Mash 的记忆” | 每次查看几条需要你决定如何处理的记忆记录，包括代理未经你确认就保存的记录、过时的记录和成对冲突的记录。你可以逐条选择保留、更正、撤回、删除或跳过。 |
| `forget` | `/mash:forget what to remove` | 选择 `forget` 技能，或明确要求忘记或删除 Mash 保存的某项内容 | 找到一条记忆记录或一段已建立索引的对话，明确显示将删除的内容，并在你确认后才删除。如果记录有误，也会提供更正选项。 |
| `standup` | `/mash:standup`，或 `/mash:standup this week` | 选择 `standup` 技能，或问“我昨天做了什么？” | 汇总你在一段时间内通过各个代理和电脑完成的工作，并按项目归类。如果命令后没有指定时间范围，就汇总上一个工作日的工作。 |
| `export` | `/mash:export`，或 `/mash:export a chat` | 选择 `export` 技能，或说“导出这段对话” | 在 [网页面板](dashboard.md#save-a-transcript) 中打开这段对话。你可以通过 **Export** 菜单将其保存为 Markdown、HTML 或 PDF。你也可以让代理直接写入 Markdown 文件。 |
| `status` | `/mash:status` | 选择 `status` 技能，或问“Mash 运行正常吗？” | 用通俗的语言说明索引、搜索、代理连接和其他电脑的状态。如果有问题，会指出下一步该做什么。 |
| `dashboard` | `/mash:dashboard` | 选择 `dashboard` 技能，或说“打开 Mash 网页面板” | 找到已运行的 [网页面板](dashboard.md)，或启动一个，然后在浏览器中打开。指定“这段对话”即可直接打开当前对话。 |
| `mash-history` | `/mash:mash-history`，或直接用自然语言提问 | 选择 `mash-history` 技能，或直接用自然语言提问 | 搜索你过去的对话，并在回答中附上作为依据的消息。当你提到以前的工作时，代理也会主动使用这项技能。 |

GitHub Copilot CLI 会加载相同的技能。你可以在其中直接说出技能名称，也可以用自然语言提出请求。

日常使用可以直接用自然语言提出请求，例如下面这些。

* “关于数据库迁移，我们做了什么决定？”
* “找到我们配置部署流程时的那段对话。”
* “记住，我们只从 `main` 分支部署。”
* “打开 Mash 网页面板。”
* “这个项目上次做到哪儿了？”
* “Mash 对部署记得些什么？”
* “关于预发布环境 URL 的那条记忆有误，请更正。”

保存后，代理会确认保存了什么内容。代理会拒绝保存凭据和令牌。删除是永久性的。`forget` 每次只处理一条记录或一段对话，并且总会先显示要删除的内容，再请你确认。

<span id="what-happens-without-asking"></span>

## 插件会自动执行哪些操作

| 触发条件 | 插件执行的操作 | 如何关闭 |
| --- | --- | --- |
| 会话开始时 | 向代理显示本项目已保存的记录和最近的对话，并在后台运行 `mash index`，每 10 分钟最多运行一次 | `MASH_AUTO_INDEX=off` 会停用插件的自动索引。`MASH_RECORD_RECALL=off` 会关闭会话提示和文件提示中的已保存记录。对话提示仍会显示。 |
| 你发送的消息提到以前的工作，或提到当前对话中还没出现过的内容 | 提醒代理先在 Mash 中搜索，避免猜测 | `MASH_PROMPT_RECALL=off` |
| 你表达了长期有效的决定或偏好 | 提醒代理如何保存这项决定或偏好。如果会话持续较长时间且一直没有保存内容，每 15 分钟最多提醒一次 | `MASH_RECORD_CAPTURE=off` |
| 代理读取项目中的文件时 | 显示与该文件有关的记录和以往对话，每个会话中每个文件只显示一次 | `MASH_FILE_RECALL=off` |

在 Copilot CLI 中，文件提示会在读取完成后立即随文件内容一起提供给代理。

在启动代理的环境中设置相应的开关。如果 Mash 未安装、尚未设置或响应缓慢，钩子会静默跳过，不会阻塞代理。钩子会调用你电脑上的 Mash。钩子在会话开始或读取文件时显示的摘录会进入代理的上下文，代理可能会将该上下文发送给模型提供商。已启用的钩子无需 MCP 连接即可运行。如果你只使用网页面板或终端，请[移除已安装的 Mash 插件](#update-or-remove-the-plugin)，并[断开代理连接](agents.md#disconnect-an-agent)。
[对话历史何时可能离开你的电脑](privacy.md#when-history-can-leave-your-machine)

<span id="update-or-remove-the-plugin"></span>

## 更新或移除插件

插件和 `mash` 程序一同发布。插件市场有时会在配套的二进制文件发布前先提供新版插件。运行
`mash status --installation` 可查看哪个组件需要更新。如果工具接口兼容，且其他安装检查均通过，新版插件可以配合当前的二进制文件使用。更新后请启动新会话。要移除插件，请运行适用于你的代理的命令。

```sh
claude plugin uninstall mash@mash
codex plugin remove mash@mash
copilot plugin uninstall mash@mash
```

如果还想移除搜索工具，请 [断开代理连接](agents.md#disconnect-an-agent)。

## GitHub Copilot CLI

开发版本支持通过 `mash init` 连接 GitHub Copilot CLI，前提是
`copilot` 位于你的 `PATH` 中。v0.1.1-alpha 版本可以读取 Copilot 保存的对话，你也可以手动授予 Copilot 使用 Mash 搜索的权限。[设置好 Mash](quick-start.md)后，在终端注册 Mash 的工具。

```sh
copilot mcp add mash -- mash mcp
```

如果 Copilot 找不到 `mash`，请将 `--` 后面的可执行文件替换为你的安装位置下该可执行文件的完整路径，并用引号将路径括起来。你也可以 [安装 Copilot 插件](#install-in-github-copilot-cli)
来使用技能和钩子。打开一个新的 Copilot 会话，让它用 Mash 查找一段之前的对话。`copilot mcp list` 会列出 `mash`，而 `copilot plugin list`
会列出 `mash@mash` （如果你安装了插件）。

如果使用 v0.1.1-alpha，请在卸载 Mash 前断开 Copilot 连接。如果你也安装了插件，请同时运行插件的卸载命令。

```sh
copilot plugin uninstall mash@mash
copilot mcp remove mash
```

* Copilot 将这些内容保存在 `~/.copilot` （或由 `COPILOT_HOME` 指定的目录）中。工具保存在 `mcp-config.json`中，已安装的插件保存在
  `config.json` 和 `installed-plugins`中。
* `mash init --backfill-memory` 不支持 Copilot。用这种方式审查旧对话会消耗你的高级请求额度，因此该功能只适用于 Claude Code 和 Codex。
* 要从 Copilot 会话保存内容到 [记忆](records.md) ，你需要使用能够读取 Copilot 当前会话的 `mash` 版本。如果 `mash --version-info` 在 `session_citations` 下列出 `copilot`，当前版本就支持读取 Copilot 会话。较旧版本的 `mash` 也能让插件查找记忆，但插件不会提示 Copilot 保存记忆。
* 在 Windows 上，Copilot 钩子通过 PowerShell 运行，并调用 `python`。目前还没有在 Windows 上验证这些钩子的记录。

<span id="check-that-it-is-working"></span>

## 检查插件是否正常运行

```sh
mash status --installation
```

每个代理的 `connection` 值为 `full` 表示工具和插件均已就绪，为 `mcp-only` 则表示只连接了工具。`next_action` 会在集成需要处理时给出下一步。对于搭配 v0.1.1-alpha 使用的 Copilot，请使用 [Copilot 自带的检查命令](#github-copilot-cli)。
[在你的代理中使用 Mash](agents.md#confirm-the-connection)

<!-- mash-docs-source: 1e02f9b9b161736cf7cca776c5a49c6df0689c3d5dab9868d0a3291d2d334be9 -->
