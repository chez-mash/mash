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

## Chinese and Japanese

Mash treats a run of characters without spaces or punctuation as one word.
If a short term comes back empty, try the full run from the chat. For
example, these queries match the sample messages as written:

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

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
