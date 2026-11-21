# クイックスタート

<span id="quick-start"></span>

[日本語のサンプルチャットをデモで検索し](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F)、結果から元の会話を開いてみてください。
Mash をセットアップすると、自分のチャットをブラウザーや端末で検索できます。
接続したエージェントに検索してもらうこともできます。

コマンドを実行する前に、[セットアップで変わること（英語）](../installation.md#what-setup-changes)と
[ダウンロードの検証方法](downloads.md#verify-the-checksum)を確認できます。
まずは少量のデータで使いたい場合は、[1 つの履歴で試す](privacy.md#try-a-small-local-setup)方法をご覧ください。
ダッシュボードのボタンやメニューは英語で表示されます。

<span id="ask-your-agent"></span>

## エージェントに頼む

Claude Code、Codex、またはほかのコーディングエージェントに、次の文を貼り付けてください。

> https://getmash.io/setup.md を読み、このコンピューターに Mash をセットアップしてください。
> 実行する前に、設定の選択肢を日本語で一緒に確認してください。

まずダッシュボードだけを使い、エージェントの接続はあとから選ぶこともできます。
[Mash プラグイン（英語）](../plugin.md#start-with-the-plugin)をすでに入れた場合は、
`setup` スキルを使うように頼んでください。

<span id="install-it-yourself"></span>

## 自分でインストールする

[対応するエージェント](supported-agents.md)で保存したチャットが少なくとも 1 件必要です。
以下のコマンドは Mash をインストールし、ローカル検索サービスとダッシュボードを起動します。
そのあと、ブラウザーでセットアップを続けます。Homebrew、apt、Scoop を使う場合は
[パッケージマネージャーでのインストール（英語）](../install-methods.md#install-with-a-package-manager)、
端末で設定する場合は [端末のセットアップウィザード（英語）](../installation.md#set-up-in-a-terminal)をご覧ください。

macOS（Apple シリコン）または Linux（x86-64、ARM64）の端末で実行します。
Linux または WSL で Qdrant を起動するには systemd のユーザーセッションが必要です。
[セッションの確認と、別の起動方法（英語）](../local-search.md#linux-without-a-systemd-user-session)

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --dashboard
```

Windows x64 では、検索データがシステムドライブ上の短いパスに保存されます。
通常は `C:\mash-qdrant\storage` です。[Mash のデータ保存場所](privacy.md#where-mash-stores-data)
PowerShell で次のコマンドを実行してください。

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Dashboard
```

スクリプトはダウンロードを検証し、Mash を `.local/bin` にインストールします。
Qdrant（約 30 MB）は自分のユーザー権限で動き、ログイン時に起動します。
単語検索はモデルをダウンロードしなくても使えます。

インストーラーが表示したアドレス（通常は [http://127.0.0.1:8790](http://127.0.0.1:8790)）を開き、
端末は開いたままにします。ブラウザーで、見つかった履歴と読み込む期間を確認してください。
既定は過去 90 日間です。履歴の選択画面が最初に表示された場合は、
**Next** で確認画面まで進みます。

- **自分で検索する。** エージェントが選択されていないことを確認します。最初から選択されている場合は、**Skip agent connections** を選ぶか、エージェントの画面で選択を解除します。
- **エージェントに検索してもらう。** **Choose agents to connect** またはエージェントの画面で、接続したいものを選びます。確認画面に表示された名前を確認します。

**Set up and index** を選びます。[ダッシュボードでのセットアップ手順（英語）](../dashboard.md#set-up-mash-from-the-dashboard)

すでにある検索サービスを使う場合や、インストールとセットアップを分ける場合は、
[インストールガイド（英語）](../installation.md)をご覧ください。

<span id="try-it"></span>
<span id="find-your-first-chat"></span>

## 最初のチャットを見つける

検索の準備ができたら、**chats** を開きます。エラーや関数名など、
チャットで使った言葉を入力して Enter を押します。結果を開くと、
該当するメッセージを会話の前後とともに読めます。

あとからダッシュボードを開き直すには、`mash dash` を実行し、表示されたアドレスを開きます。
端末で `mash` が見つからない場合は、セットアップ時に表示された実行ファイルのフルパスを使います。
[端末から `mash` を実行できるようにする（英語）](../troubleshooting.md#mash-command-not-found)

- [検索結果が見つからないときの確認（英語）](../troubleshooting.md#search-returns-no-results)
- [日本語やほかの言語のチャットを検索する](search-languages.md)
- [ダッシュボードで検索する（英語）](../dashboard.md#find-and-read-a-chat) · [端末で検索する（英語）](../search.md#your-first-search)

**エージェントから検索する。** セットアップで接続したエージェントを再起動します。
Codex なら「Mash を使って、Claude Code で認証について決めたことを探してください。
根拠になった元のメッセージも見せてください」と頼めます。
Mash が見つからない場合は、[接続を確認](agents.md#confirm-the-connection)してください。
セットアップ時に接続しなかった場合も、[あとから接続](agents.md#connect-claude-code-codex-or-copilot)できます。

新しいチャットを検索対象にするには、[`mash index` またはバックグラウンド監視で履歴を更新（英語）](../synchronization.md)します。
[履歴のバックアップ（英語）](../backup.md)も作成できます。

<!-- mash-docs-source: 62b75bc52e78a840f0a366d876bd1ca8c22b81cbfcbb81453c7673ba7f07a347 -->
