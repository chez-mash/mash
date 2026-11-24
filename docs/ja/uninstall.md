# Mash をアンインストールする

エージェントが保存した元のチャットは、Mash をアンインストールしても残ります。
Mash が検索用に保存したコピーと、保存済みの記憶は、アンインストール時に削除できます。

| やりたいこと | 最初にすること |
| --- | --- |
| Mash と、ローカルに保存したチャットのコピーを削除する | [削除内容を確認する](#remove-mash-and-its-local-copy) |
| あとで同じコンピューターに再インストールし、検索用のチャットと記憶を使い続ける | [Mash のコピーをそのまま残す](#keep-history-for-a-reinstall) |
| 同じコンピューターや別のコンピューターで復元できるように、別のコピーを保存する | [先に持ち運べるバックアップを作る（英語）](../backup.md#make-a-portable-backup) |

<span id="remove-mash-and-its-local-copy"></span>

## Mash とローカルのコピーを削除する

まず、削除の内容を確認します。次のコマンドは、削除予定の項目を表示するだけです。

```sh
mash uninstall
```

削除されるものと残るものを確認してください。Mash が保存したチャットのコピーの容量も表示されます。
結果が JSON で表示された場合は、`paths`、`kept`、`next` を確認します。

削除する準備ができたら、端末から起動した Mash のダッシュボードや履歴の監視プロセスを終了し、次のコマンドで削除を確定します。

```sh
mash uninstall --yes
```

Mash のインデックス、設定、保存済みのトークン、ダウンロードしたモデルが削除されます。
Mash が設定したサービスは停止して削除されます。Mash がインストールした検索サービスと、そのデータも対象です。
[接続したエージェント](agents.md#disconnect-an-agent)からは、それぞれのエージェントのコマンドを使って、Mash の検索ツールとプラグインの両方を削除します。

接続していたエージェントがまだ起動中なら、再起動してください。
結果に失敗した手順が表示された場合は、その原因を解消し、同じオプションでコマンドを再実行します。
削除が完了したら、最後に表示された手順に従って Mash のプログラム本体を削除します。
JSON では `binary.remove` を確認してください。インストール方法に応じて、`brew uninstall mash`、`scoop uninstall mash`、`cargo uninstall mash` などのコマンドか、削除するファイルが表示されます。

apt で `mash-cli` パッケージをインストールした場合は、`sudo apt remove mash-cli` を実行します。
Mash の `v0.1.1-alpha` では、削除結果に誤ったパッケージ名が表示されることがあります。

<span id="copies-that-stay-in-place"></span>

## アンインストールしても残るデータ

- エージェントが保存した元のチャットと、`~/.mash` 内にある Mash の管理外のファイルは、コンピューターに残ります。
- 共有サーバーにチャットを送信済みの場合、サーバー上のコピーは残ります。共有先のコピーを削除し、このコンピューターからのアクセスを取り消すには、[共有インデックスからコンピューターを削除する手順（英語）](../shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again)に従ってください。
- Qdrant を自分で運用している場合は、そのサービスと Mash の検索データを別途管理してください。

Mash がインストールした検索サービスだけを削除するには、`mash qdrant uninstall` を実行します。
`--delete-storage` を指定しなければ、検索データは残ります。
検索データは、ローカルにある Mash のインデックスから `mash index --rebuild-local-views --full` で作り直せます。

<span id="keep-history-for-a-reinstall"></span>

## 再インストールに備えて履歴を残す

あとで同じコンピューターに再インストールして、検索用に保存したチャットや記憶を使い続ける場合は、`--keep-history` を指定します。
Mash のインデックス、設定、保存済みのトークンと、Mash が管理する検索サービスのデータが残ります。
サービス、エージェントの接続、ダウンロードしたモデルは削除されます。

まず、このオプションを付けて削除内容を確認します。削除を確定するときも、同じオプションを付けてください。

```sh
mash uninstall --keep-history
```

`v0.1.1-alpha` では、削除内容の確認後に表示される確定用のコマンドに `--keep-history` が含まれていないことがあります。
次のコマンドを省略せずに使ってください。`--yes` だけを指定して確定すると、Mash が検索用に保存したチャットと記憶が削除されます。

```sh
mash uninstall --yes --keep-history
```

Mash のプログラム本体を削除する前に `mash --version` を実行し、再インストールに備えてバージョンを控えます。
失敗した手順の再実行、接続していたエージェントの再起動、プログラム本体の削除は、[Mash とローカルのコピーを削除する手順](#remove-mash-and-its-local-copy)と同様に行います。

<span id="reopen-the-retained-history"></span>

### 残した履歴を開き直す

1. 同じコンピューターで同じユーザーアカウントを使い、控えておいたバージョンの Mash をインストールします。
   [リリースのインストーラー](installation.md#recommended-release-installers)を実行する前に、`MASH_VERSION` に `v0.1.1-alpha` などのリリースタグを指定します。
   アーカイブからインストールする場合は、[GitHub リリース](https://github.com/grep-stack/mash/releases)で控えておいたタグを開き、同じリリースからお使いの環境向けのアーカイブと `SHA256SUMS` をダウンロードしてください。
   展開する前に、[アーカイブのチェックサムを確認](downloads.md#verify-the-checksum)します。
   より新しいバージョンを入れる場合は、[アップグレードガイド（英語）](../upgrades.md#upgrade-an-existing-installation)に従ってください。
2. Mash が検索サービスをインストールしていた場合は、`mash qdrant install` を実行してサービスを入れ直します。以前の検索データが再利用されます。
   [Qdrant を自分で起動していた場合](local-search.md#run-qdrant-yourself)は、以前と同じ方法で起動してください。
3. `mash dash` を実行して表示されたアドレスを開くか、端末で `mash list-chats --local --limit 5` を実行して、保存済みのチャットを確認します。
   Mash は保存済みの設定とインデックスを使います。以前に別のパスを指定していた場合は、同じパスを再度指定してください。

検索を更新する必要がある場合は、[チャットの履歴を最新に保つ](synchronization.md)をご覧ください。
意味で検索していた場合は、[モデルを入れ直す手順（英語）](../indexing.md#install-the-models)をご覧ください。
[エージェントを接続し直す](agents.md#connect-claude-code-codex-or-copilot)こともできます。

<!-- mash-docs-source: 78c676e9d7c8bc968b81510a879cb7d593ba3e7d150d2620a927882b98f7d57e -->
