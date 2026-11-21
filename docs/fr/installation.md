# Installer Mash

Installez Mash, démarrez le service de recherche et choisissez les
conversations dans lesquelles chercher. Par défaut, l’index et les recherches
restent sur votre ordinateur. [Comment Mash traite votre historique](privacy.md)

Vous pouvez [essayer la démo avec des conversations d’exemple](https://getmash.io/demo/).
Pour utiliser vos propres données, [choisissez un historique et essayez le tableau de bord local](privacy.md#essayer-mash-avec-un-historique-limité).

Le [guide de démarrage rapide](quick-start.md) propose une seule commande
pour installer Mash, démarrer le service de recherche et ouvrir l’assistant
de configuration. Si Mash est déjà installé, [démarrez le service de recherche](#start-the-local-search-service).
S’il est déjà en cours d’exécution, [configurez vos historiques](#connect-your-histories).

Pour passer à une nouvelle version, suivez les [étapes de mise à jour (en anglais)](../upgrades.md#upgrade-an-existing-installation).

<span id="install-the-cli"></span>

<span id="get-mash-for-your-system"></span>

## Télécharger Mash pour votre système

La version précompilée comprend le tableau de bord web. Après la configuration,
lancez `mash dash` et ouvrez l’adresse affichée. Vous pouvez aussi
[connecter des agents de programmation](agents.md) pendant la configuration
pour qu’ils cherchent dans vos conversations.

<span id="recommended-release-installers"></span>

### Installateurs recommandés

Les versions précompilées couvrent Linux x86-64 et ARM64, macOS sur puce
Apple et Windows x64. Choisissez la commande correspondant à votre système.
Vous pouvez [examiner l’installateur ou vérifier le fichier téléchargé](#check-the-release-download)
au préalable.

Sous Linux, les exécutables nécessitent glibc 2.34 ou une version ultérieure,
disponible dans Ubuntu 22.04, Debian 12, RHEL 9 et leurs versions ultérieures.
Alpine et les autres systèmes utilisant musl ne sont pas pris en charge.

<span id="macos-or-linux"></span>

#### macOS ou Linux

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

Utilisez la même commande sous WSL.

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

Les deux scripts vérifient la somme de contrôle et installent Mash dans
`.local/bin`, dans votre dossier personnel. Ils affichent les commandes
suivantes avec le chemin complet de l’exécutable. Vous pouvez ainsi les
utiliser avant d’ajouter le dossier à `PATH`. Définissez `MASH_VERSION` pour
choisir une version publiée ou `MASH_INSTALL_DIR` pour choisir un autre
dossier. Les scripts conservent votre profil de terminal. Si le terminal ne
trouve pas `mash`, consultez [la procédure pour retrouver la commande](troubleshooting.md#mash-command-not-found).

<span id="package-manager-alternatives"></span>

### Installer avec un gestionnaire de paquets

Vous pouvez utiliser [Homebrew, apt, Scoop, mise ou cargo-binstall](install-methods.md#install-with-a-package-manager).
Après l’installation, [démarrez le service de recherche](#start-the-local-search-service).

<span id="build-from-source"></span>

### Compiler depuis le code source

Pour chercher depuis le navigateur, [compilez Mash avec le tableau de bord](install-methods.md#include-the-web-dashboard).
Pour [la recherche dans le terminal et les outils des agents](install-methods.md#cli-and-agent-tools),
utilisez la commande Cargo plus courte.

<span id="start-the-local-search-service"></span>

## Démarrer le service de recherche local

Sous Linux ou WSL, la commande ci-dessous nécessite une session utilisateur
systemd. [Vérifiez votre session ou consultez les autres options](local-search.md#linux-without-a-systemd-user-session).
Si vous utilisez déjà un [Qdrant local compatible](local-search.md#run-qdrant-yourself),
passez à la [configuration des historiques](#connect-your-histories).

Sous Windows, Mash conserve les données de recherche de Qdrant dans un
dossier au chemin court sur le disque système, généralement
`C:\mash-qdrant\storage`. [Fichiers et journaux de Qdrant](local-search.md#where-qdrant-stores-data)

```sh
mash qdrant install
```

Mash vérifie le téléchargement de Qdrant (environ 30 Mo) et lance le service
en arrière-plan sous votre compte. Qdrant écoute sur cet ordinateur et
redémarre à l’ouverture de votre session. Sous Windows, il faut ouvrir une
session sur le bureau. Si le service ne démarre pas, consultez
[la procédure pour lire les journaux et le redémarrer](local-search.md#check-or-restart-the-service).

<span id="run-qdrant-yourself"></span>

### Exécuter Qdrant vous-même

Vous pouvez [réutiliser une instance locale de Qdrant 1.20 ou ultérieure](local-search.md#run-qdrant-yourself).
Pour en démarrer une, choisissez la procédure adaptée à votre environnement.

- <span id="optional-qdrant-docker-image"></span>[Qdrant dans Docker](local-search.md#optional-qdrant-docker-image)
- <span id="native-windows-without-docker"></span>[Windows sans Docker](local-search.md#native-windows-without-docker)
- <span id="native-macos-or-linux-without-docker"></span>[macOS ou Linux sans Docker](local-search.md#native-macos-or-linux-without-docker)

<span id="connect-your-histories"></span>

## Configurer vos historiques

```sh
mash init
```

Lors d’une première configuration, l’assistant affiche les dossiers trouvés,
la période retenue (les fichiers modifiés dans les 90 derniers jours) et les
agents qu’il peut connecter. Si l’assistant vous demande où chercher,
choisissez **Just this computer (recommended)** pour conserver l’index sur cet
ordinateur. **Set up and index** applique les options du récapitulatif, y
compris la connexion des agents sélectionnés. Mash peut lire les conversations
enregistrées même lorsque la commande de l’agent est absente.

Choisissez **Customize setup** pour modifier la période ou le dossier d’un
historique, choisir des intégrations précises, connecter un serveur partagé
existant ou enregistrer les réglages sans indexer. Cette option permet aussi
d’[extraire des éléments de mémoire de vos anciennes conversations (en anglais)](../records.md#backfill-earlier-memory)
après l’indexation. Pour cela, Mash utilise votre session Claude Code ou Codex
et envoie des extraits choisis de vos conversations au service de modèles de
l’agent.

Échap ou Ctrl-C annule avant toute modification des réglages de Mash ou de
vos agents. Si vous demandez l’indexation, l’assistant vérifie que le service
de recherche local répond en HTTP. Si vous partagez l’index, il vérifie aussi
l’authentification du serveur. En cas d’échec, vous pouvez enregistrer vos
réglages et reprendre la configuration plus tard.

Si vous avez connecté un agent, redémarrez-le pour charger Mash. Les hooks
du plugin d’historique nécessitent Python 3.9 ou ultérieur.
[Utiliser Mash avec votre agent](agents.md)

<span id="skip-agent-connections"></span>

### Ignorer la connexion aux agents

Vous pouvez chercher dans vos conversations depuis le tableau de bord ou le
terminal sans donner à un agent l’accès à la recherche Mash. Suivez l’option
que propose l’assistant.

- Si le premier récapitulatif propose **Set up and index without connecting agents**,
  choisissez cette option pour indexer les historiques affichés.
- Si l’assistant demande **Which agents should get access to Mash search?**,
  décochez tous les agents avec la barre d’espace, puis appuyez sur Entrée.
  S’il propose d’extraire des souvenirs d’anciennes conversations, refusez.
- Si le récapitulatif indique qu’aucun agent ne sera connecté, choisissez **Set up and index**.

Lorsque la recherche est prête, [ouvrez le tableau de bord](#try-your-first-search).
Vous pourrez [connecter un agent plus tard](agents.md#connect-claude-code-codex-or-copilot).

<span id="try-your-first-search"></span>

## Lancer une première recherche

Si votre installation comprend le tableau de bord, lancez `mash dash` et
ouvrez l’adresse affichée. Dans **chats**, saisissez un mot dont vous vous
souvenez et ouvrez un résultat pour lire le message d’origine.

Vous pouvez aussi [chercher dans le terminal](search.md#your-first-search)
ou [demander à un agent connecté](agents.md#try-it). Si une ancienne
conversation manque, [élargissez la période indexée (en anglais)](../indexing.md#add-older-history).

La recherche par mots fonctionne sans modèles. Pour chercher par le sens,
suivez [la procédure de configuration des modèles facultatifs (en anglais)](../indexing.md#enable-semantic-search).

<span id="setup-options"></span>

## Options de configuration

<span id="without-the-wizard"></span>

### Sans l’assistant

`mash init --yes` accepte les choix détectés. Il couvre les fichiers modifiés
dans les 90 derniers jours, y compris les sessions en cours de tous les agents
compatibles détectés, dont Copilot. Il applique les intégrations disponibles
sans extraire de mémoire des anciennes conversations. Si l’entrée ou la
sortie d’erreur du processus n’est pas reliée à un terminal, la commande
s’exécute aussi sans interaction. Le résultat JSON indique les dossiers
sélectionnés.

Sans l’assistant, `mash init` vérifie Qdrant avant toute modification. Si
Qdrant ne répond pas, la commande indique comment le démarrer et conserve
les réglages de Mash et de vos agents.

| Option | Effet |
| --- | --- |
| `--no-index` | Enregistre les réglages sans indexer ni vérifier Qdrant. La commande indique comment démarrer Qdrant. Pour choisir vous-même les dossiers, modifiez la [configuration (en anglais)](../configuration.md), puis lancez `mash index`. |
| `--no-agents` | Permet de chercher dans le tableau de bord ou le terminal sans connecter d’agents. |
| `--force` | Sans cette option, le mode non interactif conserve la configuration existante. Avec `--yes`, elle recrée la configuration à partir des historiques détectés. Dans l’assistant, elle ouvre directement la vérification des réglages. |

Sans `--force`, l’assistant vous laisse conserver les réglages existants ou
les revoir. La révision garde les paramètres qui ne figurent pas dans
l’assistant et sauvegarde le fichier précédent.

Pour lire l’historique avant de démarrer Qdrant, lancez `mash init --no-index`,
puis `mash index`. Mash sauvegarde les conversations sur cet ordinateur avant
d’avoir besoin de Qdrant. Il s’arrête en indiquant le nombre de conversations
sauvegardées et les commandes pour démarrer le service. Vous pouvez les lire
avec `mash list-chats` et `mash view`. Une fois Qdrant démarré, relancez
`mash index` pour mettre la recherche à jour.

<span id="read-the-result"></span>

### Lire le résultat

L’assistant termine par un bref bilan et la prochaine étape. Quand la
recherche est prête, il indique comment ouvrir le tableau de bord ou chercher
dans le terminal, selon la version installée. Pour obtenir le résultat JSON
complet, lancez `mash init --yes` ou redirigez la sortie standard de l’assistant
avec `mash init > setup.json`. Les questions et les progrès s’affichent sur
la sortie d’erreur.

- `"agents": []` indique qu’aucun agent n’a été connecté, même si
  l’indexation a abouti. Lisez `next` pour en connaître la raison et suivez
  [la configuration des agents](agents.md). La recherche dans le terminal
  fonctionne sans agent connecté.
- Pour chaque agent connecté, `files` indique les fichiers modifiés par sa
  commande et `undo` donne les commandes pour retirer la connexion.
- `index_error` indique que la première indexation a échoué après
  l’application de la configuration, par exemple si Qdrant s’arrête ou si
  son port gRPC est inaccessible. Mash signale l’erreur et conserve les
  réglages, les connexions et l’historique déjà enregistré. Corrigez le
  problème, puis lancez `mash index`.
- `written: false` lors d’une nouvelle exécution indique que la configuration
  existante a été réutilisée.
- Le nombre de messages consultables par la recherche compte uniquement le
  dialogue avec l’agent. Le raisonnement et la sortie des outils restent
  lisibles dans la conversation, ce qui peut expliquer un total beaucoup
  plus élevé dans la conversation.

<span id="check-the-release-download"></span>

## Vérifier le téléchargement

Vous pouvez lire [l’installateur macOS et Linux](https://github.com/mashium/mash/blob/main/install.sh)
ou [l’installateur Windows](https://github.com/mashium/mash/blob/main/install.ps1)
avant de le lancer. Les scripts installent Mash dans votre dossier utilisateur
sans droits administrateur. Avec la commande unique, le script installe et
démarre Qdrant avant d’ouvrir l’assistant. Vous pouvez y vérifier les
historiques et les connexions avant de les appliquer. Si vous annulez dans
l’assistant, utilisez [l’aperçu de désinstallation](#uninstall) pour voir ce
que le script a déjà installé.

Le [workflow public de publication](https://github.com/mashium/mash/actions/workflows/release.yml)
construit les archives après avoir vérifié que la révision source a passé
les tests Linux et Windows. Consultez les
[tests publics d’installation](https://github.com/mashium/mash/actions/workflows/release-check.yml)
pour savoir quels installateurs ont passé les tests pour votre version.

Pour installer Mash sans exécuter un script téléchargé, récupérez l’archive
de votre plateforme et `SHA256SUMS` depuis la même
[version publiée sur GitHub](https://github.com/mashium/mash/releases).
Comparez la somme SHA-256 de l’archive à la ligne de son nom de fichier
exact avant de l’extraire. Par exemple, sous Linux x86-64, utilisez la
commande suivante.

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

Sous macOS, utilisez `shasum -a 256` avec l’archive macOS. Sous Windows,
utilisez `Get-FileHash -Algorithm SHA256` avec le ZIP Windows dans
PowerShell. Si la somme correspond à celle publiée, extrayez `mash` et
`mash-server` (ou les deux fichiers `.exe`) dans un dossier à vous, puis
[démarrez le service de recherche](#start-the-local-search-service).

Si la version comprend une attestation de compilation, GitHub CLI peut aussi
vérifier le workflow et l’étiquette source de l’archive. Remplacez `ARCHIVE`
par le fichier téléchargé et `RELEASE_TAG` par son étiquette, avec le `v`
initial.

```sh
gh attestation verify ARCHIVE --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

Le workflow vérifie les attestations avant de publier les archives.
Pour les versions jusqu’à `v0.1.1-alpha`, utilisez la somme SHA-256 ci-dessus.

<span id="uninstall"></span>

## Désinstaller Mash

```sh
mash uninstall          # lists what would be removed and changes nothing
mash uninstall --yes    # removes it
```

Lisez l’aperçu dans votre terminal. Il montre quels services, connexions
et fichiers locaux seraient retirés, ainsi que la taille de la copie de
vos conversations. Si vous redirigez la sortie ou utilisez une ancienne
version qui affiche du JSON dans le terminal, lisez `paths`, `kept` et `next`.

Avec `--yes`, Mash effectue les opérations suivantes.

1. Il arrête et retire les services qu’il a mis en place, dont le service de
   recherche et ses données, le [tableau de bord distant](dashboard.md) et
   le service de mise à jour automatique de l’index s’il est installé.
2. Il déconnecte Claude Code, Codex et GitHub Copilot via leurs commandes,
   décrites dans [la procédure pour déconnecter un agent](agents.md#disconnect-an-agent).
3. Il supprime son index (la copie de vos conversations), les modèles, les
   réglages et les jetons enregistrés.

Pour retirer le programme lui-même, suivez la dernière instruction du
bilan. En JSON, elle figure dans `binary.remove`. Il peut s’agir de
`brew uninstall mash`, `scoop uninstall mash`, `sudo apt remove mash-cli`,
`cargo uninstall mash` ou de fichiers à supprimer après une installation
par script.

Les historiques d’origine des agents et les données déjà publiées dans un
[index partagé](shared-index.md) restent en place. Mash
signale et conserve les fichiers de `~/.mash` qu’il n’a pas créés. Si vous
gérez votre propre Qdrant, son service et les données de recherche qu’il
contient restent en place. Gérez ces données séparément.

Pour une réinstallation ultérieure, `--keep-history` conserve la copie
consultable de vos conversations, la configuration, les jetons enregistrés
et les données de recherche dans le Qdrant géré par Mash. Mash retire quand
même les services, les connexions et les modèles téléchargés. Si vous voulez
une sauvegarde distincte, utilisez [`mash export DIRECTORY` (en anglais)](../upgrades.md).

Affichez d’abord l’aperçu avec `mash uninstall --keep-history`, puis
confirmez avec `mash uninstall --yes --keep-history`.

Le bilan indique chaque étape en échec et sa raison. La commande renvoie le
code 1 si une étape échoue. Relancez-la avec les mêmes options pour terminer
les étapes restantes. Redémarrez ensuite vos agents pour qu’ils cessent de
charger Mash.

Pour retirer uniquement le service de recherche, lancez `mash qdrant uninstall`.
La commande conserve les données de Qdrant sauf si vous ajoutez
`--delete-storage`. Ces données servent à la recherche. Vous pouvez les
reconstruire à partir de votre index avec `mash index --rebuild-local-views --full`.

<!-- mash-docs-source: e9fdc82da63b901e9fbc86e2c87b4aa88f83b1670d47a01b3094284023646838 -->
