# Download and verify Mash

Download the ready-to-run files for your computer, check them against the
published checksum, and open the dashboard from the folder where you extract
them. This path skips the Mash install script. You can also
[inspect the script](#inspect-the-installer) or [install with a package manager](install-methods.md).

## Desktop app

The Mash app opens the dashboard in its own window, puts search in the menu
bar, and includes everything the archives below do.

| Computer | Download |
| --- | --- |
| Mac with Apple silicon | [`mash-desktop-macos-arm64.dmg`](https://github.com/grep-stack/mash/releases/latest/download/mash-desktop-macos-arm64.dmg) |
| Windows x64 | [`mash-desktop-windows-x64-setup.exe`](https://github.com/grep-stack/mash/releases/latest/download/mash-desktop-windows-x64-setup.exe) |
| Windows ARM64 | [`mash-desktop-windows-arm64-setup.exe`](https://github.com/grep-stack/mash/releases/latest/download/mash-desktop-windows-arm64-setup.exe) |
| Linux x86-64 | [`mash-desktop-linux-x86_64.AppImage`](https://github.com/grep-stack/mash/releases/latest/download/mash-desktop-linux-x86_64.AppImage) |
| Linux ARM64 | [`mash-desktop-linux-arm64.AppImage`](https://github.com/grep-stack/mash/releases/latest/download/mash-desktop-linux-arm64.AppImage) |

The Linux app needs glibc 2.35 or newer (Ubuntu 22.04, Debian 12, Fedora 36 and later) and WebKitGTK 4.1.

The app is not signed yet, so your system asks before opening it the first
time. On a Mac, open the app, then choose **Open Anyway** in **System
Settings › Privacy & Security**. On Windows, choose **More info**, then
**Run anyway**. On Linux, make the AppImage executable
(`chmod +x mash-desktop-linux-x86_64.AppImage`) and run it.

## Choose your download

These links download the latest published archives, for using Mash without
the app. Each archive contains the terminal program and browser dashboard.

| Computer | Download |
| --- | --- |
| Mac with Apple silicon | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/grep-stack/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/grep-stack/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/grep-stack/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/grep-stack/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |
| Windows ARM64 | [`mash-aarch64-pc-windows-msvc.zip`](https://github.com/grep-stack/mash/releases/latest/download/mash-aarch64-pc-windows-msvc.zip) |

See the [release notes and checksum file on GitHub](https://github.com/grep-stack/mash/releases/latest).
Use the archive and `SHA256SUMS` from the same release page when checking a file.
For Linux requirements, see [supported systems](installation.md#recommended-release-installers).
Intel Macs aren't supported yet: Mash's search models run on ONNX Runtime,
which publishes no build for them, so Mash can't be built there from source
either.

## Verify the checksum

From the folder containing the download, run the command for your system.
On Linux ARM64 or Windows ARM64, substitute your archive's filename.

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

Compare the hash it prints with the line for the exact filename in
`SHA256SUMS`. Uppercase and lowercase letters in the hash are equivalent.
If the values match, you have the same file as the one published.

## Use the archive

Extract `mash` and `mash-server` (or both `.exe` files on Windows) to a
directory you own. In a terminal opened there, run `./mash --version` on
macOS or Linux, or `.\mash.exe --version` in PowerShell. It prints the
version you downloaded.

From the same folder, start local search and open the dashboard. The first
command downloads Qdrant and starts it as a service under your account. Skip
it if you already run a [compatible local Qdrant](local-search.md#run-qdrant-yourself).
On Linux or WSL, the install command needs a
[systemd user session](local-search.md#linux-without-a-systemd-user-session).

**macOS or Linux**

```sh
./mash qdrant install
./mash dash
```

**Windows PowerShell**

```powershell
.\mash.exe qdrant install
.\mash.exe dash
```

Leave the terminal open and visit the address it prints, usually
[http://127.0.0.1:8790](http://127.0.0.1:8790). Review which chats Mash can
read and how far back to look. Leave agent connections off to search on your
own, then choose **Set up and index**. [Follow the browser setup steps](dashboard.md#set-up-mash-from-the-dashboard)

## Inspect the installer

Read the [macOS and Linux installer](https://github.com/grep-stack/mash/blob/main/install.sh)
or the [Windows installer](https://github.com/grep-stack/mash/blob/main/install.ps1)
before you run it. The scripts verify the archive checksum and put Mash in
your user directory without administrator privileges.
[See what setup changes](installation.md#what-setup-changes)

## Check how it was built

The [public release workflow](https://github.com/grep-stack/mash/actions/workflows/release.yml)
builds the archives after checking that Linux and Windows CI passed for the
release's source commit. The [public install checks](https://github.com/grep-stack/mash/actions/workflows/release-check.yml)
show which installers ran and confirmed the expected version.

For releases that include a build attestation, GitHub CLI can verify which
workflow and source tag produced the archive. With GitHub CLI signed in,
replace `ARCHIVE` with the path to the file you downloaded and
`RELEASE_TAG` with its tag, including the leading `v`.

```sh
gh attestation verify "ARCHIVE" --repo grep-stack/mash --signer-workflow grep-stack/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

The release workflow checks the attestations before publishing those
archives. Releases through `v0.1.1-alpha` use the checksum check above.
