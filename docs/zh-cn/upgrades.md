# 升级 Mash

升级 Mash 会保留设置和已建立索引的对话。先查看[版本说明（英文）](https://github.com/grep-stack/mash/releases)，做好备份，再用原来的安装方式升级。如果只想备份或找回某条对话，请参阅[创建可移植备份](backup.md#make-a-portable-backup)或[将备份恢复到单独的数据库](backup.md#restore-without-replacing-your-current-history)。

<span id="upgrade-an-existing-installation"></span>

## 升级现有安装

如果你跨设备发布历史记录，请先阅读[共享索引升级步骤](#upgrade-a-shared-index)。

1. **暂停索引更新并关闭已连接的代理。** 停止所有 `mash index --watch` 进程，等待正在进行的索引更新完成。如果网页面板是在终端里启动的，按 Ctrl-C 停止。
2. **用当前版本备份。** 保留旧版可执行文件或安装包，直到确认新版运行正常。把历史记录导出到一个可以妥善保管的新目录。

   ```sh
   mash --version
   mash export "/absolute/path/to/new-backup"
   ```

3. 用原来的方式[安装新版本](#install-the-new-version)。再次运行 `mash --version`，确认终端找到的是升级后的可执行文件。
4. **按版本要求升级 Qdrant。** 如果搜索服务是 Mash 安装的，再运行一次 `mash qdrant install` 即可升级并保留原有数据。如果你自行管理 Qdrant，请继续使用相同的存储位置。
5. **更新索引。** 如果数据库格式需要升级且在支持范围内，下面的命令会先升级格式，再更新索引。只读命令会提示需要升级格式。

   ```sh
   mash index
   ```

6. **检查搜索并重新打开 Mash。** 确认 `mash status` 中的 `"ready"` 为 `true`，然后用已收录对话中的一个词验证搜索。把示例中的文字换成你记得的词。

   ```sh
   mash status
   mash search 'a word you remember' --local --intent exact-mention
   ```

   重启已连接的代理和先前停止的索引监视进程。用 `mash dash` 重新打开本地网页面板。如果你使用[远程网页面板](dashboard.md#open-it-from-your-phone-or-another-computer)，请用原来的选项再次运行 `mash dash remote`，让服务使用新版程序。升级后如果代理找不到 Mash，请[检查插件和连接](agents.md#confirm-the-connection)。

Mash 会保留对话和消息的 ID，并复用兼容的搜索数据。索引更新也会纳入上次更新后保存的对话。

<span id="install-the-new-version"></span>

### 安装新版本

继续使用原来的安装方式。如果先前使用 shell 或 PowerShell 脚本，请重新运行[基础安装脚本](installation.md#recommended-release-installers)，并使用相同的安装目录。如果设置了 `MASH_VERSION`，请改成所需的版本。

| 安装方式 | 升级方法 |
| --- | --- |
| Homebrew | `brew update && brew upgrade grep-stack/tap/mash` |
| apt | `sudo apt update && sudo apt install --only-upgrade mash-cli` |
| Scoop | 先运行 `scoop update`，再运行 `scoop update mash`。 |
| mise | `mise upgrade github:grep-stack/mash` |
| cargo-binstall | 重新[安装预编译发布包](install-methods.md#cargo-binstall)。 |
| 源码 | [构建所需版本](install-methods.md#build-from-source)。如果使用网页面板，请一并构建网页文件。 |

如果安装的是最初名为 `mash` 的 Debian 软件包，请按 [apt 安装步骤](install-methods.md#apt-debian-or-ubuntu)改用 `mash-cli`。如果在 mise 中固定了版本，请改成你想安装的版本。

如果 `mash --version` 仍显示旧版本，请检查终端找到的是哪个可执行文件。在 macOS 或 Linux 上运行 `command -v mash`，在 PowerShell 中运行 `(Get-Command mash).Source`。请用新版可执行文件的路径重新检查。

<span id="if-the-index-format-changes"></span>

### 如果索引格式发生变化

`mash --version-info` 会显示新程序写入的索引格式（`silver_schema`）和它能升级的最旧格式（`silver_schema_min`）。如果 `mash index` 显示格式早于支持范围，请参阅[替换不能直接升级的索引](#replace-an-index-that-cannot-be-upgraded)。如果索引格式比程序更新，请使用写入索引的版本或兼容的新版本。重启仍在使用旧程序的代理、网页面板和索引监视进程。

**升级会删除已保存的未决问题。** 如果索引建于 Mash 停止保存这类记忆之前，升级会删除未决问题及其修订记录和指向解决方案的链接。决定、偏好和事实仍可使用。导出的备份会保留这些问题。如果以后还要读取它们，请保留生成导出文件的版本。

<span id="upgrade-a-shared-index"></span>

### 升级共享索引

如果你[跨设备共享历史记录](shared-index.md)，请把服务器和所有设备升到同一个版本。版本不一致时无法同步。保留搜索数据，等所有设备使用同一版本后再恢复同步。

升级前备份服务器的存储和配置。在各设备上运行 `mash export` 只会备份该设备的索引。按需升级服务器上的 Qdrant，然后启动新版 `mash-server`。它会在打开各设备的数据库时升级相应格式。等服务器和发布历史记录的设备都使用同一版本后，再恢复共享索引更新。

<span id="make-a-portable-backup"></span>

## 创建可移植备份

[将已建立索引的对话导出到私有目录](backup.md#make-a-portable-backup)。

<span id="restore-without-replacing-your-current-history"></span>

## 恢复备份并保留现有历史记录

[在单独的数据库中检查恢复出的副本](backup.md#restore-without-replacing-your-current-history)。

<span id="replace-an-index-that-cannot-be-upgraded"></span>

## 替换不能直接升级的索引

当 Mash 提示索引格式过旧时，请按以下步骤操作。常规索引更新无法升级这种格式。替换完成并确认可用前，请保留旧的可执行文件和索引。

1. 用写入索引的版本导出，再用新版把备份导入另一个路径。如果旧索引不在默认位置，导出时请加上 `--db PATH`。

   ```sh
   /path/to/old/mash export "/absolute/path/to/new-backup"
   /path/to/new/mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/staged.sqlite3" --keep-identity
   /path/to/new/mash --db "/absolute/path/to/staged.sqlite3" list-chats --local --limit 5
   ```

   某些旧备份也可能无法导入。如果导入失败，请保留原始数据和可执行文件。
2. `--keep-identity` 会保留这台设备在共享索引中的身份。切换前请关闭旧的网页面板、索引监视进程和代理连接。切换后只用替换后的索引发布历史记录。
3. 选定替换数据库的最终路径。可以直接使用导入时的路径。如果要沿用旧路径，请先把原来的数据库、对应的 `.attachments` 目录以及旁边的 `-wal`、`-shm` 或 `-journal` 文件移到安全位置。再把导入的数据库及其配套文件一起移动到最终位置。重命名 `staged.sqlite3.attachments` 和 SQLite 的配套文件，使它们与替换数据库的文件名一致。
4. 在最终路径重建搜索数据，确认能够搜索对话。请把示例中的路径换成最终路径。

   ```sh
   mash --db "/absolute/path/to/replacement.sqlite3" index --rebuild-local-views --full
   mash --db "/absolute/path/to/replacement.sqlite3" status
   mash --db "/absolute/path/to/replacement.sqlite3" search 'a word you remember' --local --intent exact-mention
   ```

5. 如果使用自定义路径，请继续传入 `--db`，也记得在代理的 MCP 设置中指定它。重启代理和索引监视进程。

<span id="other-changes-you-may-meet"></span>

## 可能遇到的其他变化

<span id="coming-from-qdrant-119"></span>

### 从 Qdrant 1.19 升级

Mash 需要 Qdrant 1.20 或更高版本。用新版 Mash 运行 `mash index` 前，请先停止 1.19 服务，再让 1.20 使用相同的存储数据。首次更新索引会在现有的搜索数据上添加关键词索引，并复用已有的表示向量和搜索数据。在此之前，搜索会提示搜索数据需要升级。在共享服务器上，新版 `mash-server` 会在启动时添加这个索引。

**不要用 Qdrant 1.19 打开 1.20 已经打开过的数据。** 如有可能回退，请先做好备份。

<span id="settings-that-are-no-longer-accepted"></span>

### 不再支持的设置

- 移除 `views.query.experimental_gpu_reranking`。重排由 CPU 完成，可继续使用原来的索引和模型。
- 旧版 `[embeddings]` 远程服务设置请改用[向量计算设置（英文）](../configuration.md#embedding-inference)。
- 从设备和服务器的配置文件中移除 `redaction.remote_embeddings`。

<span id="changing-the-embedding-model"></span>

### 更改语义搜索模型

升级后，原来本地模型生成的向量仍可使用。如果改用其他模型，请运行 `mash models install`，再重建本地搜索数据。

```sh
mash index --rebuild-local-views --full
```

如果使用共享索引，请同时更新服务器和所有设备的模型。在服务器上安装模型，再运行以下命令启动一次。

```sh
mash-server --server-config PATH --models-dir PATH --activate-views --replay-views
```

服务器会根据已有的历史记录重建搜索数据。请先备份。重建期间共享搜索无法使用。

<!-- mash-docs-source: f57dfe0421444d2aa8f7089a5af074b78ac73ec214071488c564847c13fb909c -->
