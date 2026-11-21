# 搜索其他语言的对话

Mash 会保留对话的原文。要找到某条消息，请按对话中的写法搜索，包括重音符号。
在[网页面板](../dashboard.md#find-and-read-a-chat)中，打开 **chats** 并使用
**Search your history**。默认使用关键词搜索。也可以在终端中明确选择关键词搜索：

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

## 中文和日文

Mash 会把一串没有空格或标点分隔的字符当作一个词。如果搜索短词没有结果，请尝试输入对话中的完整片段。例如：

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

如果记得部分**对话标题**，可以直接搜索那段文字。标题搜索能找到连续字符中间的片段。
标题通常取自对话的第一条消息。[如何打开搜索结果（英文）](../search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## 按含义搜索

可选的 [MiniLM 嵌入模型](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
适用于英文文本。按含义搜索时，请用英文提问。英文问题可能找不到用其他语言讨论同一主题的对话。
要找到那条消息，请搜索对话原文中的词。

如果你将网页面板设为了按含义搜索，可用上面的关键词命令进行单次查找，也可以
[将默认模式改回关键词搜索（英文）](../configuration.md#search-defaults)。

<!-- mash-docs-source: 7fa76bcd3e4fa9d45ebe6ea8df3575b59a0c645f3d00f20a143478985adc5a58 -->
