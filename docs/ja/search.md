# Mash でチャットを検索する

Mash では、このコンピューターや[共有インデックス（英語）](../shared-index.md)でつないだほかのコンピューターから、
[コーディングエージェントとの保存済みの会話](supported-agents.md)を探せます。
検索対象は、ユーザーとエージェントが書いたメッセージです。ツールの呼び出しや出力は、会話を開くと確認できます。

[ダッシュボードでチャットを探す](dashboard.md#find-and-read-a-chat)か、
[接続済みのエージェントに探してもらう](agents.md#try-it)こともできます。以下では端末での検索方法を紹介します。

<span id="your-first-search"></span>

## 最初の検索

[ローカル検索を設定した](installation.md#connect-your-histories)あと、チャットに出てきた特徴的な単語、ファイル名、
識別子などを入力して検索します。次の例では、結果を 3 件まで表示します。

```bash
mash search 'authentication token' --limit 3
```

結果は JSON で表示されます。チャットごとにまとめられ、一致度の高い順に並びます。
`title` にチャットのタイトル、`snippet` に一致したメッセージの抜粋が入ります。
メッセージの前後も読むには、その結果の `lookup.view.id` をコピーし、次のコマンドの `MESSAGE_ID` の代わりに指定してください。

```bash
mash view MESSAGE_ID --context 3
```

<details>
<summary>検索結果の例</summary>

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

検索の対象は、すでにインデックスに登録された履歴です。直前の会話を探すには、先に `mash index` を実行してください。

<span id="read-the-original-messages"></span>

## 元のメッセージを読む

`MESSAGE_ID` には、検索や `list-chats` の結果にある `lookup.view.id` を指定します。
`--context 3` は、指定したメッセージの前後を最大 3 件ずつ表示します。
該当するメッセージがなければ `not-found` が返ります。
チャット ID を渡した場合もエラーになり、代わりに使える先頭のメッセージ ID が表示されます。

ローカル履歴の出力には、チャットの ID（`chat_id`）とメッセージの総数（`message_count`）、
表示するメッセージ（`messages`）、前後で省略した件数（`omitted_before` と `omitted_after`）が含まれます。
省略があるときは `notice` に表示されます。続きを読むには `--context` の値を増やしてください。

別のコンピューターで保存されたチャットの検索結果には、履歴を共有したノード（コンピューター）の ID が `lookup.view.shared` に入ります。
その ID を `--shared` に渡します。

```bash
mash view MESSAGE_ID --shared NODE_ID --context 3
```

<span id="search-in-other-languages"></span>

## 英語以外のチャットを検索する

単語検索では、元のチャットに書かれているとおりに入力してください。
日本語や中国語の例や探し方は、[英語以外のチャットを検索する](search-languages.md)をご覧ください。
意味で検索するときは英語向けのモデルを使うため、英語で書かれたチャットの検索に適しています。

<span id="find-a-chat-by-title"></span>

## タイトルでチャットを探す

```bash
mash list-chats --title-contains 'authentication' --limit 10
```

このコマンドはチャットのタイトルだけを探します。モデルや検索用インデックスが使えない状態でも実行できます。
チャットのタイトルには最初のメッセージが使われます。名前を付けた Claude Code セッションの場合は、その名前が使われます。
変更した名前は、次にそのセッションをインデックスに登録したときに反映されます。

結果の `lookup.view.id` はチャットの先頭メッセージの ID です。その ID を指定すると最初から読めます。

```bash
mash view FIRST_MESSAGE_ID --context 10
```

特定のチャット内でメッセージを探すには、そのチャットの `chat_id` を渡し、結果をメッセージ単位で表示します。

```bash
mash search 'token' --chat CHAT_ID --unit messages
```

<span id="narrow-the-results"></span>

## 検索結果を絞り込む

| 覚えていること | コマンドの例 |
| --- | --- |
| 使ったエージェント | `mash search 'authentication' --provider codex` |
| おおよその時期 | `mash search 'authentication' --days 7` |
| 日付の範囲 | `mash search 'authentication' --after 2026-10-01 --before 2026-11-01` |
| プロジェクトのフォルダー | `mash search 'authentication' --project /absolute/path/to/project` |
| リポジトリ | `mash search 'authentication' --repository https://github.com/owner/repo.git` |
| 自分が書いたこと | `mash search 'authentication' --role user` |
| 正確な表記 | `mash search 'auth_token' --match-mode strict` |
| 正確なフレーズ | `mash search '"rotate the logs"'` |

異なるフィルターを組み合わせると、すべてを満たす結果が残ります。
同じフィルターを複数指定すると、そのいずれかに一致する結果が残ります。たとえば `--provider codex --provider claude-code` です。

`--project` と `--repository` には、記録された値と完全に同じものを指定してください。
`list-chats` や検索の結果にある `project` または `repository` の値をコピーします。

`--after` と `--before` は、日付（UTC の午前 0 時）または RFC3339 形式の日時を受け付けます。
`--after` は指定した時刻を含み、`--before` は含みません。
`list-chats` の同じオプションでは、Unix 時刻を秒で渡し、チャットの更新時刻で絞り込みます。

<span id="search-by-meaning"></span>

## 意味で検索する

既定では、Mash はモデルを使わずに単語で検索します。
[既定の検索方法（英語）](../configuration.md#search-defaults)を変更した場合でも、
`--intent exact-mention` を指定すると単語検索を選べます。

[意味で検索できるように設定した（英語）](../indexing.md#enable-semantic-search)ら、覚えている内容を自分の言葉で入力してください。
意味検索のクエリは現在、英語で入力する必要があります。

```bash
mash search 'why did we choose a separate database' --intent related-discussion
```

覚えている内容に合わせて、検索方法を選びます。

| | `exact-mention` | `related-discussion` |
| --- | --- | --- |
| 向いている検索 | 単語、識別子、ファイル名、エラーメッセージ | 元の文と表現が異なる質問や話題 |
| モデル | 不要 | 必要 |
| 検索語を複数入力した場合 | 順番を問わず、すべての単語が同じメッセージに含まれる必要がある | 検索語と同じ単語がメッセージに含まれていなくてもよい |

単語検索は入力した単語そのものを探します。「log rotation」で「rotate the application logs」を探したい場合は、
メッセージにある「rotate」を入力するか、意味で検索してください。
`auth_token` や `voyage-code-4` などは、表記どおり 1 語として扱います。
既定では単語の先頭部分でも一致し、小さなタイプミスも許容します。
完全に同じ表記だけを探すには `--match-mode strict` を使います。

<span id="search-for-an-exact-phrase"></span>

## フレーズの完全一致で検索する

シェルに渡す引用符の内側で、検索したいフレーズを二重引用符で囲みます。

```bash
mash search '"rotate the logs"'
```

引用符で囲むと、単語が同じ順序と表記で続けて現れる箇所を探します。
見つからなければ、内側の二重引用符を外して試してください。

<span id="more-results-and-local-only-searches"></span>

## 次の結果を見る、またはこのコンピューターだけを検索する

`--limit` を指定しない場合は、1 ページにつき 20 件の結果が表示されます。
続きを見るには、返された `next_offset` の値を `--offset` に指定して同じ検索を繰り返します。
`next_offset` が `null` になったら最後のページです。

`--sort recent` は、検索結果をページごとに新しい順に並べます。
チャットの一覧を時系列順で見るには `list-chats` を使います。

[共有インデックス（英語）](../shared-index.md)を使うと、このコンピューターの履歴と、
ほかのコンピューターが共有した履歴をまとめて検索できます。`--local` はこのコンピューターの履歴、
`--shared` は共有済みの履歴だけを対象にします。
`--node NODE_ID` はインデックス内の結果を、指定したコンピューターで記録されたものに絞り込みます。
この絞り込みのために、指定先のコンピューターに接続することはありません。

結果が出ない場合や、最新の履歴が検索できない場合は、[トラブルシューティング](troubleshooting.md)をご覧ください。
ほかのコマンドは [CLI コマンド一覧（英語）](../cli.md)を参照してください。

<!-- mash-docs-source: fe6be95f1cae06f55d2ce70951ed7c053b4a2ed4ad7f911f5d36d5fc9a56e42e -->
