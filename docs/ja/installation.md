# Mash をインストールする

Mash をインストールし、検索サービスを起動して、どのチャットを検索できるようにするかを選びます。
既定では、お使いのコンピューターで履歴の読み込みと検索を行います。[Mash の履歴の取り扱い](privacy.md)

インストール前に試すなら、[デモでサンプルのチャットを検索](https://getmash.io/demo/)できます。
自分のチャットで試すなら、[履歴を 1 つ選んでローカルのダッシュボードを使ってみてください](privacy.md#try-a-small-local-setup)。

コマンド 1 つで Mash のインストール、検索サービスの起動、ブラウザーでのセットアップまで進めるには、[クイックスタート](quick-start.md)をご覧ください。
すでに Mash をインストールしている場合は、[検索サービスを起動](#start-the-local-search-service)します。
サービスが動いていれば、[読み込む履歴の選択](#connect-your-histories)に進めます。

すでに Mash を使っていて新しいリリースに移行する場合は、[アップグレード手順（英語）](../upgrades.md#upgrade-an-existing-installation)をご覧ください。

<span id="what-setup-changes"></span>

## セットアップで変わること

1. **プログラムをインストールします。** [リリースのインストーラー](#recommended-release-installers)は `mash` と `mash-server` をユーザー用のフォルダーに入れます。[リリースのアーカイブを検証して展開](downloads.md)する方法もあります。
2. **ローカル検索サービスを起動します。** `mash qdrant install` は Qdrant をダウンロードし、自分のアカウントでログイン時に起動するように設定します。Qdrant はこのコンピューターからの接続だけを受け付けます。Mash は Qdrant の使用状況の送信も無効にします。[検索サービスのファイル](local-search.md#where-qdrant-stores-data)
3. **Mash が読み込む履歴を選びます。** ダッシュボードでも端末でも、設定の保存やチャットの読み込みの前に、Mash が見つけた履歴を確認できます。Mash は元の保存場所からチャットを読み、既定では、このコンピューターに検索用のコピーを保存します。エージェントを接続すると、そのエージェントの設定が更新され、このコピーから抜粋を取得できるようになります。[エージェントを接続せずに自分で検索](#skip-agent-connections)することもできます。[Mash のデータ保存場所](privacy.md#where-mash-stores-data)

クイックスタートのコマンドは、ダッシュボードを開く前にプログラムをインストールして Qdrant を起動します。
セットアップを途中でやめた場合も、`mash uninstall` でインストール済みのものと削除方法を確認できます。
[削除対象を確認する（英語）](../uninstall.md)

<span id="install-the-cli"></span>
<span id="get-mash-for-your-system"></span>

## お使いの環境に Mash をインストールする

リリースには、ブラウザーで使うダッシュボードが含まれます。
検索サービスを起動すれば、[ブラウザーでセットアップ](#set-up-in-your-browser)できます。
[コーディングエージェントを接続](agents.md)すると、チャットの検索をエージェントにも頼めます。

<span id="recommended-release-installers"></span>

### リリースのインストーラーを使う

ビルド済みのリリースは Linux x86-64 と ARM64、Apple シリコン搭載 Mac、Windows x64 に対応しています。
お使いの環境に合うコマンドを 1 つ選んでください。[インストーラーの内容やダウンロードしたファイル](downloads.md)は事前に確認できます。

Linux 版には glibc 2.34 以降が必要です。Ubuntu 22.04、Debian 12、RHEL 9 や各ディストリビューションのそれ以降の版で利用できます。
Alpine など、musl を使う環境には対応していません。

<span id="macos-or-linux"></span>

#### macOS または Linux

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

WSL でもこのコマンドを使います。

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

どちらのスクリプトもリリースのチェックサムを検証し、ユーザーの `.local/bin` にインストールします。
インストール後に表示されるコマンドには実行ファイルのパスが含まれるので、フォルダーを `PATH` に追加する前でも実行できます。
特定のリリースを使うには `MASH_VERSION` に公開済みのタグを、別のフォルダーにインストールするには `MASH_INSTALL_DIR` にそのパスを指定します。
スクリプトはシェルの初期化ファイルを変更しません。
端末で `mash` が見つからなければ、[コマンドが見つからないときの対処法](troubleshooting.md#mash-command-not-found)をご覧ください。

<span id="package-manager-alternatives"></span>

### パッケージマネージャーでインストールする

[Homebrew、apt、Scoop、mise、cargo-binstall を使う手順](install-methods.md#install-with-a-package-manager)をご覧ください。
インストール後は[検索サービスを起動](#start-the-local-search-service)します。

<span id="build-from-source"></span>

### ソースからビルドする

ブラウザーで検索するには、[ダッシュボードを含めてビルド](install-methods.md#include-the-web-dashboard)します。
[端末での検索とエージェントのツール](install-methods.md#cli-and-agent-tools)だけを使う場合は、そちらの短い Cargo コマンドでビルドできます。

<span id="start-the-local-search-service"></span>

## ローカル検索サービスを起動する

Linux または WSL で以下のコマンドを実行するには、systemd のユーザーセッションが必要です。
[端末で使えるかの確認と代替手段](local-search.md#linux-without-a-systemd-user-session)をご覧ください。
[対応するローカル Qdrant](local-search.md#run-qdrant-yourself)がすでに動いていれば、[読み込む履歴の選択](#connect-your-histories)に進めます。

Windows では、Mash は Qdrant の検索データをシステムドライブ上の短いパスに保存します。通常は `C:\mash-qdrant\storage` です。
[Qdrant のファイルとログの場所](local-search.md#where-qdrant-stores-data)をご覧ください。

```sh
mash qdrant install
```

Mash は Qdrant をダウンロード（約 30 MB）して内容を検証し、自分のユーザー権限でバックグラウンドのサービスを起動します。
Qdrant はこのコンピューターからの接続だけを受け付け、次回ログイン時にも起動します。Windows ではデスクトップへのサインインが必要です。
サービスが起動しない場合は、[ログを確認してサービスを再起動する手順](local-search.md#check-or-restart-the-service)をご覧ください。

<span id="run-qdrant-yourself"></span>

### Qdrant を自分で起動する

[ローカルの Qdrant 1.20 以降](local-search.md#run-qdrant-yourself)をそのまま使うこともできます。
これから起動する場合は、お使いの環境に合う手順を選んでください。

- <span id="optional-qdrant-docker-image"></span>[Docker で Qdrant を起動する](local-search.md#optional-qdrant-docker-image)
- <span id="native-windows-without-docker"></span>[Windows で Docker を使わずに起動する](local-search.md#native-windows-without-docker)
- <span id="native-macos-or-linux-without-docker"></span>[macOS または Linux で Docker を使わずに起動する](local-search.md#native-macos-or-linux-without-docker)

<span id="connect-your-histories"></span>

## 読み込む履歴を選ぶ

ローカル検索サービスが起動したら、ブラウザーのダッシュボードか端末のウィザードで、Mash が読み込むチャットを選びます。

<span id="set-up-in-your-browser"></span>

### ブラウザーでセットアップする

リリースをインストールした場合、または[ダッシュボードを含めてビルド](install-methods.md#include-the-web-dashboard)した場合は、端末で次のコマンドを実行します。

```sh
mash dash
```

Mash が表示したリンクを開き、端末は開いたままにします。
初めてセットアップするときは、ブラウザーの案内に沿って進み、**Set up and index** を選びます。
エージェントの接続は省略できます。選択済みのエージェントがあれば、選択を解除できます。
検索の準備ができたら **chats** を開き、覚えている言葉で検索します。
[ダッシュボードでのセットアップ手順](dashboard.md#set-up-mash-from-the-dashboard)

<span id="set-up-in-a-terminal"></span>

### 端末でセットアップする

Mash のインストールと端末のウィザードの起動を 1 つのコマンドで済ませるには、[クイックスタートのスクリプト](quick-start.md#install-it-yourself)のオプションを置き換えます。
macOS または Linux では `--dashboard` の代わりに `--setup`、Windows では `-Dashboard` の代わりに `-Setup` を指定します。

ダッシュボードを含めずにソースからビルドした場合や、履歴フォルダーを指定したい場合、共有サーバーに接続したい場合は、端末のウィザードを使います。

```sh
mash init
```

初回は、見つかった履歴フォルダー、読み込む期間（過去 90 日以内に更新されたファイル）、接続できるエージェントが表示されます。
対象のチャットには、それより前のメッセージが含まれることもあります。
検索先を尋ねられたときに **Just this computer (recommended)** を選ぶと、インデックスはこのコンピューターに保存されます。
**Set up and index** を選ぶと、確認画面の設定が適用され、選んだエージェントも接続されます。
エージェントのコマンドが見つからなくても、Mash は保存済みのチャットを読み込めます。

期間や履歴フォルダーを変えたい、接続するエージェントを選びたい、既存の共有サーバーに接続したい、または設定だけ保存したい場合は **Customize setup** を選びます。
この手順ではインデックスの作成後に、任意で[過去のチャットから記憶を作成（英語）](../records.md#backfill-earlier-memory)できます。
この処理ではサインイン済みの Claude Code または Codex を使い、選ばれたチャットのテキストをそのモデルサービスに送ります。

Esc または Ctrl-C を押せば、Mash の設定やエージェントの設定を変更する前にキャンセルできます。
履歴の読み込みを選ぶと、ウィザードはローカル検索サービスの HTTP 接続先を確認します。共有を選んだ場合は、サーバーの認証も確認します。
接続できなければ、先に設定を保存し、あとからセットアップを完了できます。

エージェントを接続した場合は、Mash を使えるようにエージェントを再起動します。
履歴プラグインのフックには Python 3.9 以降が必要です。[エージェントと Mash を使う](agents.md)

<span id="skip-agent-connections"></span>

### エージェントを接続せずに使う

コーディングエージェントに Mash の検索を許可しなくても、ダッシュボードや端末で自分のチャットを検索できます。
ウィザードの表示に応じて、次のいずれかの方法で進めます。

- 最初の確認画面に **Set up and index without connecting agents** があれば、それを選んで表示された履歴を読み込みます。
- **Which agents should get access to Mash search?** と尋ねられたら、スペースキーですべての選択を解除し、Enter を押します。過去のチャットから記憶を作るか尋ねられた場合は、作成しない方を選びます。
- 確認画面に接続するエージェントが表示されていなければ、**Set up and index** を選びます。

検索の準備ができたら[ダッシュボードを開きます](#try-your-first-search)。
[あとからエージェントを接続](agents.md#connect-claude-code-codex-or-copilot)することもできます。

<span id="try-your-first-search"></span>

## 最初のチャットを検索する

ダッシュボードを含めてインストールした場合は、`mash dash` を実行して、表示されたアドレスを開きます。
**chats** で以前のチャットに出てきた特徴的な言葉を検索します。結果を開くと、元のメッセージを読めます。
[ダッシュボードでチャットを探して読む](dashboard.md#find-and-read-a-chat)

ダッシュボードを開かずにチャットを探すには、[端末で検索する](search.md#your-first-search)か、[接続済みのエージェントに頼みます](agents.md#try-it)。
探している古いチャットが見つからない場合は、[古い履歴を追加する方法（英語）](../indexing.md#add-older-history)をご覧ください。

言葉で検索するだけならモデルは不要です。意味で検索できるようにするには、[任意のモデル設定（英語）](../indexing.md#enable-semantic-search)に進みます。

<span id="setup-options"></span>

## セットアップのオプション

<span id="without-the-wizard"></span>

### ウィザードを使わずにセットアップする

`mash init --yes` は自動検出した既定の設定を使います。
過去 90 日以内に更新された履歴が対象で、見つかった対応エージェント（Copilot を含む）の進行中のセッションも含みます。
接続できるエージェントは接続し、過去のチャットからの記憶作成は行いません。
標準入力か標準エラー出力のどちらかが端末につながっていない場合も、対話なしで実行します。
JSON の結果に、選ばれたエージェント別の履歴フォルダーが表示されます。

ウィザードを使わない場合、`mash init` は何かを変更する前に Qdrant を確認します。
Qdrant が応答しなければ、起動するためのコマンドを表示して終了します。Mash とエージェントの設定はそのままです。

| オプション | 動作 |
| --- | --- |
| `--no-index` | 履歴を読み込まず、設定だけを保存します。Qdrant の確認を省略し、次の手順に Qdrant の起動を表示します。フォルダーを自分で選ぶには、[設定ファイル（英語）](../configuration.md)を編集してから `mash index` を実行します。 |
| `--no-agents` | エージェントを接続せず、ダッシュボードや端末から検索します。 |
| `--force` | 対話なしのセットアップでは、このオプションを付けた場合にだけ既存の設定を書き換えます。`--yes` と併用すると、検出した履歴から設定を作り直します。ウィザードでは設定の確認画面を直接開きます。 |

`--force` を使わない場合、ウィザードでは既存の設定を使うか、内容を確認するかを選べます。
内容を確認する場合は、ウィザードで扱わない設定が維持され、以前のファイルのバックアップも作られます。

Qdrant がまだ動いていなくても履歴を読み込めます。`mash init --no-index` を実行してから `mash index` を実行します。
インデックスの作成では、まずこのコンピューターにチャットを保存します。Qdrant が必要になった時点で処理を止め、起動に使うコマンドと保存したチャットの件数を表示します。
その間も `mash list-chats` と `mash view` で保存済みのチャットを読めます。
Qdrant を起動したら、もう一度 `mash index` を実行して検索できるようにします。

<span id="read-the-result"></span>

### セットアップ結果を確認する

端末のウィザードは、最後に短い結果と次の手順を表示します。
検索の準備ができると、ビルドの種類に応じて、ダッシュボードの起動か端末での検索に使うコマンドが表示されます。
結果を JSON で詳しく確認するには、`mash init --yes` を実行するか、`mash init > setup.json` でウィザードの標準出力をファイルに保存します。
質問と進捗は標準エラー出力に表示されます。

- `"agents": []` は、履歴の読み込みに成功していても、接続されたエージェントがないことを示します。理由は `next` を確認し、必要に応じて[エージェントのセットアップ](agents.md)に進みます。端末での検索はエージェントを接続せずに使えます。
- 接続したエージェントごとに、その CLI が変更した設定 `files` と、接続を解除する `undo` コマンドが表示されます。
- `index_error` は、設定を適用したあとに初回のインデックス作成が失敗したことを示します。たとえば、Qdrant が停止したり、gRPC ポートに接続できなかったりした場合です。コマンドはそのエラーで終了しますが、設定、エージェントの登録、保存済みの履歴は保持され、結果にも表示されます。原因を解消してから `mash index` を実行します。
- やり直したときに `written: false` と表示される場合は、既存の設定が再利用されています。
- 検索できるメッセージ数には、ユーザーとアシスタントの会話だけが含まれます。ツールの出力や推論の内容は元のチャットで読めますが、検索対象には含まれないため、表示される件数はチャット内の総数より大幅に少なくなることがあります。

<span id="check-the-release-download"></span>

## ダウンロードしたリリースを確認する

[リリースのアーカイブを選んでチェックサムを確認する](downloads.md)か、[インストーラーの内容を実行前に確認する](downloads.md#inspect-the-installer)ことができます。

<span id="uninstall"></span>

## アンインストールする

`mash uninstall` で Mash が削除する対象を確認できます。[アンインストール手順と削除後に残るデータ（英語）](../uninstall.md)をご覧ください。

<!-- mash-docs-source: ee4a2a27f23c403ecd60868d3802d5825da71d2845b9b858c965efd527ca32f3 -->
