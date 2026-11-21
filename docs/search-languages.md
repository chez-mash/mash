# Search chats in other languages

Mash keeps chats in their original language. To find a message in a
non-English chat, use word search and type a word as it appears in the chat,
including its accents. In the [browser dashboard](dashboard.md#find-and-read-a-chat),
open **chats** and use **Search your history**. Word search is the default.
From a terminal you can select it explicitly:

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

Try word search in the demo with sample chats in [Spanish](https://getmash.io/demo/#/search?q=autenticaci%C3%B3n) or [French](https://getmash.io/demo/#/search?q=corrig%C3%A9).

## Chinese and Japanese

Mash treats a run of characters without spaces or punctuation as one word.
If a short term comes back empty, try the full run from the chat. For
example, these queries match the sample messages as written:

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

Try a full run in [Chinese](https://getmash.io/demo/#/search?q=%E6%88%91%E4%BB%AC%E6%9B%B4%E6%96%B0%E4%BA%86%E8%BA%AB%E4%BB%BD%E9%AA%8C%E8%AF%81%E4%BB%A4%E7%89%8C) or [Japanese](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F) in the demo.

If you remember part of the **chat title**, search for that fragment
instead. Title lookup can find a substring, including one in the middle of
a run of characters. It looks in titles, which often start with the first
message in the chat. [How to open a match](search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## Search by meaning

The optional [MiniLM embedding model](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
is built for English text. Use English questions for search by meaning.
An English question may miss a chat about the same idea in another language.
Use words from the original chat when you need to find that message.

For a single dashboard query, choose **words** in the
[search mode menu](dashboard.md#choose-a-search-mode) and select **Apply filters**.
If your dashboard does not show the menu, use the word-search command above.
