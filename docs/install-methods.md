# Other ways to install Mash

Use this guide to install Mash with a package manager or build it from source.
After installing, [start the local search service](installation.md#start-the-local-search-service)
and [choose which histories to read](installation.md#connect-your-histories).

## Install with a package manager

### Homebrew, macOS or Linux

```sh
brew install mashium/tap/mash
```

### apt, Debian or Ubuntu

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

The package is `mash-cli` because Debian and Ubuntu already have an unrelated
package named `mash`; the command it installs is still `mash`. The two cannot
be installed together, and apt offers to remove the other one.

### Scoop, Windows

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

### mise

```sh
mise use -g github:mashium/mash
```

### cargo-binstall

```sh
cargo binstall --git https://github.com/mashium/mash mash
```

## Build from source

Use a stable Rust toolchain to build for a machine with no prebuilt release
or to try an unreleased revision. The first build takes several minutes.

### CLI and agent tools

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

If the shell cannot find `mash`, add Cargo's binary directory to your path.
On macOS or Linux, run `export PATH="$HOME/.cargo/bin:$PATH"`. In PowerShell,
run `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`. Add the directory
to your shell profile or Windows environment settings to keep it.

### Include the web dashboard

Build the web files before compiling Mash to include them in the executable.
Install [Node.js 24+](https://nodejs.org/en/download) and a stable Rust
toolchain, then run these commands from a terminal.

```sh
git clone https://github.com/mashium/mash.git
cd mash
npm install --global pnpm@11.28.5
pnpm --dir dashboard install --frozen-lockfile
pnpm --dir dashboard run build
cargo +stable install --locked --path .
```

If you already use Mash, check out the same revision you installed before
building to avoid a database format mismatch. The `cargo install --git`
command in the previous section installs Mash without the dashboard files.
