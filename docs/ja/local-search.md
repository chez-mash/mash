# ローカル検索サービス（Qdrant）

Mash は、このコンピューターで履歴のインデックスを作成し、検索するために Qdrant を使います。
ログイン時に起動するサービスを使うなら、Mash のインストーラーで設定できます。
すでに Qdrant を使っている場合や、別の方法で起動したい場合は、自分で用意した Qdrant も使えます。
Qdrant が起動したら、[読み込む履歴を選びます](installation.md#connect-your-histories)。

<span id="let-mash-manage-qdrant"></span>

## Mash で Qdrant をインストールする

```sh
mash qdrant install
```

ローカルのインデックス作成と検索には [Qdrant](https://qdrant.tech) を使います。
このコマンドは、[Qdrant の GitHub リリース](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)からお使いの環境向けの Qdrant 1.20.0（約 30 MB）をダウンロードします。
Mash に組み込まれた SHA-256 チェックサムと照合してから、自分のユーザー権限でバックグラウンドに起動します。
管理者権限や Docker は不要です。Qdrant はこのコンピューターからの接続だけを受け付け（`127.0.0.1` のポート 6333 と 6334）、次回ログイン時にも起動します。
成功すると JSON の結果に `"outcome": "installed"` と表示されます。

<span id="where-qdrant-stores-data"></span>

### Qdrant のデータとログの保存場所

以下は、Mash がインストールしたサービスの保存場所です。
Qdrant には、検索対象のチャットのテキストと、意味で検索するために作成したベクトルが保存されます。

| プラットフォーム | 実行方法 | ログの確認先 | データの保存先 |
| --- | --- | --- | --- |
| Linux | systemd のユーザーサービス `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant/storage` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant/storage` |
| Windows | ログオンタスク `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `%SystemDrive%\mash-qdrant\storage` |

Qdrant は `storage` と同じ階層にある `snapshots` フォルダーも使います。
Windows では、パスが長いと Qdrant が動作しないことがあるため、これらのデータフォルダーは通常 `C:\mash-qdrant` 以下に置かれます。
実行ファイル、設定、ログファイルは `%USERPROFILE%\.mash\qdrant` に残ります。

`mash status` の `local_qdrant.installed` で、実際のパス、バージョン、サービスの状態を確認できます。
`mash qdrant install` の結果では、パスは `service.storage` に表示されます。
[Mash のほかのファイルの保存場所](privacy.md#where-mash-stores-data)

<span id="check-or-restart-the-service"></span>

### サービスの状態を確認する・再起動する

起動時にエラーが出たら、[お使いの環境に対応するログのコマンドやファイル](#where-qdrant-stores-data)を確認します。

Windows のログオンタスクは、デスクトップにサインインしている間だけ動きます。
SSH でのみ接続するコンピューターや、誰もサインインしないサーバーでは、コマンドを実行しても Qdrant の起動を確認できなかったと表示されます。
その場合は、[Windows で Docker を使わずに起動する手順](#native-windows-without-docker)に従い、自分で Qdrant を起動してください。

インストールのコマンドはいつでも実行し直せます。Qdrant のデータを維持したまま、停止中のサービスを起動し、不足しているファイルを復元します。
新しいバージョンの Mash がより新しい Qdrant を指定していれば、Qdrant も更新します。
変更が不要なときの結果は `current` です。以下の Docker コンテナーなど、対応する Qdrant がすでに指定のポートで応答している場合は、何もインストールせず `already_running` と表示します。
別のプログラムがポートを使用中なら、コマンドは使用中のポートを表示して終了します。

- **Linux:** コマンドの実行には [systemd のユーザーセッション](#linux-without-a-systemd-user-session)が必要です。サービスはログイン中に動作します。誰もログインしないコンピューターでも動かし続けるには、`loginctl enable-linger` でログアウト後もユーザーサービスが動くようにします。システムによっては管理者権限が必要です。
- **別のポートを使う場合:** インストール前に、[Qdrant の設定（英語）](../configuration.md#local-qdrant)の `url` と `grpc_url` に `127.0.0.1` のアドレスを指定してください。

`mash init` は Qdrant が応答することを確認します。Qdrant のダウンロードや起動はこのコマンドでは行いません。

<span id="linux-without-a-systemd-user-session"></span>

### Linux で systemd のユーザーセッションがない場合

Mash のインストーラーは Linux のユーザーセッションを使って Qdrant をバックグラウンドで動かします。
セットアップに使う Linux または WSL のターミナルで、設定を変更せずに確認できる次のコマンドを実行します。

```sh
systemctl --user show-environment >/dev/null
```

エラーがなければ `mash qdrant install` を実行できます。
ユーザーバスに接続できない、あるいは `systemctl` が見つからないというエラーが出た場合は、[Docker で Qdrant を起動](#optional-qdrant-docker-image)するか、[Linux 用の実行ファイルを端末で起動](#native-macos-or-linux-without-docker)してください。
WSL 2 で systemd を有効にしたい場合は、[Microsoft の WSL ガイド](https://learn.microsoft.com/ja-jp/windows/wsl/systemd)に従い、WSL のターミナルでもう一度確認します。

クイックスタートのインストーラーが「Installed」と表示したあとに停止した場合も、Mash の実行ファイルはインストール済みです。
Qdrant を起動し、ブラウザーでセットアップするなら `mash dash`、端末なら `mash init` を実行します。
必要に応じて、インストール先の実行ファイルをパスで指定します。
これからインストールする場合は、[Mash の実行ファイルだけをインストールする手順](installation.md#recommended-release-installers)を済ませてから、自分で Qdrant を起動してください。

<span id="run-qdrant-yourself"></span>

## Qdrant を自分で起動する

[公式の Docker イメージ](#optional-qdrant-docker-image)を使うか、以下の手順で Qdrant を手動で起動します。
すでに使っている Qdrant 1.20 以降のサービスも利用できます。
HTTP と gRPC の両方のポートが、同じ Qdrant に接続する必要があります。
既定以外の接続先を使う場合は、[Qdrant の設定（英語）](../configuration.md#local-qdrant)をご覧ください。
この構成でも、Mash とダッシュボードはお使いのコンピューターで直接動作します。

Qdrant のデータは、起動方法ごとに別の場所に保存されます。
手動で起動した Qdrant、Docker コンテナー、`mash qdrant install` で入れたサービスを切り替える際は、先に元の Qdrant を停止してポート 6333 と 6334 を空けます。
切り替え後は `mash index --rebuild-local-views --full` を実行して、新しい Qdrant に検索用のデータを作ります。

<span id="native-windows-without-docker"></span>

### Windows で Docker を使わずに起動する

Windows x64 では [Windows 版の Qdrant 1.20.0](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip)をダウンロードし、`C:\mash-qdrant` か、書き込み可能でパスの短い別のフォルダーに展開します。
ユーザープロファイル内の長いパスを使うと、起動時の確認に通っても、Qdrant がペイロードのインデックスを作成できないことがあります。
PowerShell で次を実行します。

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

このウィンドウは開いたままにします。次回も同じ設定で起動すると、保存済みのデータを再利用できます。
別のターミナルウィンドウで `curl.exe --fail http://127.0.0.1:6333/readyz` を実行し、Qdrant の準備ができたことを確認してから `mash init` を実行します。

<span id="native-macos-or-linux-without-docker"></span>

### macOS または Linux で Docker を使わずに起動する

[Qdrant 1.20.0 のリリース](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)から、お使いのコンピューター向けのアーカイブをダウンロードします。

| コンピューター | アーカイブ |
| --- | --- |
| Apple シリコン搭載 Mac | [ダウンロード](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| Intel 搭載 Mac | [ダウンロード](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux x86_64 | [ダウンロード](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux ARM64 | [ダウンロード](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

`~/qdrant` などの専用フォルダーに展開し、そのフォルダーで起動します。

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

このターミナルは開いたままにします。次回も同じフォルダーから起動すると、保存済みのデータを再利用できます。
別のターミナルウィンドウで `curl --fail http://127.0.0.1:6333/readyz` を実行し、Qdrant の準備ができたことを確認してから `mash init` を実行します。

<span id="optional-qdrant-docker-image"></span>

### Docker で Qdrant を起動する

macOS や Windows では [Docker Desktop](https://www.docker.com/products/docker-desktop/)、Linux では [Docker Engine（英語）](https://docs.docker.com/engine/install/)をインストールして起動します。
Windows では Linux コンテナーを実行できる環境が必要です。
Windows Server や Docker Engine だけを使う環境では、Linux コンテナーに対応していないことがあります。
その場合は、[Windows で Docker を使わずに起動する手順](#native-windows-without-docker)をご覧ください。
Docker が起動したら、Bash または PowerShell で次を実行します。

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

この Qdrant に接続できるのは、このコンピューターだけです。
[起動状況を確認するページ](http://127.0.0.1:6333/readyz)を開き、「all shards are ready」と表示されてから `mash init` を実行します。

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

手動で停止していなければ、Docker の再起動時に Qdrant も再起動します。
コンテナーがすでにある場合は、`docker run` ではなく `docker start` を使ってください。
検索データは `mash-qdrant-data` ボリュームにあり、コンテナーを停止しても残ります。ボリュームを削除するのは、保存されているデータを破棄するときだけにしてください。

<!-- mash-docs-source: 5506852e48e6f2fe5fb713326fe5b9ea939c430cd352dd42c470fb85bf1aab9f -->
