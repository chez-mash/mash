# 搜索其他语言的对话

Mash 会保留对话的原文。要找到某条消息，请按对话中的写法搜索，包括重音符号。
在[网页面板](dashboard.md#find-and-read-a-chat)中，打开 **chats** 并使用
**Search your history**。默认使用关键词搜索。也可以在终端中明确选择关键词搜索：

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

你可以在演示中用[西班牙语](https://getmash.io/demo/#/search?q=autenticaci%C3%B3n)或[法语](https://getmash.io/demo/#/search?q=corrig%C3%A9)关键词搜索示例对话。

## 中文和日文

Mash 会把一串没有空格或标点分隔的字符当作一个词。如果搜索短词没有结果，请尝试输入对话中的完整片段。例如：

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

在演示中试试搜索一段完整的[中文](https://getmash.io/demo/#/search?q=%E6%88%91%E4%BB%AC%E6%9B%B4%E6%96%B0%E4%BA%86%E8%BA%AB%E4%BB%BD%E9%AA%8C%E8%AF%81%E4%BB%A4%E7%89%8C)或[日文](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F)。

如果记得部分**对话标题**，可以直接搜索那段文字。标题搜索能找到连续字符中间的片段。
标题通常取自对话的第一条消息。[如何打开搜索结果](search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## 按含义搜索

可选的 [MiniLM 嵌入模型](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
适用于英文文本。按含义搜索时，请用英文提问。英文问题可能找不到用其他语言讨论同一主题的对话。
要找到那条消息，请搜索对话原文中的词。

只想在网页面板中进行一次关键词搜索时，请在
[搜索模式菜单](dashboard.md#choose-a-search-mode)中选择 **words**，
再选择 **Apply filters**。如果面板中没有这个菜单，请使用上面的关键词搜索命令。

<!-- mash-docs-source: 5b106b1548f31cf83fd1530542f3bcd86db96597bd5ac3728288286ce9cfddc0 -->
