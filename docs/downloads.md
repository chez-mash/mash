# Download and verify Mash

Download the ready-to-run files for your computer, check them against the
published checksum, and open Mash from the folder where you extract them.
You can also inspect the setup script or [install with a package manager](install-methods.md).

## Choose your download

These links download the latest published archives. Each archive contains
the terminal program and browser dashboard.

| Computer | Download |
| --- | --- |
| Mac with Apple silicon | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

See the [release notes and checksum file on GitHub](https://github.com/mashium/mash/releases/latest).
Use the archive and `SHA256SUMS` from the same release page when checking a file.
For Linux requirements, see [supported systems](installation.md#recommended-release-installers).
For a platform such as an Intel Mac, see [build from source](install-methods.md#build-from-source).

## Verify the checksum

From the folder containing the download, run the command for your system.
On Linux ARM64, substitute your archive's filename.

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

Then [start the local search service](installation.md#start-the-local-search-service)
and choose which chats Mash can read. Use the full path to `mash` in those
commands or [add its directory to your PATH](troubleshooting.md#mash-command-not-found).

## Inspect the installer

Read the [macOS and Linux installer](https://github.com/mashium/mash/blob/main/install.sh)
or the [Windows installer](https://github.com/mashium/mash/blob/main/install.ps1)
before you run it. The scripts verify the archive checksum and put Mash in
your user directory without administrator privileges.
[See what setup changes](installation.md#what-setup-changes)

## Check how it was built

The [public release workflow](https://github.com/mashium/mash/actions/workflows/release.yml)
builds the archives after checking that Linux and Windows CI passed for the
release's source commit. The [public install checks](https://github.com/mashium/mash/actions/workflows/release-check.yml)
show which installers ran and confirmed the expected version.

For releases that include a build attestation, GitHub CLI can verify which
workflow and source tag produced the archive. With GitHub CLI signed in,
replace `ARCHIVE` with the path to the file you downloaded and
`RELEASE_TAG` with its tag, including the leading `v`.

```sh
gh attestation verify "ARCHIVE" --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

The release workflow checks the attestations before publishing those
archives. Releases through `v0.1.1-alpha` use the checksum check above.
