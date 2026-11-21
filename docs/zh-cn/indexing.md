# 纳入较早的对话并按含义搜索

<span id="add-older-history"></span>

## 纳入更早的历史记录

首次设置时，Mash 读取代理在过去 90 天更新过的历史记录。最近更新的对话可能包含更早的消息。要纳入仍保存在磁盘上的更早对话：

```sh
mash index --all-history
```

如果只想回溯固定天数：

```sh
mash index --since-days 365
```

如果只想更新一个代理的历史记录：

```sh
mash index --provider codex
```

可用的代理名称是 `claude-code`、`codex`、`opencode`、`copilot`、`cursor` 和 `antigravity`。

建立索引时，本地搜索服务（Qdrant）必须在运行。Mash 会读取原始历史文件，但不会修改内容。请查看命令的最终输出，确认是否有错误。

这些选项只对本次运行生效。要长期调整时间范围，请参阅[配置（英文）](../configuration.md)。要持续更新，请参阅[如何保持历史记录最新（英文）](../synchronization.md)。

<span id="copilot-in-vs-code-remote-windows"></span>

### VS Code 远程窗口中的 Copilot

请在保存会话的设备上为相应的会话建立索引：

- Remote-SSH、容器或 Codespaces 窗口中的 **Copilot Chat** 会话保存在运行 VS Code 的电脑上。会话的项目是远程设备上的文件夹。
- 从远程窗口的对话视图启动的 **Copilot CLI** 会话在远程设备上运行，并保存在该设备的 `~/.copilot/session-state` 目录下。

如果 Mash 读取时 Copilot 仍在写入，Mash 会保留已索引的消息，并在下次更新时重新读取该会话。

<span id="enable-semantic-search"></span>

## 启用按含义搜索

按含义搜索也叫语义搜索。忘记原来的措辞时，可以直接用英文提问。按词或短语搜索无需模型，在你更改设置前仍是默认模式。要搜索其他语言的对话，请参阅[关键词搜索示例](search-languages.md)。

<span id="install-the-models"></span>

### 安装模型

```sh
mash models install
```

此命令会从 `https://huggingface.co` 下载约 177 MB 的文件到 `~/.mash/models`。Apple 芯片的 Mac 和 Windows 还会为 GPU 下载未量化的嵌入模型，约需额外 90 MB。可设置 `HF_ENDPOINT` 使用 Hugging Face 镜像，或通过 `--models-dir PATH` 选择其他目录。

<span id="build-search-data"></span>

### 为按含义搜索构建索引

如果是首次设置，请先安装模型，再运行 `mash init`。Mash 创建配置时会启用按含义搜索所需的索引，并在第一次索引时准备相关数据。[首次设置](installation.md#connect-your-histories)

如果已经在使用 Mash，请编辑现有[配置（英文）](../configuration.md)中的 `[views.build]` 表。可以打开网页面板的 **settings**，或在 `mash status` 的输出中查找 `configuration.path`，以确定文件位置。保留其他设置，仅在缺少该表时添加。

```toml
[views.build]
representations = "lexical-and-dense"
```

然后运行 `mash index`。第一次运行会为现有历史准备按含义搜索所需的数据，因此耗时较长。之后的运行会复用未变更对话的数据。

完成后运行 `mash status`。`realized_mode` 显示 `hybrid` 时，此设备上已可使用关键词搜索和按含义搜索。

<span id="try-a-question"></span>

### 试着提一个问题

```sh
mash search 'why did we choose a separate database' --intent related-discussion
```

<span id="use-search-by-meaning-by-default"></span>

### 默认按含义搜索

要在网页面板以及未指定模式的搜索中使用按含义搜索，请在 `[views.query]` 表中设置 `intent`。如果没有该表，可以添加。搜索数据准备好后，更改此默认值无需重新索引。

```toml
[views.query]
intent = "related-discussion"
```

即使默认按含义搜索，也可以通过 `--intent exact-mention` 指定按关键词搜索。

```sh
mash search 'auth_token' --intent exact-mention
```

遇到问题时，请参阅[故障排查](troubleshooting.md)。

<!-- mash-docs-source: 6724bc90be8c2bf0b6959aaeb8d9bf61cc977de3fcd373c007efa07572341d58 -->
