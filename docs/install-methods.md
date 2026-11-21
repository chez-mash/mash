# Other ways to install Mash

Use this guide to install Mash with a package manager or build it from source.
After installing, [start the local search service](installation.md#start-the-local-search-service)
and [choose which histories to read](installation.md#connect-your-histories).
If Mash is already installed, follow the [upgrade steps](upgrades.md#upgrade-an-existing-installation).

## Install with a package manager

The packages below use Mash's prebuilt releases. They include the browser
dashboard, terminal search, and the tools a connected agent can use.
Once local search is running, you can [finish setup in the browser](installation.md#set-up-in-your-browser).
Before installing, you can [check the published release downloads](downloads.md)
and [see which installers passed for each release](https://github.com/mashium/mash/actions/workflows/release-check.yml).

### Homebrew, macOS or Linux

On Apple silicon macOS or Linux x86-64 or ARM64, install from
[Mashium's Homebrew formula](https://github.com/mashium/homebrew-tap/blob/main/Formula/mash.rb).
The full name tells Homebrew which repository to use and selects the
individual formula for [Homebrew's tap trust](https://docs.brew.sh/Tap-Trust).

```sh
brew install mashium/tap/mash
```

### apt, Debian or Ubuntu

Use this on Debian 12+ or Ubuntu 22.04+ on x86-64 or ARM64.
The Mashium apt repository publishes signed package metadata. The
`signed-by` setting uses [Mashium's signing key](https://getmash.io/apt/mash-archive-keyring.gpg)
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

On Windows x64 with Scoop installed, open PowerShell, add Mashium's bucket
and install its [Mash package](https://github.com/mashium/mash/blob/main/bucket/mash.json).
The manifest lists the Windows archive and its SHA-256 hash.

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

### mise

With mise installed, use its GitHub backend to install from the
[Mashium release archives](https://github.com/mashium/mash/releases).

```sh
mise use -g github:mashium/mash
```

### cargo-binstall

With cargo-binstall installed, specify the GitHub project to select Mashium's
release archive.

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/mashium/mash mash
```

If a release archive is unavailable for your platform, see
[build from source](#build-from-source).

## Build from source

Use these instructions on a machine without a prebuilt release or to try an
unreleased revision. [Include the web dashboard](#include-the-web-dashboard)
to search in a browser. For [terminal search and agent tools](#cli-and-agent-tools),
use the shorter Cargo install.

### Include the web dashboard

To include the dashboard, build its web files before compiling Mash. With
Git, [Node.js 24+](https://nodejs.org/en/download), and a stable Rust toolchain,
run these commands in a terminal.

```sh
git clone https://github.com/mashium/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo +stable install --locked --path .
```

When adding the dashboard to an existing installation, check out the revision
you installed before building. After building, [start local search and index
your chats](installation.md#start-the-local-search-service), then run `mash dash`
and open the address it prints.

### CLI and agent tools

This command builds terminal search and the tools that a connected agent can
use. For the browser dashboard, follow the build steps above.

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

After building, [start local search and index your chats](installation.md#start-the-local-search-service).
Then [try your first terminal search](search.md#your-first-search).

If your terminal cannot find Mash after either build, follow
[how to add Cargo's binary directory to your command search path](troubleshooting.md#mash-command-not-found).
