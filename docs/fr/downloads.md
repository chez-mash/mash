# Télécharger et vérifier Mash

Téléchargez les fichiers prêts à l’emploi, vérifiez leur somme de contrôle
et ouvrez le tableau de bord depuis le dossier où vous les extrayez.
Cette méthode évite d’exécuter le script d’installation. Vous pouvez aussi
[examiner le script](#inspect-the-installer) ou [utiliser un gestionnaire de paquets](install-methods.md).

<span id="choose-your-download"></span>

## Choisir le téléchargement

Les liens téléchargent les archives de la dernière version publiée.
Chaque archive inclut le programme de terminal et le tableau de bord web.

| Ordinateur | Téléchargement |
| --- | --- |
| Mac avec puce Apple | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/mashhq/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/mashhq/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/mashhq/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/mashhq/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

Consultez les [notes de version et le fichier des sommes de contrôle sur GitHub](https://github.com/mashhq/mash/releases/latest).
Pour vérifier une archive, utilisez le fichier `SHA256SUMS` de la même
version. Consultez les [prérequis pour Linux](installation.md#recommended-release-installers).
Sur une plateforme comme un Mac Intel, vous pouvez [compiler depuis le code source](install-methods.md#build-from-source).

<span id="verify-the-checksum"></span>

## Vérifier la somme de contrôle

Dans le dossier du téléchargement, exécutez la commande correspondant à
votre système. Sur Linux ARM64, remplacez le nom de l’archive par le vôtre.

**Linux**

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

**macOS**

```sh
shasum -a 256 mash-aarch64-apple-darwin.tar.gz
```

**PowerShell sous Windows**

```powershell
Get-FileHash -Algorithm SHA256 .\mash-x86_64-pc-windows-msvc.zip
```

Recherchez le nom exact de l’archive dans `SHA256SUMS` et comparez la somme
de cette ligne à celle qu’affiche la commande. Les majuscules et minuscules
sont équivalentes dans la somme. Si les valeurs correspondent, vous avez
bien le fichier publié.

<span id="use-the-archive"></span>

## Utiliser l’archive

Extrayez `mash` et `mash-server` (les deux fichiers `.exe` sous Windows)
dans un dossier qui vous appartient. Dans un terminal ouvert dans ce
dossier, lancez `./mash --version` sur macOS ou Linux, ou
`.\mash.exe --version` dans PowerShell. La commande affiche la version
téléchargée.

Depuis le même dossier, démarrez la recherche locale et ouvrez le tableau
de bord. La première commande télécharge Qdrant et démarre le service sous
votre compte. Vous pouvez l’ignorer si vous avez déjà un
[Qdrant local compatible](local-search.md#run-qdrant-yourself).
Sous Linux ou WSL, elle nécessite une
[session utilisateur systemd](local-search.md#linux-without-a-systemd-user-session).

**macOS ou Linux**

```sh
./mash qdrant install
./mash dash
```

**Windows PowerShell**

```powershell
.\mash.exe qdrant install
.\mash.exe dash
```

Laissez le terminal ouvert, puis ouvrez l’adresse qu’il affiche dans votre
navigateur, généralement [http://127.0.0.1:8790](http://127.0.0.1:8790).
Vérifiez les conversations que Mash pourra lire et la période retenue.
Pour effectuer vos propres recherches, laissez les agents déconnectés,
puis choisissez **Set up and index**.
[Étapes de configuration dans le navigateur](dashboard.md#set-up-mash-from-the-dashboard)

<span id="inspect-the-installer"></span>

## Examiner l’installateur

Avant de le lancer, lisez [l’installateur macOS et Linux](https://github.com/mashhq/mash/blob/main/install.sh)
ou [l’installateur Windows](https://github.com/mashhq/mash/blob/main/install.ps1).
Le script vérifie la somme de contrôle de l’archive et place Mash dans
votre dossier utilisateur sans droit d’administrateur.
[Ce que la configuration change](installation.md#what-setup-changes)

<span id="check-how-it-was-built"></span>

## Vérifier la provenance du fichier

Le [workflow public de publication](https://github.com/mashhq/mash/actions/workflows/release.yml)
construit les archives après avoir vérifié que les tests Linux et Windows
ont réussi pour le code de la version. Les
[tests publics d’installation](https://github.com/mashhq/mash/actions/workflows/release-check.yml)
indiquent quels installateurs ont été exécutés et ont confirmé la version attendue.

Pour les versions qui incluent une attestation de compilation, GitHub CLI
peut vérifier le workflow et le tag source de l’archive. Connectez-vous dans
GitHub CLI et remplacez `ARCHIVE` par le chemin du fichier téléchargé et
`RELEASE_TAG` par son tag, avec le `v` initial.

```sh
gh attestation verify "ARCHIVE" --repo mashhq/mash --signer-workflow mashhq/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

Le workflow vérifie les attestations avant de publier ces archives. Jusqu’à
`v0.1.1-alpha`, utilisez la somme de contrôle ci-dessus.

<!-- mash-docs-source: d089547ce72f1abd31d192459b684cc330176a9a9a11aeeadbb18065c6b8f2c4 -->
