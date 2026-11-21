# 下载并校验 Mash

下载适合你电脑的预编译文件，核对发布时提供的校验值，然后从解压目录启动网页面板。这种方式可以跳过 Mash 安装脚本。你也可以[查看脚本](#inspect-the-installer)，或[使用软件包管理器安装](install-methods.md)。

<span id="choose-your-download"></span>

## 选择下载文件

以下链接指向最新发布的安装包。每个安装包都包含终端程序和网页面板。

| 电脑 | 下载文件 |
| --- | --- |
| 搭载 Apple 芯片的 Mac | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

可以在 GitHub 上查看[版本说明和校验文件](https://github.com/mashium/mash/releases/latest)。校验下载文件时，请使用同一版本发布页的安装包和 `SHA256SUMS`。使用 Linux 时，请先查看[系统要求](installation.md#recommended-release-installers)。Intel Mac 等平台可以[从源码构建](install-methods.md#build-from-source)。

<span id="verify-the-checksum"></span>

## 核对校验值

在下载目录中运行适合你系统的命令。如果使用 Linux ARM64，请把示例中的文件名换成你下载的文件名。

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

在 `SHA256SUMS` 中找到与下载文件名完全一致的一行，把该行的校验值与命令输出比较。校验值的字母不区分大小写。如果两者一致，就说明下载的文件与发布的文件相同。

<span id="use-the-archive"></span>

## 使用下载的安装包

将 `mash` 和 `mash-server` 解压到你自己的目录，Windows 上则解压两个 `.exe` 文件。在该目录打开终端。在 macOS 或 Linux 上运行 `./mash --version`，或在 PowerShell 中运行 `.\mash.exe --version`。命令会显示你下载的版本。

在同一个目录中启动本地搜索和网页面板。第一条命令会下载 Qdrant，并以你的用户身份启动服务。
如果你已经运行[兼容的本地 Qdrant](local-search.md#run-qdrant-yourself)，可以跳过第一条命令。
在 Linux 或 WSL 上，安装服务需要 [systemd 用户会话](local-search.md#linux-without-a-systemd-user-session)。

**macOS 或 Linux**

```sh
./mash qdrant install
./mash dash
```

**Windows PowerShell**

```powershell
.\mash.exe qdrant install
.\mash.exe dash
```

保持终端运行，在浏览器中打开终端显示的地址，通常是 [http://127.0.0.1:8790](http://127.0.0.1:8790)。
确认 Mash 可以读取哪些对话以及读取的时间范围。如果想自己搜索，可以不连接代理，然后选择 **Set up and index**。
[浏览器设置步骤](dashboard.md#set-up-mash-from-the-dashboard)

<span id="inspect-the-installer"></span>

## 查看安装脚本

运行前可以先阅读 [macOS 和 Linux 安装脚本](https://github.com/mashium/mash/blob/main/install.sh)或 [Windows 安装脚本](https://github.com/mashium/mash/blob/main/install.ps1)。脚本会校验安装包并把 Mash 放到你的用户目录，无需管理员权限。[设置会更改什么](installation.md#what-setup-changes)

<span id="check-how-it-was-built"></span>

## 核对构建来源

[公开的发布流程](https://github.com/mashium/mash/actions/workflows/release.yml)先确认该版本源码的 Linux 和 Windows 测试通过，再构建各平台的安装包。[公开的安装检查](https://github.com/mashium/mash/actions/workflows/release-check.yml)会显示运行了哪些安装程序，以及它们是否安装了预期版本。

如果版本提供了构建证明，GitHub CLI 可以验证安装包来自哪个工作流和源码标签。登录 GitHub CLI 后，把 `ARCHIVE` 换成下载文件的路径，把 `RELEASE_TAG` 换成包括前导 `v` 的版本标签。

```sh
gh attestation verify "ARCHIVE" --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

发布流程会在发布这些安装包之前核验构建证明。`v0.1.1-alpha` 及更早的版本使用上面的校验值核对步骤。

<!-- mash-docs-source: d089547ce72f1abd31d192459b684cc330176a9a9a11aeeadbb18065c6b8f2c4 -->
