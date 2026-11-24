# 卸载 Mash

Mash 会保留各代理保存的原始对话。卸载时，可以删除 Mash 为搜索而保存的副本，其中也包括保存的记忆。

| 你想做什么 | 从哪里开始 |
| --- | --- |
| 移除 Mash 及其本地对话副本 | [预览卸载](#remove-mash-and-its-local-copy) |
| 日后在同一台电脑重装，并保留已建立索引的对话和记忆 | [原地保留 Mash 的副本](#keep-history-for-a-reinstall) |
| 另存一份副本，以便在本机或其他电脑上恢复 | [先创建可移植备份](backup.md#make-a-portable-backup) |

<span id="remove-mash-and-its-local-copy"></span>

## 移除 Mash 及其本地副本

先运行下面的命令查看预览。它只会列出将移除的内容，不会更改任何文件。

```sh
mash uninstall
```

核对哪些内容将移除、哪些会保留。预览也会显示 Mash 的对话副本占用的空间。如果报告以 JSON 显示，请查看 `paths`、`kept` 和 `next`。

确认无误后，关闭你在终端启动的网页面板和索引监视进程，再运行：

```sh
mash uninstall --yes
```

此命令会删除 Mash 的索引、设置、保存的令牌和下载的模型。Mash 会停止并移除它设置的服务，包括搜索服务及其数据。Mash 还会通过[已连接代理](agents.md#disconnect-an-agent)各自的命令，移除 Mash 搜索工具和插件。

重启仍在运行的已连接代理。如果报告中有步骤失败，请解决报告指出的问题，再以相同选项重试。卸载成功后，按最后一条指令移除程序本身。JSON 报告中的对应字段为 `binary.remove`。根据安装方式，可能需要执行 `brew uninstall mash`、`scoop uninstall mash` 或 `cargo uninstall mash`，也可能需要删除指定文件。

如果通过 apt 安装的是 `mash-cli` 软件包，请运行 `sudo apt remove mash-cli`。
Mash `v0.1.1-alpha` 的卸载报告可能会显示错误的软件包名称。

## 哪些副本会保留

- 各代理保存的原始对话和 `~/.mash` 中与 Mash 无关的文件会留在本机。
- 如果发布过对话到共享服务器，副本会留在服务器上。要删除该副本并撤销本机的访问权限，请[从共享索引中移除设备](shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again)。
- 如果你自行运行 Qdrant，请单独管理该服务和其中的 Mash 搜索数据。

如果只想移除 Mash 安装的搜索服务，请运行 `mash qdrant uninstall`。默认保留搜索数据，加上 `--delete-storage` 才会删除。可以用 `mash index --rebuild-local-views --full` 从本地索引重建这些数据。

<span id="keep-history-for-a-reinstall"></span>

## 为重装保留历史记录

如果以后会在同一台电脑上重装，并希望继续使用已建立索引的对话和保存的记忆，请使用 `--keep-history`。它会保留 Mash 的索引、设置、保存的令牌，以及 Mash 管理的搜索服务的数据。服务、代理连接和下载的模型仍会移除。

在预览和确认时都使用此选项。

```sh
mash uninstall --keep-history
```

在 `v0.1.1-alpha` 中，预览给出的确认命令可能会漏掉 `--keep-history`。
请使用下方的完整命令。仅用 `--yes` 确认会删除 Mash 已建立索引的对话和已保存的记忆。

```sh
mash uninstall --yes --keep-history
```

移除程序前，运行 `mash --version` 并记下版本号，以便重装。
如需重试失败步骤、重启代理或移除程序，请按照[移除 Mash 及其本地副本](#remove-mash-and-its-local-copy)中的步骤操作。

<span id="reopen-the-retained-history"></span>

### 重新打开保留的历史记录

1. 在同一台电脑上使用同一个用户账号，安装先前记下的 Mash 版本。
   运行[基础安装脚本](installation.md#recommended-release-installers)前，将 `MASH_VERSION` 设为对应的版本标签，例如 `v0.1.1-alpha`。
   如果想用压缩包安装，请在 [GitHub 发布页面](https://github.com/chez-mash/mash/releases)打开该版本标签，
   下载适用于你的系统的压缩包，以及同一版本的 `SHA256SUMS`。
   解压前先[校验压缩包](downloads.md#verify-the-checksum)。如果要安装更新的版本，请参阅[升级指南](upgrades.md#upgrade-an-existing-installation)。
2. 如果搜索服务原先由 Mash 安装，请运行 `mash qdrant install` 重新安装服务并复用保留的数据。如果原先是[自行运行 Qdrant](local-search.md#run-qdrant-yourself)，请按原来的方式启动它。
3. 运行 `mash dash` 并打开显示的网址，或在终端运行 `mash list-chats --local --limit 5` 查看保留的对话。Mash 会复用已保存的设置和索引。如果原先指定了自定义路径，请再次指定。

如果搜索结果需要更新，请参阅[保持历史记录最新](synchronization.md)。如果原先使用按含义搜索，请[重新安装所需模型](indexing.md#install-the-models)。也可以[重新连接代理](agents.md#connect-claude-code-codex-or-copilot)。

<!-- mash-docs-source: 78c676e9d7c8bc968b81510a879cb7d593ba3e7d150d2620a927882b98f7d57e -->
