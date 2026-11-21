# 日本語など、英語以外のチャットを検索する

Mash は元の言語のままチャットを保存します。英語以外のチャットからメッセージを見つけるには、単語検索で、アクセント記号も含めてチャットに書かれているとおりに入力します。[ブラウザーのダッシュボード](dashboard.md#find-and-read-a-chat)で **chats** を開き、**Search your history** に入力します。既定は単語検索です。端末では、次のように指定できます。

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

デモでは、[スペイン語](https://getmash.io/demo/#/search?q=autenticaci%C3%B3n)や[フランス語](https://getmash.io/demo/#/search?q=corrig%C3%A9)のサンプルを検索できます。

<span id="chinese-and-japanese"></span>

## 中国語と日本語

Mash は、空白や句読点で区切られていない文字列を 1 つの単語として扱います。短い語で見つからない場合は、会話に出てくる連続した文字列をそのまま入力してください。以下の例は、サンプルの本文と同じ文字列を検索します。

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

デモで[中国語](https://getmash.io/demo/#/search?q=%E6%88%91%E4%BB%AC%E6%9B%B4%E6%96%B0%E4%BA%86%E8%BA%AB%E4%BB%BD%E9%AA%8C%E8%AF%81%E4%BB%A4%E7%89%8C)または[日本語](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F)の文字列を検索できます。

タイトルの一部を覚えている場合は、**チャットのタイトル**から検索できます。タイトル検索は文字列の途中でも一致します。チャットの最初のメッセージが、タイトルの冒頭に使われることもあります。[見つかったチャットを開く](search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

<span id="search-by-meaning"></span>

## 意味で検索する

オプションの[MiniLM 埋め込みモデル](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)は英語のテキスト向けです。意味で検索するときは英語で質問してください。同じ話題でも、別の言語で書かれたチャットは英語の質問で見つからない場合があります。目的のメッセージを探すには、元のチャットに出てくる言葉を使ってください。

ダッシュボードで検索ごとにモードを切り替えるには、**words** を[検索モードのメニュー](dashboard.md#choose-a-search-mode)で選び、**Apply filters** を押します。メニューが表示されない場合は、上記の単語検索コマンドを使ってください。

<!-- mash-docs-source: 5b106b1548f31cf83fd1530542f3bcd86db96597bd5ac3728288286ce9cfddc0 -->
