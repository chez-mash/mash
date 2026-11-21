# エージェントと Mash を使う

Claude Code、Codex、GitHub Copilot CLI を接続すると、過去の作業を探すように頼めます。
接続したエージェントは Mash に読み込んだ履歴を検索でき、読んだ結果がエージェントのコンテキストに入ります。
[対応するエージェント](supported-agents.md) · [履歴とプライバシー](privacy.md)

マーケットプレースから Mash プラグインを入れた場合は、[エージェント内でセットアップを完了（英語）](../plugin.md#finish-setup-in-the-agent)してください。
ローカルの MCP サーバーを利用できる別のエージェントは、[手動で接続（英語）](../mcp.md#connect-another-mcp-client)できます。

<span id="connect-claude-code-codex-or-copilot"></span>

## セットアップ後にエージェントを接続する

初めて使う場合は [クイックスタート](quick-start.md#quick-start)に沿って、セットアップ中に接続するエージェントを選べます。
すでにチャットを検索できる場合は、履歴を読み込み直さずに接続できます。

<span id="in-the-browser"></span>

### ブラウザーで

1. **settings** から **Connect an agent** を選びます。お使いのバージョンで **go through setup** と表示されている場合は、そちらを選びます。
2. 履歴と日付の設定はそのままにして、**Choose agents to connect** から接続したいエージェントを選びます。
   古いバージョンでは **Customize setup** の中、または履歴の画面で **Next** を選んだ先に表示される場合があります。
   接続しないエージェントの選択は解除します。
3. **Set up without indexing** を選びます。接続したエージェントで新しいセッションを始め、[検索を試します](#try-it)。

Mash はダッシュボードを実行しているコンピューター上で、エージェントの CLI を探します。
CLI をインストールした直後や `PATH` に追加した直後は、その CLI が動く端末からダッシュボードを起動し直してください。
ダッシュボードが読み取り専用の場合は、そのコンピューターの端末で以下の手順を実行します。

<span id="in-a-terminal"></span>

### 端末で

履歴を読み込まない設定でセットアップウィザードを開きます。

```bash
mash init --no-index
```

1. **Use existing settings** を選んで、読み込む履歴をそのままにします。
2. **Which agents should get access to Mash search?** でスペースキーを使い、接続するエージェントだけを選びます。
   過去のチャットから記憶を作るか聞かれた場合は、作らずに進めることもできます。
3. **Set up without indexing** を選び、エージェントを再起動して接続を読み込ませます。[検索を試す](#try-it)

選択肢にエージェントがない場合は、同じ端末で `codex --version` などを実行し、CLI が動くことを確認します。
CLI をインストールするか `PATH` に追加し、上のセットアップコマンドを再実行してください。

開発ビルドは GitHub Copilot CLI のセットアップにも対応します。v0.1.1-alpha では
[Copilot CLI を手動で接続（英語）](../plugin.md#github-copilot-cli)してください。

<span id="what-the-connection-installs"></span>

### 接続時にインストールされるもの

Mash は選んだエージェントの CLI（`claude`、`codex`、`copilot`）を使って、以下の 2 つを追加します。

- **MCP サーバー**は、エージェントから呼び出せる Mash の検索ツールを提供します。
  MCP（Model Context Protocol）は、エージェントが外部ツールを呼び出す仕組みです。
- **[Mash プラグイン（英語）](../plugin.md)** には、検索する場面と手順を伝える `mash-history` スキル、
  頼んだ内容を [記憶（英語）](../records.md)に残す `remember` スキル（Claude Code では `/mash:remember`）、
  [新しいチャットを検索できるようにする](#keep-new-chats-searchable)フックが含まれます。

接続時に CLI が変更するユーザー設定は、Claude Code では `~/.claude.json`、`~/.claude/settings.json`、`~/.claude/plugins`、
Codex では `~/.codex/config.toml`、`~/.codex/plugins`、GitHub Copilot CLI では
`~/.copilot/mcp-config.json`、`~/.copilot/config.json`、`~/.copilot/installed-plugins` です。
Mash は接続したエージェントを自身の `config.toml` に記録します。あとで接続が切れた場合は
[`mash status --installation`](#confirm-the-connection) で確認できます。

プラグインのインストールにはネット接続が必要です。
フックには Python 3.9 以上（`python3`、`python`、または Windows の `py` ランチャー）が必要です。
プラグインをインストールできなくても、MCP サーバーが接続されていれば、頼んだときに検索できます。

登録先の Mash の実行ファイルが移動・変更された場合は、`mash init` を再実行すると登録を修復できます。

<span id="try-it"></span>

## エージェントに過去の作業を探してもらう

エージェントに次のように頼めます。

- 「Mash を使って、認証について決めたことを探してください。」
- 「このリポジトリのデプロイ方法について話したチャットを見つけてください。」
- 「先週は何をしましたか。根拠になったメッセージも見せてください。」
- 「このエラーについて以前話したチャットを探してください。」

エージェントは検索結果の元のメッセージを読み、チャットのタイトルや日付、リンクを出典として示します。
セッション ID やメッセージ ID が必要なら、表示するように頼んでください。

Claude Code でスキルを直接呼び出すには `/mash:mash-history` を使います。
Codex や Copilot では、プラグインの `mash-history` スキルを選ぶか、Mash を使うように頼みます。

<span id="confirm-the-connection"></span>

## 接続を確認する

接続したエージェントで新しいセッションを開始します。ダッシュボードで見つかるチャットを、
Mash で探すように頼んでください。エージェントが Mash を使えない場合は、登録を確認します。

```bash
mash status --installation
```

JSON の `summary.clients` に、エージェントごとの `connection` が表示されます。

| `connection` | 意味 |
| --- | --- |
| `full` | 対処は不要です。 |
| `mcp-only` | Mash のツールは接続済みです。プラグイン、スキル、またはフックを確認してください。 |
| `none` | 利用できる接続がありません。セットアップ時に接続を省いた可能性があります。 |

修復が必要な場合は `next_action` を確認してください。接続の修復にインデックスの再作成は不要です。
マーケットプレースのプラグインが CLI より新しい場合は、ツールの仕様の互換性が確認されます。
互換性があり、プラグインのファイルに問題がなければ、そのまま使えます。仕様が違う場合は、対応する CLI が必要です。

`unconfigured` の件数が表示され、`next_action: null` の場合は、エージェントの履歴は見つかっていますが接続はありません。
読み込んだチャットはダッシュボードや端末で検索できます。

`mash init` で接続したエージェント、Mash の設定にあるエージェント、または `--agent` で指定したエージェントに
問題がある場合、コマンドは結果を表示して終了コード 1 を返します。使う予定がないエージェントを接続する必要はありません。

エージェントを個別に調べ、新しいプロセスで MCP サーバーの起動も確認するには、次のように実行します。

```bash
mash status --installation --agent codex --probe
mash status --installation --agent claude-code --probe
```

対応するビルドでは `--agent copilot` も使えます。v0.1.1-alpha では [Copilot CLI 内で確認（英語）](../plugin.md#github-copilot-cli)してください。

確認に成功した場合は、新しいプロセスで接続が動作しています。セットアップ、修復、アップグレードのあとはエージェントを再起動してください。
プロジェクトの設定で接続が上書きされている可能性がある場合は、プロジェクトの外で実行してユーザー設定を確認し、
エージェント内でプロジェクトの接続設定を確認します。

<span id="keep-new-chats-searchable"></span>

## チャットの履歴を更新する

プラグインのフックは、エージェントの新しいセッションが始まるときに、最大 10 分に 1 回、バックグラウンドで履歴を読み込みます。
進行中の会話をリアルタイムで追う機能ではありません。無効にするには、エージェントの環境で `MASH_AUTO_INDEX=off` を設定します。

すぐに読み込むには `mash index` を実行します。変更を継続して読み込むには、別の端末で `mash index --watch` を実行しておきます。
[履歴の更新方法（英語）](../synchronization.md)

<span id="disconnect-an-agent"></span>

## エージェントの接続を解除する

接続を解除するエージェントに対応したコマンドを使います。`mash init` の結果では `undo` に表示されます。

```bash
claude mcp remove --scope user mash
claude plugin uninstall mash@mash
codex mcp remove mash
codex plugin remove mash@mash
copilot mcp remove mash
copilot plugin uninstall mash@mash
```

コンピューターから Mash も削除する場合は、[アンインストール（英語）](../uninstall.md)を参照してください。
Mash を残す場合は [`[agents] connected`（英語）](../configuration.md#connected-agents)からエージェントを外すと、
状態確認の対象からその接続が外れます。

<span id="other-mcp-clients-and-manual-setup"></span>

## ほかの MCP クライアントと手動接続

標準入出力で MCP サーバーを利用できるクライアントでは、`mash mcp` を実行できます。
必要な環境変数も含め、[MCP 設定例（英語）](../mcp.md#connect-another-mcp-client)を参照してください。
[接続のトラブルシューティング（英語）](../troubleshooting.md)

<!-- mash-docs-source: 9eefd5390c868ddde79cb2de929c06967a270e9fc40f875d4b1e1587129f6907 -->
