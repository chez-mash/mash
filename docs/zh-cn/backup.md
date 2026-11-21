# 备份和恢复历史记录

- **保存一份可以恢复的独立副本。** 升级或卸载 Mash 前，或者要把历史记录迁移到另一台电脑时，可以运行 [`mash export`](#make-a-portable-backup)。
- **保存对话文本供阅读或分享。** 使用[网页面板的导出功能](dashboard.md#export-many-chats-at-once)，即可获得能在 Mash 之外打开的文件。
- **日后在同一台电脑重装。** 使用 [`mash uninstall --keep-history`](uninstall.md#keep-history-for-a-reinstall)，在原处保留 Mash 中可搜索的对话副本及设置。需要独立副本时，请另外备份。

<span id="make-a-portable-backup"></span>

## 创建可移植备份

在私密位置选择一个新目录，与 [Mash 的数据目录](privacy.md#where-mash-stores-data)分开。请把示例路径替换成你电脑上的绝对路径。如果路径中有空格，请保留引号。

```sh
mash export "/absolute/path/to/new-backup"
```

结果会显示复制了多少对话、消息和保留的附件。请完整保存整个备份目录。它包含此电脑上的索引中的历史记录和已保存的记忆。如果使用共享索引，请在每台需要备份历史的电脑上分别执行导出命令。

**请妥善保管备份。** 备份包含已保存的对话文本，包括可能在搜索结果中被遮盖的凭据。配置、服务凭据和代理的原始历史文件需要单独保存。如果今后可能需要[升级或回退（英文）](../upgrades.md#upgrade-an-existing-installation)，请保留创建该备份的 Mash 版本。

<span id="restore-without-replacing-your-current-history"></span>

## 恢复副本，保留现有历史

请选择一个新的数据库路径。Mash 会拒绝导入到已有数据库。导入备份后，列出几条恢复的对话来检查副本。

```sh
mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/restored.sqlite3"
mash --db "/absolute/path/to/restored.sqlite3" list-chats --local --limit 5
```

导出、导入和列出本地对话都无需启动搜索服务。如果导入提示归档不兼容，请保留备份，并使用创建备份的 Mash 版本。如果要替换索引并以同一台电脑的身份继续发布，请按[升级流程（英文）](../upgrades.md#replace-an-index-that-cannot-be-upgraded)操作。

<span id="make-the-restored-chats-searchable"></span>

## 搜索已恢复的对话

启动[本地搜索服务](installation.md#start-the-local-search-service)，再根据恢复的副本构建搜索数据。可以搜索刚才列出的对话里记得的某个词。

```sh
mash --db "/absolute/path/to/restored.sqlite3" index --rebuild-local-views --full
mash --db "/absolute/path/to/restored.sqlite3" search 'a word you remember' --local --intent exact-mention
```

每次要使用恢复的副本时都请传入 `--db`。如果配置或模型放在其他位置，也请传入 `--config PATH` 或 `--models-dir PATH`。

<!-- mash-docs-source: 6e60fff6c2bbe05cb0dcefc8bc1c14ffc6d70d8d220650fe25fc9d0cc762e945 -->
