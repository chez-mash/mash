# コーディングエージェントの履歴ツールを比較する

Mash では、エージェントがすでに保存した会話を検索し、決定の根拠になった元のメッセージを開けます。
ブラウザー、端末、接続済みのエージェントから検索できます。

デモで [請求書のエクスポートに関するサンプル会話を検索](https://getmash.io/demo/#/search?q=invoice+exports+queue)し、
ワークステーションの検索結果を開くと、当時の話し合いを読めます。次に
[保存されたログインの決定](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)を開き、
**open the message** を選ぶと、根拠になったメッセージに移動します。

<span id="when-mash-fits"></span>

## Mash が向いている場面

- **すでに残しておきたい会話がある。** Mash は、対応するエージェントがディスクに保存した会話を読み取ります。
  単語やフレーズで検索し、結果から元の会話を開くと、前後のやり取りやツールの出力も読めます。
  ダッシュボード、端末、または接続したエージェントで使えます。[対応するエージェント](supported-agents.md)
- **保存した決定の根拠を確認したい。** プラグインを使うと、エージェントは、会話内で確認できた引用を添えて
  決定事項や作業上の好みを保存できます。ダッシュボードでは、引用されたメッセージを前後の文脈と一緒に読めます。
  [記録の保存方法（英語）](../records.md#how-saving-works)
- **複数のコンピューターで作業する。** どのコンピューターでも、そのコンピューターのインデックスをオフラインで検索できます。
  自分で運用する Mash サーバーに履歴を送って共有すると、ノートパソコンからワークステーションの会話も探せます。
  [複数のコンピューターで履歴を共有する（英語）](../shared-index.md)

Mash は Qdrant というローカル検索サービスを使います。インストーラーから起動できます。
単語での検索には、Mash のアカウント、モデルのダウンロード、エージェントの接続は不要です。
あとからローカルモデルを追加して意味で検索したり、エージェントを接続したりできます。
[インストール](installation.md) · [プライバシー](privacy.md) · [意味で検索する（英語）](../indexing.md#enable-semantic-search)

<span id="other-workflows"></span>

## ほかのツールが向いている場面

| やりたいこと | ツール | できること |
| --- | --- | --- |
| セッションを検索して再開する、または以前のファイル編集を復元する | Déjà Vu | ひとつの実行ファイルで保存済みの履歴にインデックスを作り、ローカルで検索できます。元のエージェントでセッションを再開したり、別のエージェントに作業を引き継いだり、記録された編集から以前のテキストを復元したりできます。[概要（英語）](https://github.com/vshulcz/deja-vu#overview) · [対話型検索（英語）](https://github.com/vshulcz/deja-vu#interactive-search) |
| さまざまなエージェントの保存済み履歴をキーボードで検索する | CASS | 端末で対話型検索ができ、スクリプトやエージェントに構造化された検索結果を返せます。[CASS リポジトリ（英語）](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| コードの行やコミットを元のエージェントのセッションまでたどる | ctx | ローカルに保存された履歴にインデックスを作り、検索できます。`ctx blame` は Git の履歴と記録されたエージェントの操作をもとに、セッション内の根拠を引用します。[検索（英語）](https://ctx.rs/search/) · [Blame（英語）](https://ctx.rs/blame/) |
| 作業の自動要約を次のエージェントセッションで参照する | Claude-Mem | エージェントのフックで操作を記録し、別のエージェントが要約します。次のセッションで関連するメモを読み込めます。[Claude-Mem の仕組み（英語）](https://github.com/thedotmack/claude-mem#how-it-works) |
| 会話をプロジェクト内に Markdown で保存する、またはエージェントで再開する | SpecStory | 拡張機能と CLI で、記録したセッションをプロジェクト内に Markdown で保存できます。CLI では、元のエージェントで再開したり、対応する別のエージェントで開けるようにセッションを再構成したりできます。[会話の保存（英語）](https://docs.specstory.com/specstory/features) · [検索と再開（英語）](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[Mash をインストールして、最初の会話を探す](quick-start.md)。

<!-- mash-docs-source: e46b86ce4739cba4e7caa3de5458a8a8928d9850fd953a45ddffc1f1d8788368 -->
