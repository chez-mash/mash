# Télécharger et vérifier Mash

Téléchargez les fichiers prêts à l’emploi, vérifiez leur somme de contrôle
et lancez Mash depuis le dossier où vous les extrayez. Vous pouvez aussi
lire le script d’installation ou [utiliser un gestionnaire de paquets](install-methods.md).

<span id="choose-your-download"></span>

## Choisir le téléchargement

Les liens téléchargent les archives de la dernière version publiée.
Chaque archive inclut le programme de terminal et le tableau de bord web.

| Ordinateur | Téléchargement |
| --- | --- |
| Mac avec puce Apple | [`mash-aarch64-apple-darwin.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-apple-darwin.tar.gz) |
| Linux x86-64 | [`mash-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [`mash-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/mashium/mash/releases/latest/download/mash-aarch64-unknown-linux-gnu.tar.gz) |
| Windows x64 | [`mash-x86_64-pc-windows-msvc.zip`](https://github.com/mashium/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip) |

Consultez les [notes de version et le fichier des sommes de contrôle sur GitHub](https://github.com/mashium/mash/releases/latest).
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

[Démarrez ensuite le service de recherche local](installation.md#start-the-local-search-service)
et choisissez quelles conversations Mash peut lire. Utilisez le chemin
complet de `mash` dans les commandes ou
[ajoutez son dossier à votre PATH](troubleshooting.md#mash-command-not-found).

<span id="inspect-the-installer"></span>

## Examiner l’installateur

Avant de le lancer, lisez [l’installateur macOS et Linux](https://github.com/mashium/mash/blob/main/install.sh)
ou [l’installateur Windows](https://github.com/mashium/mash/blob/main/install.ps1).
Le script vérifie la somme de contrôle de l’archive et place Mash dans
votre dossier utilisateur sans droit d’administrateur.
[Ce que la configuration change](installation.md#what-setup-changes)

<span id="check-how-it-was-built"></span>

## Vérifier la provenance du fichier

Le [workflow public de publication](https://github.com/mashium/mash/actions/workflows/release.yml)
construit les archives après avoir vérifié que les tests Linux et Windows
ont réussi pour le code de la version. Les
[tests publics d’installation](https://github.com/mashium/mash/actions/workflows/release-check.yml)
indiquent quels installateurs ont été exécutés et ont confirmé la version attendue.

Pour les versions qui incluent une attestation de compilation, GitHub CLI
peut vérifier le workflow et le tag source de l’archive. Connectez-vous dans
GitHub CLI et remplacez `ARCHIVE` par le chemin du fichier téléchargé et
`RELEASE_TAG` par son tag, avec le `v` initial.

```sh
gh attestation verify "ARCHIVE" --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

Le workflow vérifie les attestations avant de publier ces archives. Jusqu’à
`v0.1.1-alpha`, utilisez la somme de contrôle ci-dessus.

<!-- mash-docs-source: 594b9d6aebb6145cc4183bc3c653c9262c2e692f6fbe4963c3e44038960682c0 -->
