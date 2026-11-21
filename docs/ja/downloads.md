# Mash のダウンロードと検証

お使いのコンピューター向けのアーカイブをダウンロードし、公開されているチェックサムと照合します。
展開先のフォルダーから Mash を起動できます。[パッケージマネージャーでインストールする（英語）](../install-methods.md)方法もあります。

<span id="choose-your-download"></span>

## ダウンロードを選ぶ

以下のリンクから最新の公開アーカイブを取得できます。各アーカイブに端末用のプログラムとブラウザー用のダッシュボードが含まれます。

| コンピューター | ダウンロード |
| --- | --- |
| Apple シリコン搭載 Mac | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

アーカイブと `SHA256SUMS` は同じ [GitHub リリースページ](https://github.com/mashium/mash/releases/latest)から取得してください。
Linux の動作要件は [対応環境（英語）](../installation.md#recommended-release-installers)をご覧ください。
Intel Mac などでは [ソースからビルド（英語）](../install-methods.md#build-from-source)できます。

<span id="verify-the-checksum"></span>

## チェックサムを確認する

ダウンロードしたファイルのあるフォルダーで、お使いの OS に合うコマンドを実行します。
Linux ARM64 では、コマンドのファイル名をダウンロードしたアーカイブの名前に置き換えてください。

**Linux**

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

**macOS**

```sh
shasum -a 256 mash-aarch64-apple-darwin.tar.gz
```

**Windows PowerShell**

```powershell
Get-FileHash -Algorithm SHA256 .\mash-x86_64-pc-windows-msvc.zip
```

表示されたハッシュ値を、`SHA256SUMS` の同じファイル名の行と照合します。
英字の大文字と小文字は区別しません。一致すれば、公開されたファイルと同じ内容です。

<span id="use-the-archive"></span>

## アーカイブから使う

`mash` と `mash-server`（Windows では 2 つの `.exe` ファイル）を自分のフォルダーに展開します。
そのフォルダーで端末を開き、macOS または Linux では `./mash --version`、
PowerShell では `.\mash.exe --version` を実行すると、ダウンロードしたバージョンが表示されます。

[ローカル検索サービスを起動し（英語）](../installation.md#start-the-local-search-service)、読み込むチャットを選びます。
コマンドを実行するときは Mash のフルパスを指定するか、
[展開先を PATH に追加（英語）](../troubleshooting.md#mash-command-not-found)してください。

<span id="inspect-the-installer"></span>

## インストーラーを確認する

実行前に [macOS と Linux 用のスクリプト](https://github.com/mashium/mash/blob/main/install.sh)や
[Windows 用のスクリプト](https://github.com/mashium/mash/blob/main/install.ps1)を確認できます。
スクリプトはチェックサムを検証し、管理者権限を使わずに Mash をユーザーフォルダーに配置します。
[セットアップで変わること（英語）](../installation.md#what-setup-changes)

<span id="check-how-it-was-built"></span>

## ビルドの出所を確認する

[公開リリースワークフロー](https://github.com/mashium/mash/actions/workflows/release.yml)は、
対象のコミットで Linux と Windows の CI が通ったことを確認してからアーカイブを作成します。
[インストール確認の実行結果](https://github.com/mashium/mash/actions/workflows/release-check.yml)では、
どのインストーラーを実行し、配布バージョンを確認したかを見ることができます。

ビルド証明書が付いたリリースは、GitHub CLI で生成元のワークフローとソースタグを検証できます。
GitHub CLI にログインし、`ARCHIVE` をダウンロードしたファイルのパスに、`RELEASE_TAG` を先頭の `v` を含むタグに置き換えます。

```sh
gh attestation verify "ARCHIVE" --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

リリースワークフローでもアーカイブの公開前に証明書を検証します。
`v0.1.1-alpha` までのリリースは、上記のチェックサムで確認してください。

<!-- mash-docs-source: 594b9d6aebb6145cc4183bc3c653c9262c2e692f6fbe4963c3e44038960682c0 -->
