# Autres façons d’installer Mash

Utilisez ce guide pour installer Mash avec un gestionnaire de paquets ou le compiler
depuis le code source. Après l’installation, [démarrez le service de recherche local](installation.md#start-the-local-search-service)
et [choisissez les historiques à lire](installation.md#connect-your-histories). Si Mash est
déjà installé, suivez les [étapes de mise à jour](upgrades.md#upgrade-an-existing-installation).

<span id="install-with-a-package-manager"></span>

## Installer avec un gestionnaire de paquets

Les paquets ci-dessous utilisent les versions précompilées de Mash. Ils incluent
le tableau de bord, la recherche dans le terminal et les outils utilisables par un
agent connecté. Une fois la recherche locale démarrée, vous pouvez [terminer la configuration dans le navigateur](installation.md#set-up-in-your-browser).
Avant l’installation, vous pouvez [vérifier les téléchargements publiés](downloads.md)
et [voir quels programmes d’installation ont passé les vérifications de chaque version](https://github.com/mashium/mash/actions/workflows/release-check.yml).

<span id="homebrew-macos-or-linux"></span>

### Homebrew sous macOS ou Linux

Sous macOS sur une puce Apple ou sous Linux sur x86-64 ou ARM64, installez Mash avec la
[formule Homebrew de Mashium](https://github.com/mashium/homebrew-tap/blob/main/Formula/mash.rb). Le nom complet indique à Homebrew quel dépôt utiliser et désigne la formule pour les [règles de confiance des taps Homebrew](https://docs.brew.sh/Tap-Trust).

```sh
brew install mashium/tap/mash
```

<span id="apt-debian-or-ubuntu"></span>

### apt sous Debian ou Ubuntu

Cette méthode fonctionne sous Debian 12 ou ultérieur et Ubuntu 22.04 ou ultérieur, sur x86-64 ou ARM64.
Le dépôt apt de Mashium publie des métadonnées de paquets signées. L’option
`signed-by` utilise la [clé de signature de Mashium](https://getmash.io/apt/mash-archive-keyring.gpg)
pour ce dépôt. Vous pouvez [examiner les métadonnées signées](https://getmash.io/apt/dists/stable/InRelease)
avant d’ajouter le dépôt.

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

Le paquet s’appelle `mash-cli` car Debian et Ubuntu proposent déjà un autre paquet nommé `mash`. La commande installée s’appelle toujours `mash`. Les deux paquets ne peuvent pas cohabiter. apt propose de retirer l’autre paquet.

<span id="scoop-windows"></span>

### Scoop sous Windows

Sous Windows x64, une fois Scoop installé, ouvrez PowerShell, ajoutez le dépôt de Mashium et installez son
[paquet Mash](https://github.com/mashium/mash/blob/main/bucket/mash.json). Le manifeste indique l’archive Windows et son empreinte SHA-256.

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

### mise

Une fois mise installé, utilisez son intégration GitHub pour installer Mash à partir des
[archives des versions de Mashium](https://github.com/mashium/mash/releases).

```sh
mise use -g github:mashium/mash
```

### cargo-binstall

Une fois cargo-binstall installé, indiquez le projet GitHub pour utiliser l’archive publiée par Mashium.

```sh
cargo binstall --strategies crate-meta-data --git https://github.com/mashium/mash mash
```

Si aucune archive n’est disponible pour votre plateforme, consultez la
[compilation depuis le code source](#build-from-source).

<span id="build-from-source"></span>

## Compiler depuis le code source

Suivez ces étapes si votre ordinateur ne dispose pas de version précompilée ou si vous voulez essayer une révision qui n’a pas encore été publiée. [Incluez le tableau de bord](#include-the-web-dashboard)
pour chercher dans le navigateur. Pour utiliser la [recherche dans le terminal et les outils pour agents](#cli-and-agent-tools), utilisez la commande Cargo plus courte.

<span id="include-the-web-dashboard"></span>

### Inclure le tableau de bord

Pour inclure le tableau de bord, générez ses fichiers web avant de compiler Mash.
Avec Git, [Node.js 24 ou supérieur](https://nodejs.org/en/download) et une version stable de
Rust, exécutez ces commandes dans un terminal.

```sh
git clone https://github.com/mashium/mash.git
cd mash
npx --yes pnpm@11.28.5 --dir dashboard install --frozen-lockfile
npx --yes pnpm@11.28.5 --dir dashboard run build
cargo +stable install --locked --path .
```

Pour ajouter le tableau de bord à une installation existante, revenez à la révision que vous aviez installée avant de compiler. Ensuite, [démarrez la recherche locale et indexez vos conversations](installation.md#start-the-local-search-service), puis lancez `mash dash`
et ouvrez l’adresse affichée.

<span id="cli-and-agent-tools"></span>

### Recherche dans le terminal et outils pour agents

Cette commande compile la recherche dans le terminal et les outils utilisables par un agent connecté. Pour inclure le tableau de bord, suivez les étapes de compilation ci-dessus.

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

Après la compilation, [démarrez la recherche locale et indexez vos conversations](installation.md#start-the-local-search-service).
Ensuite, [lancez votre première recherche dans le terminal](search.md#your-first-search).

Si le terminal ne trouve pas Mash après la compilation, consultez la procédure pour
[ajouter le dossier des exécutables de Cargo au chemin de recherche des commandes](troubleshooting.md#mash-command-not-found).

<!-- mash-docs-source: 400da6d93d5805098a5628678b37ade9f61dc9fdc1eedfb23cf768d2c4e84eec -->
