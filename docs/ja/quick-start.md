# クイックスタート

<span id="quick-start"></span>

<span id="install-it-yourself"></span>

## 自分でインストールする

お使いの OS に合ったコマンドを実行すると、Mash がダウンロードされ、ブラウザーでセットアップを始められます。
[対応するエージェント](supported-agents.md)で保存したチャットが少なくとも 1 件必要です。
[セットアップで変わること](installation.md#what-setup-changes)と
[ダウンロードの検証方法](downloads.md#verify-the-checksum)も確認できます。

**macOS（Apple シリコン）または Linux（x86-64、ARM64）**

Linux または WSL では [systemd のユーザーセッション](local-search.md#linux-without-a-systemd-user-session)が必要です。

```bash
curl -fsSL https://raw.githubusercontent.com/mashhq/mash/main/install.sh | sh -s -- --dashboard
```

**Windows x64（PowerShell）**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashhq/mash/main/install.ps1))) -Dashboard
```

インストーラーに表示されたリンクを開き、端末は開いたままにしてください。
ブラウザーの案内に沿って進み、**Set up and index** を選びます。
エージェントの接続は省略できます。選択済みのエージェントがあれば、選択を解除できます。
[ダッシュボードでのセットアップ手順](dashboard.md#set-up-mash-from-the-dashboard)

<span id="try-it"></span>
<span id="find-your-first-chat"></span>

## 最初のチャットを見つける

検索の準備ができたら、**chats** を開きます。エラーや関数名など、
チャットで使った言葉を入力して Enter を押します。結果を開くと、
該当するメッセージを会話の前後とともに読めます。

あとからダッシュボードを開き直すには、`mash dash` を実行し、表示されたアドレスを開きます。
端末で `mash` が見つからない場合は、セットアップ時に表示されたフルパスを使うか、
[端末から `mash` を実行できるようにします](troubleshooting.md#mash-command-not-found)。
[ダッシュボードで検索する](dashboard.md#find-and-read-a-chat) ·
[端末で検索する](search.md#your-first-search) ·
[検索結果が見つからないときの確認](troubleshooting.md#search-returns-no-results)

<span id="ask-your-agent"></span>

## エージェントに頼む

Claude Code、Codex、またはほかのコーディングエージェントに、次の文を貼り付けてください。

> https://getmash.io/setup.md を読み、このコンピューターに Mash をセットアップしてください。
> 実行する前に、設定の選択肢を日本語で一緒に確認してください。

[Mash プラグイン（英語）](../plugin.md#start-with-the-plugin)をすでに入れた場合は、
`setup` スキルを使うように頼んでください。自分で Mash をインストールした場合は、[あとからエージェントを接続](agents.md#connect-claude-code-codex-or-copilot)できます。

パッケージマネージャーの使用、ソースからのビルド、端末でのセットアップについては
[インストールガイド](installation.md)をご覧ください。少量のデータで試す場合は、[1 つの履歴で試す](privacy.md#try-a-small-local-setup)方法をご覧ください。

<!-- mash-docs-source: d57d2515612c6e32065f147541df1dc3c18acd5e701eed2c2fd739454faeaf4a -->
