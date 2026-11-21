# 用 Mash 搜索对话

Mash 可以搜索本机和其他设备上由[编程代理保存的对话](supported-agents.md)。通过[共享索引](shared-index.md)连接其他设备后，就可以搜索它们的对话。搜索范围包括你和代理写的消息。打开对话可以查看工具调用及其输出。

你可以[在网页面板中查找和阅读对话](dashboard.md#find-and-read-a-chat)，也可以[让已连接的代理帮忙](agents.md#try-it)。本指南的示例使用终端。

<span id="your-first-search"></span>

## 进行第一次搜索

完成[本地搜索设置](installation.md#connect-your-histories)后，输入你记得的关键词、文件名或标识符。这个示例最多显示三条结果。

```bash
mash search 'authentication token' --limit 3
```

JSON 输出按对话分组，最相关的结果排在前面。找到对话的 `title` 字段和匹配消息的 `snippet` 字段。复制该消息的 `lookup.view.id` 值，用它替换 `MESSAGE_ID`，即可结合上下文阅读消息。

```bash
mash view MESSAGE_ID --context 3
```

<details>
<summary>搜索结果示例</summary>

```json
{
  "results": [
    {
      "chat_id": "01a1a77a-8105-7cf1-849b-37524f39e223",
      "provider": "codex",
      "title": "Add token authentication to the dashboard",
      "project": "/home/you/project",
      "matches": [
        {
          "lookup": { "view": { "id": "709e029507a46e56c1e1cf40" } },
          "role": "assistant",
          "snippet": "It's an authentication gate: the dashboard asks for a token…"
        }
      ]
    }
  ],
  "next_offset": null
}
```

</details>

搜索使用已有索引。如果要找刚结束的对话，请先运行 `mash index`。

<span id="read-the-original-messages"></span>

## 阅读原始消息

`MESSAGE_ID` 对应 `lookup.view.id` 字段，可以在搜索结果或 `list-chats` 的结果中找到。`--context 3` 会附带最多三条前面的消息和三条后面的消息。如果 ID 不对应任何消息，Mash 会返回 `not-found` 错误。如果传入的是对话 ID，错误信息会给出这段对话的第一条消息 ID。

查看本地历史时，输出会包含消息的 `chat_id`、对话的 `message_count`、消息正文 `messages`，以及前后各省略了多少条消息（`omitted_before` 和 `omitted_after`）。如果有消息省略，`notice` 字段会提示。要查看更多，可以增大 `--context` 的值。

来自其他设备的结果还有 `lookup.view.shared` 字段，记录了发布这份历史的设备 ID。用 `--shared` 传入该 ID。

```bash
mash view MESSAGE_ID --shared NODE_ID --context 3
```

<span id="search-in-other-languages"></span>

## 搜索其他语言的对话

按关键词搜索时，请使用原始对话里的写法。如需中文或日语的提示和示例，请参阅[搜索其他语言的对话](search-languages.md)。按含义搜索使用面向英语的模型，最适合英语对话。

<span id="find-a-chat-by-title"></span>

## 按标题查找对话

```bash
mash list-chats --title-contains 'authentication' --limit 10
```

这条命令直接列出对话，不读取消息来进行搜索。因此没有模型或可用的搜索索引时也能使用。标题通常来自第一条消息。对于已命名的 Claude Code 会话，标题使用会话名称。下次为会话建立索引时会显示新名称。

每段对话都提供第一条消息的 `lookup.view.id` 值，可以用它从头阅读对话。

```bash
mash view FIRST_MESSAGE_ID --context 10
```

如果要在某段对话内搜索，请提供它的 `chat_id` ，并选择按消息返回结果。

```bash
mash search 'token' --chat CHAT_ID --unit messages
```

<span id="narrow-the-results"></span>

## 缩小结果范围

| 你还记得…… | 示例                                                                            |
| ------ | ----------------------------------------------------------------------------- |
| 使用的代理  | `mash search 'authentication' --provider codex`                               |
| 大致时间   | `mash search 'authentication' --days 7`                                       |
| 日期范围   | `mash search 'authentication' --after 2026-10-01 --before 2026-11-01`         |
| 项目目录   | `mash search 'authentication' --project /absolute/path/to/project`            |
| 代码仓库   | `mash search 'authentication' --repository https://github.com/owner/repo.git` |
| 我说过的话  | `mash search 'authentication' --role user`                                    |
| 确切写法   | `mash search 'auth_token' --match-mode strict`                                |
| 确切短语   | `mash search '"rotate the logs"'`                                             |

多个筛选条件会同时生效，每项结果都必须满足所有条件。要在一个条件中接受多个值，可以重复使用同一个选项，例如 `--provider codex --provider claude-code`。

`--project` 和 `--repository` 必须与记录的值完全一致。可以复制结果里的 `project` 或 `repository` 字段。该字段出现在 `list-chats` 和搜索结果中。

`--after` 和 `--before` 接受日期（按 UTC 零点计算）或 RFC 3339 时间戳。 `--after` 包含边界时刻，`--before` 不包含。在 `list-chats` 中，这两个选项使用 Unix 秒数，并按对话活动时间筛选。

<span id="search-by-meaning"></span>

## 按含义搜索

Mash 默认按关键词搜索，无需模型。可以传入 `--intent exact-mention` 来明确选择关键词搜索，即使配置用了其他 [默认搜索模式（英文）](../configuration.md#search-defaults)。

在[启用按含义搜索（英文）](../indexing.md#enable-semantic-search)之后，可以用自己的话描述记得的内容。

```bash
mash search 'why did we choose a separate database' --intent related-discussion
```

目前，按含义搜索时请用英语提问。

根据你记得的内容选择搜索模式。

|      | `exact-mention`   | `related-discussion` |
| ---- | ----------------- | -------------------- |
| 适用于  | 关键词、标识符、文件名或报错文字  | 措辞可能不同的问题或想法         |
| 需要模型 | 否                 | 是                    |
| 多个词  | 所有词都必须出现在消息中，顺序不限 | 这些词可以都不在消息中          |

关键词搜索匹配输入的词。例如 “log rotation” 搜不到 “rotate the application logs”，可以试试 “rotate”，或者按含义搜索。像 `auth_token` 或 `voyage-code-4` 这样的单个词会按原样匹配，不会拆开。默认也支持前缀和少量拼写错误。 `--match-mode strict` 要求拼写完全一致。

<span id="search-for-an-exact-phrase"></span>

## 搜索确切短语

在 shell 的引号内，再用双引号括起要查找的短语。

```bash
mash search '"rotate the logs"'
```

用双引号括起来后，Mash 按原拼写匹配完整短语。如果没有结果，可以去掉双引号来分别匹配这些词。

<span id="more-results-and-local-only-searches"></span>

## 查看更多结果或只搜索本机

每页默认显示 20 条结果，可以用 `--limit` 设置其他数量。要看下一页，用 `--offset` 传入上一次返回的 `next_offset` 值，再运行同一条命令。如果该值为空，就没有下一页了。

`--sort recent` 会把每页相关结果中较新的放在前面。要按时间顺序列出对话，请用 `list-chats`。

连接[共享索引](shared-index.md)后，Mash 会搜索本机历史和其他设备发布的历史。 `--local` 只搜索本机。 `--shared` 只搜索已发布的历史。 `--node NODE_ID` 仅保留指定设备上记录的结果，无需连接到那台设备。

如果结果缺失或过旧，请参阅[故障排查指南](troubleshooting.md)。其他操作可以查看[常用命令（英文）](../cli.md)。

<!-- mash-docs-source: fe6be95f1cae06f55d2ce70951ed7c053b4a2ed4ad7f911f5d36d5fc9a56e42e -->
