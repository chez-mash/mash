# 保持历史记录最新

Mash 为代理保存的历史记录建立索引后，就可以搜到新对话。想找刚完成的对话，请[立即更新](#refresh-now)。要在工作时持续更新，请[保持监听程序运行](#keep-refreshing-while-you-work)。

<span id="refresh-now"></span>

## 立即更新

在代理保存对话的电脑上运行以下命令。如果还未选择 Mash 可以读取的历史记录，请先[完成设置](installation.md#connect-your-histories)。

```bash
mash index
mash status
```

`mash index` 读取新增和变更的历史记录，并显示进度。查看最终结果，确认是否有未读完的记录或尚未完成的发布。按提示解决问题后，再运行 `mash index`。`mash status` 会显示本地搜索是否就绪。

索引更新后，网页面板中的对话列表会自动更新。要更新已打开的搜索，选择 **Search again**。如果面板中没有这个按钮，请重新加载页面，再次运行同一查询。

<span id="keep-refreshing-while-you-work"></span>

## 工作时持续更新

在另一个终端运行此命令，并保持终端打开：

```bash
mash index --watch
```

它会先建立一次索引，再在每一批历史变更稳定后更新索引。两次索引之间至少间隔 30 秒。如果上一次耗时较长，等待可能更久。按 Ctrl-C 会等待当前索引任务结束，再停止监听。关闭终端也会停止监听。

监听程序使用配置中的代理、时间范围和本地或共享目标。要纳入更早的历史记录：

```bash
mash index --watch --all-history
```

要长期修改时间范围，请编辑[配置（英文）](../configuration.md)。`--watch` 不能和 `--full` 或 `--reread` 同时使用。

升级 Mash 时，监听程序会完成当前索引任务，再用相同选项启动新版本的 `mash`。终端或服务可以继续监听。如果无法启动新版本，监听程序会报错退出，可以手动重新启动。

<span id="what-the-agent-plugin-does-automatically"></span>

## 插件会自动做什么

会话开始时，[Mash 插件](plugin.md)会在后台以低优先级更新索引，每十分钟最多启动一次，代理可以继续工作。如果已有索引任务在运行，这次尝试会退出，不会应用更改。

插件在会话开始时补上最新记录。要在长会话中持续更新，请使用监听程序。要关闭会话开始时的自动更新，请在代理环境中设置 `MASH_AUTO_INDEX=off`。

<span id="understand-freshness-warnings"></span>

## 理解索引更新提示

运行 `mash init` 或第一次运行 `mash index` 后，可以手动更新。搜索使用上次索引时读取的历史记录。如果旧版本显示“Live indexing is inactive”，表示下次索引时才会更新。

如果上次读取未完成，或者 Mash 无法读取索引状态，搜索仍可能返回较早的对话。按提示检查或更新索引。如果缺少近期对话，请运行 `mash index` 后再搜索。

- 如果需要持续更新，请保持监听程序运行，并查看其结果，确认所有历史记录来源都读取成功。
- 如果提示出现在升级后，或者提到索引不兼容，请先按[升级指南](upgrades.md)处理，再重试。

<span id="in-the-dashboard"></span>

## 网页面板中的更新

索引完成后，网页面板就能显示新的历史记录。阅读位置在对话底部时，面板会跟随新消息。在对话的其他位置，选择 **N new messages** 即可查看新消息。已经显示的旧消息会保留原样，直到你选择 **Show updated messages**。[网页面板如何更新](dashboard.md#keep-your-place)

<span id="across-machines"></span>

## 跨设备更新

如果缺少另一台电脑上的对话，请先检查那台电脑。它负责建立索引并发布共享副本。在那台电脑上运行 `mash status --shared` 查看发布状态。如果发布尚未完成，请恢复连接，再在那台电脑上运行 `mash index`。

[跨设备设置](shared-index.md)

<!-- mash-docs-source: 56a328baf0148ab0de7ea34a1c484a9c9c22f2c01f15fd990a45f1550dda4071 -->
