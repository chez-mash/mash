# Other ways to install Mash

Choose a [package manager](#install-with-a-package-manager) or
[build from source](#build-from-source). For the installer that starts the
dashboard for you, see the [quick start](../README.md#quick-start).
If you're replacing a version you already use, follow the
[upgrade steps](upgrades.md#upgrade-an-existing-installation).

## Install with a package manager

The packages below use Mash's prebuilt releases. They include the browser
dashboard, terminal search, and the tools a connected agent can use. After
installing a package, [start the local search service and finish setup](installation.md#start-the-local-search-service).
Before installing, you can [check the published release downloads](downloads.md)
and [see which installers passed for each release](https://github.com/grep-stack/mash/actions/workflows/release-check.yml).

### Homebrew, macOS or Linux

On Apple silicon macOS or Linux x86-64 or ARM64, install from
[GrepStack's Homebrew formula](https://github.com/grep-stack/homebrew-tap/blob/main/Formula/mash.rb).
The full name tells Homebrew which repository to use and selects the
individual formula for [Homebrew's tap trust](https://docs.brew.sh/Tap-Trust).

```sh
brew install grep-stack/tap/mash
```

### apt, Debian or Ubuntu

Use this on Debian 12+ or Ubuntu 22.04+ on x86-64 or ARM64.
The GrepStack apt repository publishes signed package metadata. The
`signed-by` setting uses [GrepStack's signing key](https://getmash.io/apt/mash-archive-keyring.gpg)
for this repository. You can [inspect the signed metadata](https://getmash.io/apt/dists/stable/InRelease)
before adding it.

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

The package is `mash-cli` because Debian and Ubuntu already have an unrelated
package named `mash`; the command it installs is still `mash`. The two cannot
be installed together, and apt offers to remove the other one.

### Scoop, Windows

On Windows x64 or ARM64 with Scoop installed, open PowerShell, add GrepStack's bucket
and install its [Mash package](https://github.com/grep-stack/mash/blob/main/bucket/mash.json).
The manifest lists the Windows archive and its SHA-256 hash.

```powershell
scoop bucket add grep-stack https://github.com/grep-stack/mash
scoop install grep-stack/mash
```

### mise

With mise installed, use its GitHub backend to install from the
[GrepStack release archives](https://github.com/grep-stack/mash/releases).

```sh
mise use -g github:grep-stack/mash
```

### cargo-binstall

With cargo-binstall installed, specify the GitHub project to select GrepStack's
release archive.

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/grep-stack/mash mash
```

To try a revision that isn't released yet, see
[build from source](#build-from-source).

## Build from source

Use these instructions to try an unreleased revision. Building works on the
same systems as the releases, since Mash's search models need a prebuilt ONNX
Runtime for the platform. [Include the web dashboard](#include-the-web-dashboard)
to search in a browser. For [terminal search and agent tools](#cli-and-agent-tools),
use the shorter Cargo install.

### Include the web dashboard

To include the dashboard, build its web files before compiling Mash. With
Git, [Node.js 24+](https://nodejs.org/en/download), and a stable Rust toolchain,
run these commands in a terminal.

```sh
git clone https://github.com/grep-stack/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo +stable install --locked --path .
```

When adding the dashboard to an existing installation, check out the revision
you installed before building. After building, [start the local search service](installation.md#start-the-local-search-service)
and follow the browser setup steps.

### CLI and agent tools

This command builds terminal search and the tools that a connected agent can
use. For the browser dashboard, follow the build steps above.

```sh
cargo +stable install --locked --git https://github.com/grep-stack/mash mash
mash --version
```

After building, [start the local search service](installation.md#start-the-local-search-service),
then follow the terminal setup steps.

If your terminal cannot find Mash after either build, follow
[how to add Cargo's binary directory to your command search path](troubleshooting.md#mash-command-not-found).
