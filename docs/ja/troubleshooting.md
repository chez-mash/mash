# トラブルシューティング

| 症状 | 最初に確認すること |
| --- | --- |
| 端末で `mash` が見つからない、または PowerShell がコマンドを認識しない | [インストール済みのコマンドを探す](#mash-command-not-found) |
| セットアップ中に検索サービスへ接続できない | [Qdrant を起動する](#qdrant-connection-failed) |
| ダッシュボードのアドレスを開けない | [ダッシュボードが起動しているか確認する](#the-dashboard-doesnt-open) |
| 履歴が見つからない、またはまだインデックスが作成されていない | [履歴の設定を確認する](#nothing-is-indexed-yet) |
| 最近のチャットが見つからない | [新しい履歴の読み込みを確認する](#new-chats-are-missing-or-search-is-behind) |
| チャットの一覧は見えるが、検索結果が出ない | [検索語、フィルター、対象期間を確認する](#search-returns-no-results) |
| コーディングエージェントが Mash を見つけられない | [エージェントの接続を確認する](#my-agent-cannot-find-mash) |
| アップグレード時にインデックスの形式が対応外だと表示される | [インデックスを復旧またはアップグレードする](#the-index-schema-is-older-or-newer-than-mash) |

解決できない場合やセキュリティ上の問題が気になる場合は、[サポートを受ける](#get-help)をご覧ください。

<span id="check-what-mash-sees"></span>

## Mash が検出した状態を確認する

原因が分からないときは、設定や履歴を変更しない診断コマンドを実行します。

```sh
mash doctor
```

`findings` で `"state": "problem"` の項目を探します。
`summary` に問題の内容が表示され、`repair` があれば修復に使うコマンドや手順が示されます。
問題が見つかると、コマンドの終了コードは 1 になります。
`warning` の項目も、必要に応じて確認してください。

設定の一部だけを確認したい場合は、目的に合ったコマンドを使います。

```sh
mash status                  # index and search readiness
mash status --installation   # history locations and agent connections
mash status --shared         # the index shared with your other machines
```

上から順に、インデックスと検索の準備状況、履歴の保存場所とエージェントの接続、
ほかのコンピューターと共有するインデックスの状態を確認できます。
`mash status --installation` はインデックスの準備ができていなくても実行できます。
エラーが出たら、`error.kind` の値や `error.message` の文言を手がかりに、以下の項目を確認してください。

<span id="mash-command-not-found"></span>

## 端末で `mash` が見つからない

**エラーの例** `mash: command not found`

インストーラーが表示したフルパスを使うか、開いている端末でインストール先を `PATH` に追加します。
リリース版を既定の場所にインストールした場合は、以下のコマンドを使えます。

```sh
export PATH="$HOME/.local/bin:$PATH"
mash --version
```

Windows の PowerShell では、開いているウィンドウで次のコマンドを実行します。

```powershell
$env:Path = "$HOME\.local\bin;$env:Path"
mash --version
```

別のフォルダーにインストールした場合は、`.local/bin` の代わりにその場所を指定してください。
`cargo install` の場合、通常はホームフォルダーの `.cargo/bin` にインストールされます。
`mash --version` でバージョンが表示されたら、同じ端末で[セットアップ](quick-start.md#install-it-yourself)を続けられます。
以降も使えるようにするには、シェルの設定ファイルか、Windows の「環境変数」にあるユーザー環境変数 **Path** に追加してください。
新しく開く端末に設定が反映されます。

<span id="the-dashboard-doesnt-open"></span>

## ダッシュボードを開けない

`mash dash` を起動したまま、表示されたアドレスを同じコンピューターのブラウザーで開きます。
別の端末ですでに Mash を起動しているなら、そちらに表示された URL を使います。
アドレスをほかのプログラムが使っている場合は、空いているローカルポートを Mash に選ばせます。

```sh
mash dash --listen 127.0.0.1:0
```

`0` を指定すると空いているポートが選ばれます。実際に表示されたアドレスを開いてください。
ポートを固定したい場合は、`127.0.0.1:8791` など、使われていないアドレスを指定します。

ダッシュボードがビルドされていないとページに表示された場合は、リリースのアーカイブからインストールするか、
[ソースからビルドするときにダッシュボードを含めてください](install-methods.md#include-the-web-dashboard)。

<span id="qdrant-connection-failed"></span>

## Qdrant に接続できない

`mash doctor` を実行し、`search-service` の項目で検索を妨げている原因を確認します。

`mash status` で `local_qdrant` が `rejected` の場合は、セットアップやインデックス作成をやり直す前に、表示された理由を確認してください。
ポートや API キーが間違っているか、古い Qdrant に接続している可能性があります。
HTTP と gRPC のアドレスは、同じ Qdrant に接続する必要があります。対応バージョンは 1.20 以上の 1.x 系です。
[接続先と API キーを指定する方法（英語）](../configuration.md#local-qdrant)をご覧ください。

`qdrant-unavailable`、`qdrant-install`、「Could not reach Qdrant at …」のいずれかが表示されるか、
`local_qdrant` が `unreachable` の場合は、サービスが起動しているか確認します。

Linux または WSL で、セットアップ中に `systemctl` のエラーが出た場合は、
[ユーザーセッションを確認して Qdrant の起動方法を選んでください](local-search.md#linux-without-a-systemd-user-session)。

インストール方法に合わせて Qdrant を起動します。

- **`mash qdrant install` で入れた場合**は、同じコマンドをもう一度実行します。停止したサービスを起動し、不足しているファイルを修復します。
  `mash status` の `local_qdrant.installed` でサービスの状態とログの確認方法が分かります。
  Linux では、たとえば `journalctl --user --unit mash-qdrant.service` でログを確認します。
- **Docker の場合**は、既存のコンテナーを起動してログを確認します。

  ```sh
  docker start mash-local-qdrant
  docker logs mash-local-qdrant
  ```

- **Qdrant を自分で用意した場合**は、元のフォルダーから起動し直し、端末に出るログを確認します。

Qdrant をまだ用意していなければ、[ローカル検索サービスを起動する手順](installation.md#start-the-local-search-service)に従ってください。
[稼働確認ページ](http://127.0.0.1:6333/readyz)に「all shards are ready」と表示されたら準備完了です。

`qdrant-failure` は Qdrant がエラーを返したことを示します。前述の方法でログを読み、
ストレージ、認証情報、スキーマに関するエラーがないか確認してください。

Qdrant が停止していても、チャットの一覧表示、メッセージの閲覧、バックアップのエクスポートはできます。
Mash のセットアップ中なら、ウィザードで **Save configuration only** を選ぶか、`mash init --no-index` を実行します。
続いて `mash index` を実行し、チャットをローカルに保存します。
このコマンドは `qdrant-unavailable` で終了しますが、保存できたチャットの件数を表示します。
保存したチャットは `mash list-chats` や `mash view` で確認できます。
Qdrant が起動したら `mash index` を再実行して、検索できるようにします。

WSL を使わず Windows 上で直接実行していて、Qdrant のログに `gridstore`、「path specified」、
`os error 3` が出る場合は、データのパスが長すぎる可能性があります。
[Docker を使わず Windows で設定する手順](local-search.md#native-windows-without-docker)にある、短い絶対パスを使います。
すでに運用している場合は、データを移動する前に Qdrant を停止し、バックアップを残してください。
最初からストレージを削除しないでください。設定のパスだけを変えると、別の空のストレージが作られます。
元のデータを移すには、別途移動が必要です。

<span id="nothing-is-indexed-yet"></span>

## まだインデックスが作成されていない

**エラー** `database-missing`

| メッセージ | 対処 |
| --- | --- |
| 「Mash is not set up on this machine」 | `mash init` |
| 「Mash is configured, but nothing is indexed yet」 | `mash index` |

「database does not exist」と表示された場合、初回は `mash init`、設定を保存済みなら `mash index` を実行します。
どちらの場合も、検索用のインデックスを作成するには Qdrant が起動している必要があります。

セットアップでコーディングエージェントのチャットが見つからない場合は、
このコンピューターで対応するエージェントとの会話を 1 件保存し、`mash init` をもう一度実行します。
すでにチャットがある場合は、`mash status --installation` で Mash が検出した履歴フォルダーを確認します。
[読み込む履歴の設定](installation.md#connect-your-histories)で、選択したエージェントとフォルダーも確認してください。
古いチャットが見つからない場合は、[対象期間を広げる手順（英語）](../indexing.md#add-older-history)をご覧ください。

<span id="my-agent-cannot-find-mash"></span>

## エージェントが Mash を見つけられない

使いたいエージェントの接続を確認します。Claude Code の場合は次のコマンドを実行します。

```sh
mash status --installation --agent claude-code
```

Codex の場合は `--agent codex` を指定します。Copilot CLI の場合は、
[Mash のバージョンに合った確認手順](agents.md#confirm-the-connection)をご覧ください。
診断結果の `next_action` に修復手順があれば、その指示に従ってください。
[エージェントの接続を設定する](agents.md#connect-claude-code-codex-or-copilot)間も、ダッシュボードで検索できます。

セットアップや修復の後、または実行ファイルやプラグインの更新後は、エージェントを再起動します。
開いたままのチャットでは、以前の接続が使われます。新しい接続を確認するには、次のコマンドを実行します。

```sh
mash status --installation --agent claude-code --probe
```

この確認で接続できた場合は、新しい MCP プロセスが正常に起動できています。
すでに開いているチャットの接続は、エージェントを再起動して確認します。
[エージェントと Mash を使う](agents.md)もご覧ください。

<span id="search-returns-no-results"></span>

## 検索結果が出ない

1. [検索モード](dashboard.md#choose-a-search-mode)を確認します。
   単語で検索する場合、入力した単語がすべて同じメッセージに含まれている必要があります。
   覚えている特徴的な単語を 1 つ試してください。
   意味で検索する場合は、英語の短い質問を試します。日本語のように単語を空白で区切らない言語は、
   [英語以外で検索する](search-languages.md)をご覧ください。
2. ダッシュボードにフィルターが表示されている場合は、**Clear filters** を選んで同じ検索をやり直します。
   チャット一覧が開いたら、検索語を入力し直します。端末では、フィルターのオプションを外して検索します。
3. チャットが一覧にあるか確認します。**Browse chats** を選ぶと、現在のフィルターを適用したチャット一覧が開きます。
   タイトルの一部を覚えていれば、検索欄に入力し、候補の **chats** を確認します。端末では以下のコマンドを使います。
4. 新しいチャットが見つからないか、会話の途中までしか表示されない場合は、
   [新しい履歴の読み込みを確認](#new-chats-are-missing-or-search-is-behind)してください。
   古いチャットの場合は [対象期間を広げます（英語）](../indexing.md#add-older-history)。
   特定のエージェントのチャットが見つからない場合は、[履歴フォルダーを確認](#nothing-is-indexed-yet)します。

検索対象はユーザーとアシスタントのメッセージです。
ツールの出力や推論の内容は会話を開くと読めます。前後のメッセージを検索し、そこから会話を開いてください。

端末でタイトルからチャットを探すには、次のコマンドを実行します。

```sh
mash list-chats --title-contains 'part of the title'
```

結果の `lookup.view.id` を `mash view` に渡すと、最初のメッセージから読めます。
チャット ID を指定して、そのチャット内だけを検索することもできます。

<span id="models-are-missing"></span>

## 検索モデルがインストールされていない

**メッセージ** 「semantic models are not installed」（`model`）、または
「related-discussion needs semantic search, which this configuration turns off」（`usage`）

意味での検索は任意で、モデルのダウンロードが必要です。
単語やフレーズはモデルを使わずに検索できます。

```sh
mash search 'auth_token' --intent exact-mention
```

意味で検索するには、[モデルをインストールし（英語）](../indexing.md#install-the-models)、
[検索用のデータを作成します（英語）](../indexing.md#build-search-data)。
モデルをインストールしても現在の設定は変わりません。すでに Mash を使っている場合は、
リンク先の手順に従って現在の設定で意味検索用のインデックス作成を有効にしてください。

<span id="embedding-failed-during-indexing"></span>

## インデックス作成中に埋め込みの計算に失敗した

**症状** `mash index` の結果に `embedding_error` が表示され、再試行が必要なステータスで終了する。

インデックス作成は完了しましたが、埋め込みを計算できませんでした。
モデルが未インストールか、別のマシンの埋め込みワーカーが応答しなかった可能性があります。
新しいメッセージは単語でならすぐに検索できます（`exact-mention`）。
意味で検索できるようにするには（`related-discussion`）、表示された原因を解消して `mash index` をもう一度実行します。

<span id="new-chats-are-missing-or-search-is-behind"></span>

## 最近のチャットが見つからない、または検索が最新の履歴に追いついていない

**症状** 最近の会話が表示されない、または `search-not-ready` が出る。

エージェントがチャットを保存したコンピューターで、次のコマンドを実行します。

```sh
mash index
```

インデックス作成が終わったら、ダッシュボードで **Search again** を選びます。
このボタンがなければ検索ページを再読み込みします。
別のコンピューターのチャットなら、[共有インデックスに反映されたか確認します](synchronization.md#across-machines)。

- 「search is catching up with new history」なら、インデックス作成中です。少し待ってから再試行します。
- 「search is behind the stored history and nothing is refreshing it」なら、`mash index` を実行します。

エージェント用プラグインは、セッション開始時に新しい履歴を読み込みます。
実行頻度は最大で 10 分に 1 回です。継続して更新する場合は、
[履歴を最新に保つ手順](synchronization.md)をご覧ください。

あるエージェントのチャットが一度も表示されていない場合は、
[Mash が読み込む履歴とその選び方](supported-agents.md#if-a-chat-is-missing)を確認してください。

<span id="a-chat-still-shows-what-an-older-release-read"></span>

## チャットに旧バージョンで読み込んだ内容が表示される

Mash は、前回読み込んでからファイルが変わっていないチャットを読み直しません。
アップグレードによってエージェントの履歴の読み取り方が変わった場合、たとえばチャットが属するフォルダーの判定方法が変わっても、
以前にインデックスに登録したチャットには、ファイルが更新されるまで旧バージョンで読み込んだ結果が表示されます。
すべての履歴を今すぐ読み直すには、次のコマンドを実行します。

```sh
mash index --reread
```

読み直しても内容が同じチャットには既存の検索データを使うため、初回のインデックス作成より短時間で済みます。
エージェントを 1 つに絞るには、たとえば `--provider codex` を追加します。

<span id="search-views-are-missing-or-stale"></span>

## 検索データが見つからない、または古い

対応する形式のデータベースに履歴が保存されていれば、その履歴からローカルの検索データを作り直せます。

```sh
mash index --rebuild-local-views --full
```

この操作で作り直すのは保存済みの履歴から作る検索データです。
エージェントの履歴ファイルは読み直さず、未取得の履歴の追加やスキーマの移行も行いません。
新しい履歴を取り込んだり、前回読み込みに失敗した履歴を読み直したりするには、`mash index` を実行し、終了時の結果を確認します。

<span id="the-index-schema-is-older-or-newer-than-mash"></span>

## インデックスの形式が Mash の対応範囲より古い、または新しい

**エラー** `schema-incompatible`

作業を止めて、[アップグレードとバックアップの手順（英語）](../upgrades.md)に従ってください。
古い実行ファイルとデータを残しておきます。
インデックスを書き込んだバージョンでエクスポートし、新しいバージョンでは別のパスにインポートして確認します。

データベースの削除やスキーマ番号の変更は行わないでください。`index --full` では移行できません。
アップグレードの途中であることを示すマーカーがある場合は、新しいインデックスを作る前に、途中のアップグレードを復旧するか完了させてください。

<span id="indexing-is-busy-incomplete-or-shared-pending"></span>

## インデックスの作成が実行中、未完了、または共有待ち

- **実行中**（`index-busy`、「another Mash operation is updating this local index」）は、別の処理が書き込み中です。
  完了を待つか、自分が起動した `mash index --watch` を停止してから再試行します。ロックファイルは削除しないでください。
- **未完了**の場合は、表示された履歴の読み込みエラーを確認して修復し、`mash index` をもう一度実行します。
  保存済みの履歴は残ります。
- **共有待ち**の場合も、ローカルでのインデックス作成は完了していることがあります。
  接続とトークンの設定を確認し、`mash status --shared` で状態を調べてから、`mash index` をもう一度実行します。

複数のコンピューターを対象に検索すると、ローカルの検索結果と一緒に `shared_error` が返される場合があります。
手元だけを検索するには `--local` を使います。すべてのコンピューターを対象にするには、
共有先への接続を修復してから結果を確認してください。

<span id="get-help"></span>

## サポートを受ける

インストール、セットアップ、日常の使用で問題が起きたら、[GitHub で不具合を報告します](https://github.com/chez-mash/mash/issues/new?template=bug_report.yml)。
OS、インストール方法、失敗した手順、期待していた動作を記載してください。
`mash --version` が実行できる場合は、その出力も添えます。
インストールできなかった場合は、インストーラーのエラーが分かれば報告できます。

GitHub Issues の内容は公開されます。関係のあるエラーか、[`mash doctor` の出力](#check-what-mash-sees)の短い抜粋だけを載せてください。
非公開のファイルパスやアドレス、チャットの本文、認証情報は削除します。例が必要なら架空のチャットを使ってください。
[Mash が履歴をどう扱うか](privacy.md)も確認できます。

非公開の履歴が見える、サインインなしでアクセスできる、配布パッケージのチェックサムが一致しないなどの問題が疑われる場合は、
[GitHub の非公開の報告フォーム](https://github.com/chez-mash/mash/security/advisories/new)を使ってください。
[セキュリティ報告に含める内容（英語）](../../SECURITY.md#report-a-vulnerability)をご覧ください。

<!-- mash-docs-source: a73203b99483a5a34c28c3271033920105964cfdc7a39ddc01fb5d725de247fb -->
