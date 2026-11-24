# 対応するエージェント

以下のエージェントのチャットを、`mash dash` で開く
[ブラウザーのダッシュボード](dashboard.md)や [端末](search.md)から検索できます。

[エージェントを接続](agents.md#connect-claude-code-codex-or-copilot)すると、過去の作業を探すように頼めます。
接続は任意です。接続したエージェントは、ほかのエージェントのチャットも含め、Mash に読み込んだ履歴を検索できます。

[Mash をセットアップして最初のチャットを見つける](quick-start.md#quick-start)

| エージェント | 読み込めるチャット | `mash init` での接続 |
| --- | --- | --- |
| Claude Code | ローカルに保存された会話 | 対応 |
| Codex | ローカルに保存された会話 | Codex CLI 経由で対応 |
| GitHub Copilot CLI | ローカルのコマンドラインセッション | 開発ビルドで対応。[v0.1.1-alpha で手動接続する（英語）](../plugin.md#github-copilot-cli) |
| VS Code の GitHub Copilot Chat | VS Code がコンピューターに保存したチャット | — |
| OpenCode | ローカルデータベースの会話 | — |
| Cursor | コマンドラインのチャットと、対応形式で保存されたエディターのチャット | — |
| Antigravity | アプリや IDE がローカルデータベースに保存した会話 | — |

標準入出力でローカル MCP サーバーを使えるクライアントは、[手動で接続（英語）](../mcp.md#connect-another-mcp-client)できます。

<span id="if-a-chat-is-missing"></span>

## チャットが見つからないとき

`mash init` を実行すると、検出されたエージェントと読み込む履歴を確認できます。
セットアップ済みの場合は **Customize setup** から履歴の選択を変更できます。
履歴を別の場所に保存している場合は [設定ガイド（英語）](../configuration.md#choose-histories-and-an-age-limit)に沿って追加します。

新しく保存されたチャットを読み込むには `mash index` を実行します。
既定では過去 90 日間の履歴を読み込みます。[古いチャットを含める（英語）](../indexing.md#add-older-history) ·
[VS Code のリモートセッションの保存先（英語）](../indexing.md#copilot-in-vs-code-remote-windows)

<span id="tested-with"></span>

## 動作確認の範囲

以下の環境で履歴の読み込みと検索を確認しています。

| エージェント | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Claude Code | 日常利用 | 日常利用 | 少数の実例 |
| Codex | 日常利用 | 日常利用 | 少数の実例 |
| GitHub Copilot | 少数の実例 | 少数の実例 | 履歴データのみ |
| OpenCode | 履歴データのみ | 少数の実例 | CI で再現 |
| Antigravity | 履歴データのみ | 少数の実例 | 履歴データのみ |
| Cursor | 履歴データのみ | 一部 | 履歴データのみ |

- **日常利用**では、その環境にインストールしたエージェントの実際のチャットを数百件使っています。
- **少数の実例**では、インストールしたエージェントのチャットを 20 件未満使っています。
- **CI で再現**したテストでは、エージェントと同じ形式で履歴を書き込みながら、Mash が読み取ります。
- **一部**は、インストールした Cursor CLI のチャットを確認したものです。エディターの古い形式の一部には対応していません。
- **履歴データのみ**では、エージェントの形式に沿った記録でテストしています。その環境にインストールしたエージェントでの確認は今後の課題です。

[Linux CI](https://github.com/grep-stack/mash/actions/workflows/ci.yml)と
[Windows CI](https://github.com/grep-stack/mash/actions/workflows/windows.yml)はプッシュ時に実行します。
[Apple シリコンのビルドと基本動作の確認](https://github.com/grep-stack/mash/actions/workflows/macos-build.yml)は必要に応じて実行します。
チャットが表示されない場合や内容に問題がある場合は、エージェント名、バージョン、OS を添えて
[問題を報告](https://github.com/grep-stack/mash/issues)できます。

<!-- mash-docs-source: 103ac8fd4f6f882272bc1d643a5e8f893430be72a0b0625ebac257e648edca39 -->
