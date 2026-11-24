# Mettre Mash à niveau

Une mise à niveau conserve vos réglages et vos conversations indexées.
Consultez les [notes de version (en anglais)](https://github.com/chez-mash/mash/releases),
faites une sauvegarde et utilisez votre méthode d’installation habituelle.
Pour sauvegarder ou récupérer une conversation, voyez
[comment créer une sauvegarde portable](backup.md#make-a-portable-backup) ou
[restaurer une copie à part](backup.md#restore-without-replacing-your-current-history).

<span id="upgrade-an-existing-installation"></span>

## Mettre à niveau une installation existante

Si vous publiez l’historique de plusieurs ordinateurs, consultez
[les étapes pour l’index partagé](#upgrade-a-shared-index) avant la mise à niveau.

1. **Interrompez l’indexation et fermez les agents connectés.** Arrêtez
   les processus `mash index --watch` et laissez les indexations en cours
   se terminer. Arrêtez le tableau de bord lancé dans un terminal avec Ctrl-C.
2. **Sauvegardez avec votre version actuelle.** Gardez son exécutable ou son
   archive jusqu’à ce que la nouvelle version fonctionne. Exportez dans un
   nouveau dossier que vous pouvez garder privé.

   ```sh
   mash --version
   mash export "/absolute/path/to/new-backup"
   ```

3. **[Installez la nouvelle version](#install-the-new-version)** avec votre
   méthode habituelle. Relancez `mash --version` pour vérifier que votre
   terminal trouve le nouvel exécutable.
4. **Mettez Qdrant à niveau si la version l’exige.** Relancez
   `mash qdrant install` pour mettre à niveau le service installé par Mash
   en conservant ses données. Si vous gérez Qdrant vous-même, gardez le même
   espace de stockage.
5. **Actualisez l’index.** Si le format de la base peut être mis à niveau,
   cette commande le migre avant l’indexation. Les commandes de lecture
   signalent qu’une mise à niveau du format est nécessaire.

   ```sh
   mash index
   ```

6. **Vérifiez la recherche et rouvrez Mash.** Vérifiez que `mash status`
   indique `"ready": true`, puis recherchez un mot d’une conversation
   indexée. Remplacez le texte d’exemple par un mot dont vous vous souvenez.

   ```sh
   mash status
   mash search 'a word you remember' --local --intent exact-mention
   ```

   Redémarrez les agents connectés et le suivi de l’index que vous aviez
   arrêté. Relancez le tableau de bord local avec `mash dash`. Pour le
   [tableau de bord à distance](dashboard.md#open-it-from-your-phone-or-another-computer),
   relancez `mash dash remote` avec les mêmes options pour faire tourner le
   service avec le nouvel exécutable.
   Si un agent a du mal à trouver Mash, [vérifiez le module et sa connexion](agents.md#confirm-the-connection).

Mash garde les identifiants de conversations et de messages et réutilise les
données de recherche compatibles. L’indexation ajoute aussi l’historique
enregistré depuis le dernier passage.

<span id="install-the-new-version"></span>

### Installer la nouvelle version

Utilisez votre méthode habituelle. Si vous avez utilisé le script shell
ou PowerShell, relancez [l’installateur de base](installation.md#recommended-release-installers)
avec le même dossier d’installation. Si vous avez défini `MASH_VERSION`,
indiquez la version voulue.

| Méthode d’installation | Comment mettre à niveau |
| --- | --- |
| Homebrew | `brew update && brew upgrade chez-mash/tap/mash` |
| apt | `sudo apt update && sudo apt install --only-upgrade mash-cli` |
| Scoop | Lancez `scoop update`, puis `scoop update mash`. |
| mise | `mise upgrade github:chez-mash/mash` |
| cargo-binstall | Reprenez [l’installation de l’archive précompilée](install-methods.md#cargo-binstall). |
| Code source | [Compilez la révision voulue](install-methods.md#build-from-source) avec les fichiers web si vous utilisez le tableau de bord. |

Si vous utilisiez le premier paquet Debian sous le nom `mash`, suivez
[les étapes pour apt](install-methods.md#apt-debian-or-ubuntu) pour passer
à `mash-cli`. Si vous aviez fixé une version dans mise, remplacez-la par
celle que vous souhaitez installer.

Si `mash --version` affiche encore l’ancienne version, vérifiez l’exécutable
que trouve le terminal avec `command -v mash` sur macOS ou Linux, ou avec
`(Get-Command mash).Source` dans PowerShell. Utilisez le chemin du nouvel
exécutable pour refaire les vérifications précédentes.

<span id="if-the-index-format-changes"></span>

### Si le format de l’index change

`mash --version-info` affiche le format produit par le nouvel exécutable
(`silver_schema`) et le plus ancien qu’il sait mettre à niveau
(`silver_schema_min`). Si `mash index` signale un format trop ancien,
voyez [comment remplacer un index incompatible](#replace-an-index-that-cannot-be-upgraded).
Si le format est plus récent, utilisez l’exécutable qui l’a écrit ou une
version ultérieure compatible. Redémarrez les agents, tableaux de bord
et processus de suivi de l’index qui utilisent encore l’ancien exécutable.

**La migration supprime les questions ouvertes enregistrées.** Si l’index
date d’avant l’abandon de ce type d’élément de mémoire, les questions,
leurs révisions et les liens qui les résolvaient sont supprimés. Les
décisions, préférences et faits restent disponibles. L’export conserve les
questions ouvertes. Gardez la version qui a écrit l’export si vous devez
les relire plus tard.

<span id="upgrade-a-shared-index"></span>

### Mettre à niveau un index partagé

Si vous [partagez l’historique entre ordinateurs](shared-index.md), mettez
le serveur et tous les ordinateurs à la même version. Des versions différentes
refusent de se synchroniser. Gardez les données de recherche pour reprendre
la synchronisation quand tous les ordinateurs auront la même version.

Sauvegardez le stockage et la configuration du serveur avant la mise à
niveau. Sur un ordinateur, `mash export` sauvegarde son propre index.
Sur le serveur, mettez Qdrant à niveau si nécessaire et démarrez le nouveau
`mash-server`. Il met à niveau chaque base d’ordinateur quand il l’ouvre.
Reprenez l’indexation partagée quand le serveur et les ordinateurs qui
publient sont à la même version.

<span id="make-a-portable-backup"></span>

## Faire une sauvegarde portable

[Exportez les conversations indexées dans un dossier privé](backup.md#make-a-portable-backup).

<span id="restore-without-replacing-your-current-history"></span>

## Restaurer sans remplacer l’historique actuel

[Vérifiez une copie restaurée dans une base de données séparée](backup.md#restore-without-replacing-your-current-history).

<span id="replace-an-index-that-cannot-be-upgraded"></span>

## Remplacer un index qui ne peut pas être mis à niveau

Suivez ces étapes si Mash signale un index trop ancien. L’indexation
ordinaire ne peut pas le mettre à niveau. Gardez l’ancien exécutable et
l’ancien index jusqu’à ce que le remplacement fonctionne.

1. Exportez avec la version qui a écrit l’index, puis importez avec la
   nouvelle version à un autre emplacement. Passez `--db PATH` à
   l’export si l’ancien index utilisait un chemin personnalisé.

   ```sh
   /path/to/old/mash export "/absolute/path/to/new-backup"
   /path/to/new/mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/staged.sqlite3" --keep-identity
   /path/to/new/mash --db "/absolute/path/to/staged.sqlite3" list-chats --local --limit 5
   ```

   Certaines anciennes sauvegardes ne peuvent pas non plus être importées.
   Si l’importation échoue, conservez les données et l’exécutable d’origine.
2. `--keep-identity` conserve l’identité de l’ordinateur dans un index
   partagé. Arrêtez l’ancien tableau de bord, le suivi de l’index et les
   connexions aux agents avant de basculer. Après le changement, seul
   l’index de remplacement doit publier.
3. Choisissez le chemin définitif. Vous pouvez utiliser la base là où vous
   l’avez importée. Pour reprendre le chemin d’origine, mettez d’abord en
   lieu sûr l’ancienne base, son dossier `.attachments` et les fichiers
   `-wal`, `-shm` ou `-journal` présents à côté. Déplacez ensemble la base
   importée et les fichiers qui l’accompagnent. Renommez
   `staged.sqlite3.attachments` et les fichiers auxiliaires SQLite pour
   qu’ils correspondent au nom de la nouvelle base.
4. Construisez les données de recherche à l’emplacement définitif et
   vérifiez qu’une recherche retrouve vos conversations. Indiquez le
   chemin choisi à la place de celui de l’exemple.

   ```sh
   mash --db "/absolute/path/to/replacement.sqlite3" index --rebuild-local-views --full
   mash --db "/absolute/path/to/replacement.sqlite3" status
   mash --db "/absolute/path/to/replacement.sqlite3" search 'a word you remember' --local --intent exact-mention
   ```

5. Si vous utilisez un chemin personnalisé, continuez à passer
   `--db`, y compris dans la configuration MCP de vos agents. Redémarrez
   vos agents et les processus de suivi de l’index.

<span id="other-changes-you-may-meet"></span>

## Autres changements possibles

<span id="coming-from-qdrant-119"></span>

### Passer de Qdrant 1.19 à 1.20

Mash exige Qdrant 1.20 ou une version plus récente. Avant de lancer
`mash index` avec le nouveau Mash, arrêtez le service 1.19 et lancez 1.20
avec le même stockage. La première indexation ajoute un index de recherche
par mots aux données existantes sans les reconstruire ni recalculer les
représentations sémantiques. Avant ce passage, la recherche signale que ses
données doivent être mises à niveau. Sur un serveur partagé, le nouveau
`mash-server` ajoute l’index à son démarrage.

**N’ouvrez pas avec Qdrant 1.19 des données déjà ouvertes avec 1.20.**
Faites d’abord une sauvegarde si vous risquez de revenir en arrière.

<span id="settings-that-are-no-longer-accepted"></span>

### Réglages abandonnés

- Retirez `views.query.experimental_gpu_reranking`. Le reclassement se fait
  sur le CPU et vous pouvez garder l’index et les modèles existants.
- Remplacez les anciens réglages des services distants de `[embeddings]`
  par ceux du [calcul des représentations (en anglais)](../configuration.md#embedding-inference).
- Retirez `redaction.remote_embeddings` des réglages de l’ordinateur et du serveur.

<span id="changing-the-embedding-model"></span>

### Changer de modèle de recherche par le sens

Les représentations calculées avec votre modèle local restent valables
après la mise à niveau. Si vous changez de modèle, lancez
`mash models install` et reconstruisez les données de recherche locales.

```sh
mash index --rebuild-local-views --full
```

Pour un index partagé, changez le modèle du serveur et de tous les
ordinateurs ensemble. Installez les modèles sur le serveur, puis
démarrez-le une fois avec cette commande.

```sh
mash-server --server-config PATH --models-dir PATH --activate-views --replay-views
```

Le serveur reconstruit les données de recherche à partir de l’historique
qu’il conserve. Faites d’abord une sauvegarde. La recherche partagée est
indisponible pendant la reconstruction.

<!-- mash-docs-source: f57dfe0421444d2aa8f7089a5af074b78ac73ec214071488c564847c13fb909c -->
