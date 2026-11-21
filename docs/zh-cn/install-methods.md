# Mash 的其他安装方式

本指南介绍如何用软件包管理器安装 Mash，或从源码构建。安装后，请[启动本地搜索服务](installation.md#start-the-local-search-service)，
并[选择要读取的历史记录](installation.md#connect-your-histories)。如果已经安装了 Mash，请按[升级步骤（英文）](../upgrades.md#upgrade-an-existing-installation)操作。

<span id="install-with-a-package-manager"></span>

## 用软件包管理器安装

以下软件包使用 Mash 的预编译版本，包含网页面板、终端搜索，以及已连接的代理可使用的工具。
设置完成后，运行 `mash dash`，然后打开命令显示的地址。安装前，你可以[校验已发布的下载文件](installation.md#check-the-release-download)，
也可以[查看每个版本有哪些安装方式通过了检查](https://github.com/mashium/mash/actions/workflows/release-check.yml)。

<span id="homebrew-macos-or-linux"></span>

### Homebrew，适用于 macOS 或 Linux

在搭载 Apple 芯片的 macOS 或 x86-64、ARM64 架构的 Linux 上，请使用
[Mashium 的 Homebrew 配方](https://github.com/mashium/homebrew-tap/blob/main/Formula/mash.rb)安装。完整名称告诉 Homebrew 使用哪个仓库，也让 [Homebrew 的 tap 信任机制](https://docs.brew.sh/Tap-Trust)只针对这个配方。

```sh
brew install mashium/tap/mash
```

<span id="apt-debian-or-ubuntu"></span>

### apt，适用于 Debian 或 Ubuntu

Mashium 的 apt 仓库发布带有签名的软件包元数据。选项 `signed-by` 为此仓库指定
[Mashium 的签名密钥](https://getmash.io/apt/mash-archive-keyring.gpg)。
添加仓库前，可以[查看签名后的元数据](https://getmash.io/apt/dists/stable/InRelease)。

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

软件包名为 `mash-cli`，因为 Debian 和 Ubuntu 已经有另一个无关的 `mash` 软件包。
安装后的命令仍然是 `mash`。两个软件包无法同时安装，apt 会提示移除另一个。

<span id="scoop-windows"></span>

### Scoop，适用于 Windows

安装 Scoop 后，在 PowerShell 中添加 Mashium 的软件包仓库，再安装
[Mash 软件包](https://github.com/mashium/mash/blob/main/bucket/mash.json)。清单列出了 Windows 压缩包及其 SHA-256 哈希值。

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

### mise

安装 mise 后，可以通过它的 GitHub 后端从
[Mashium 的发布压缩包](https://github.com/mashium/mash/releases)安装 Mash。

```sh
mise use -g github:mashium/mash
```

### cargo-binstall

安装 cargo-binstall 后，指定 GitHub 项目即可使用 Mashium 的发布压缩包。

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/mashium/mash mash
```

如果你的平台没有发布压缩包，请参阅
[从源码构建](#build-from-source)。

<span id="build-from-source"></span>

## 从源码构建

如果你的电脑没有适用的预编译版本，或者想尝试尚未发布的修订，可以按以下步骤构建。
[构建包含网页面板的版本](#include-the-web-dashboard)，就可以在浏览器中搜索。
如果只需要[终端搜索和代理工具](#cli-and-agent-tools)，请使用下面较短的 Cargo 命令。

<span id="include-the-web-dashboard"></span>

### 构建包含网页面板的版本

若要包含网页面板，请先构建网页文件，再编译 Mash。准备好 Git、[Node.js 24 或更高版本](https://nodejs.org/en/download)
和稳定版 Rust 工具链后，在终端中运行以下命令。

```sh
git clone https://github.com/mashium/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo +stable install --locked --path .
```

如果想给现有安装加上网页面板，请先切换到已安装版本对应的修订再构建。完成后，
[启动本地搜索并为对话建立索引](installation.md#start-the-local-search-service)，然后运行 `mash dash` 并打开命令显示的地址。

<span id="cli-and-agent-tools"></span>

### 终端搜索和代理工具

以下命令会构建终端搜索，以及已连接的代理可使用的工具。要在浏览器中搜索，请按上面的步骤构建。

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

构建完成后，[启动本地搜索并为对话建立索引](installation.md#start-the-local-search-service)。
然后[在终端中进行第一次搜索](search.md#your-first-search)。

如果构建后终端找不到 Mash，请参阅
[如何将 Cargo 的可执行文件目录加入命令搜索路径](troubleshooting.md#mash-command-not-found)。

<!-- mash-docs-source: d3613d85a3631aa76607f2cf3c51639286c04dc9a1a3a07c2ddb06b5bcadb149 -->
