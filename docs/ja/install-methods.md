# Mash のほかのインストール方法

パッケージマネージャーでインストールするか、ソースからビルドする場合の手順です。
インストールが終わったら、[ローカル検索サービスを起動し](installation.md#start-the-local-search-service)、
[読み込む履歴を選びます](installation.md#connect-your-histories)。
すでに Mash を使っている場合は、[既存のインストールをアップグレードする手順（英語）](../upgrades.md#upgrade-an-existing-installation)をご覧ください。

<span id="install-with-a-package-manager"></span>

## パッケージマネージャーでインストールする

以下のパッケージは、ビルド済みの Mash のリリースを使います。
ブラウザーで使うダッシュボード、端末での検索、接続したエージェントが使うツールが含まれます。
ローカル検索サービスを起動したら、[ブラウザーでセットアップを完了できます](installation.md#set-up-in-your-browser)。
インストールの前に、[公開済みのダウンロード](downloads.md)や、
[各リリースで検証に通ったインストーラー](https://github.com/mashium/mash/actions/workflows/release-check.yml)を確認できます。

<span id="homebrew-macos-or-linux"></span>

### Homebrew（macOS または Linux）

Apple シリコン搭載 Mac、または Linux の x86-64 と ARM64 では、[Mashium の Homebrew フォーミュラ](https://github.com/mashium/homebrew-tap/blob/main/Formula/mash.rb)からインストールできます。
下のようにリポジトリを含めた名前を指定すると、Homebrew が使用するリポジトリを特定でき、[tap の信頼設定（英語）](https://docs.brew.sh/Tap-Trust)の対象をこのフォーミュラに絞れます。

```sh
brew install mashium/tap/mash
```

<span id="apt-debian-or-ubuntu"></span>

### apt（Debian または Ubuntu）

x86-64 または ARM64 の Debian 12 以降、または Ubuntu 22.04 以降で使えます。
Mashium の apt リポジトリでは、署名済みのパッケージメタデータを公開しています。
`signed-by` で、このリポジトリの検証に使う [Mashium の署名鍵](https://getmash.io/apt/mash-archive-keyring.gpg)を指定します。
リポジトリを追加する前に、[署名済みのメタデータ](https://getmash.io/apt/dists/stable/InRelease)を確認できます。

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

Debian と Ubuntu には別の `mash` パッケージがあるため、Mash のパッケージ名は `mash-cli` です。
インストールされるコマンドは `mash` です。両方のパッケージを同時にはインストールできません。
もう一方がすでに入っている場合、apt はその削除を提案します。

<span id="scoop-windows"></span>

### Scoop（Windows）

Windows x64 で Scoop をインストールしたら、PowerShell を開き、Mashium の bucket を追加して、
[Mash パッケージ](https://github.com/mashium/mash/blob/main/bucket/mash.json)をインストールします。
マニフェストには、Windows 用アーカイブとその SHA-256 ハッシュが記載されています。

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

### mise

mise をインストール済みなら、GitHub バックエンドを使って
[Mashium のリリースアーカイブ](https://github.com/mashium/mash/releases)からインストールできます。

```sh
mise use -g github:mashium/mash
```

### cargo-binstall

cargo-binstall をインストール済みなら、GitHub のプロジェクトを指定して Mashium のリリースアーカイブを選びます。

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/mashium/mash mash
```

お使いの環境向けのリリースアーカイブがなければ、[ソースからビルドする](#build-from-source)手順をご覧ください。

<span id="build-from-source"></span>

## ソースからビルドする

お使いの環境向けのビルド済みリリースがない場合や、未公開のリビジョンを試す場合は、ソースからビルドします。
ブラウザーで検索するには、[Web ダッシュボードを含めてビルドします](#include-the-web-dashboard)。
[端末での検索とエージェント用ツール](#cli-and-agent-tools)だけを使う場合は、短い Cargo コマンドでインストールできます。

<span id="include-the-web-dashboard"></span>

### Web ダッシュボードを含めてビルドする

Mash をコンパイルする前に、ダッシュボードの Web ファイルをビルドします。
Git、[Node.js 24 以降（英語）](https://nodejs.org/en/download)、安定版の Rust ツールチェーンを用意し、端末で次のコマンドを実行します。

```sh
git clone https://github.com/mashium/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo +stable install --locked --path .
```

インストール済みの Mash にダッシュボードを追加する場合は、ビルドする前に、前回インストールしたリビジョンをチェックアウトしてください。
ビルドが終わったら、[ローカル検索サービスを起動してチャットをインデックスに登録します](installation.md#start-the-local-search-service)。
次に `mash dash` を実行し、表示されたアドレスをブラウザーで開きます。

<span id="cli-and-agent-tools"></span>

### 端末での検索とエージェント用ツール

次のコマンドは、端末での検索と、接続したエージェントが使うツールをビルドします。
ブラウザーで使うダッシュボードも必要なら、上のビルド手順を使ってください。

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

ビルドが終わったら、[ローカル検索サービスを起動してチャットをインデックスに登録し](installation.md#start-the-local-search-service)、
[端末で最初の検索を試します](search.md#your-first-search)。

どちらの方法でも、ビルド後に端末で Mash が見つからない場合は、
[Cargo の実行ファイルのフォルダーを `PATH` に追加する手順](troubleshooting.md#mash-command-not-found)をご覧ください。

<!-- mash-docs-source: 400da6d93d5805098a5628678b37ade9f61dc9fdc1eedfb23cf768d2c4e84eec -->
