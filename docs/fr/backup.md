# Sauvegarder et restaurer votre historique

- **Garder une copie séparée que vous pourrez restaurer.** Lancez
  [`mash export`](#make-a-portable-backup) avant de mettre Mash à jour ou de le
  désinstaller, ou pour transférer votre historique sur un autre ordinateur.
- **Garder une transcription à lire ou à partager.** Utilisez
  [l’export du tableau de bord](dashboard.md#export-many-chats-at-once) pour
  obtenir des fichiers que vous pourrez ouvrir en dehors de Mash.
- **Réinstaller Mash plus tard sur le même ordinateur.** Utilisez
  [`mash uninstall --keep-history`](installation.md#uninstall) pour garder sur
  place la copie consultable de Mash et ses réglages. Faites une sauvegarde
  si vous avez besoin d’une copie séparée.

<span id="make-a-portable-backup"></span>

## Faire une sauvegarde portable

Choisissez un nouveau dossier dans un emplacement privé, séparé du
[dossier de données de Mash](privacy.md#where-mash-stores-data). Remplacez
les chemins d’exemple par des chemins absolus sur votre ordinateur. Gardez
les guillemets autour des chemins qui contiennent des espaces.

```sh
mash export "/absolute/path/to/new-backup"
```

Le résultat indique le nombre de conversations, de messages et de pièces
jointes conservées qui ont été copiés. Gardez tout le dossier de sauvegarde
ensemble. Il contient l’historique et les éléments de mémoire de l’index de
cet ordinateur. Pour un index partagé, lancez l’export sur chaque ordinateur
dont vous voulez sauvegarder l’historique.

**Gardez les sauvegardes privées.** Elles contiennent le texte enregistré
des conversations, y compris les identifiants secrets qui peuvent être
masqués dans les résultats de recherche. Gardez séparément les réglages,
les identifiants des services et les fichiers d’historique d’origine des
agents. Conservez la version de Mash qui a créé la sauvegarde si vous risquez
d’en avoir besoin pour une [mise à jour ou un retour en arrière (en anglais)](../upgrades.md#upgrade-an-existing-installation).

<span id="restore-without-replacing-your-current-history"></span>

## Restaurer une copie sans remplacer votre historique actuel

Choisissez un nouveau chemin de base de données. Mash refuse d’importer
dans une base existante. Importez la sauvegarde et listez quelques
conversations pour vérifier la copie.

```sh
mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/restored.sqlite3"
mash --db "/absolute/path/to/restored.sqlite3" list-chats --local --limit 5
```

L’export, l’import et la liste des conversations locales fonctionnent sans
le service de recherche. Si l’import signale une archive incompatible,
gardez la sauvegarde et utilisez la version de Mash qui l’a créée. Pour
remplacer un index et continuer à publier sous la même identité de machine,
suivez la [procédure de mise à jour (en anglais)](../upgrades.md#replace-an-index-that-cannot-be-upgraded).

<span id="make-the-restored-chats-searchable"></span>

## Rechercher dans les conversations restaurées

Démarrez le [service de recherche local](installation.md#start-the-local-search-service)
et préparez les données de recherche à partir de la copie restaurée.
Recherchez un mot de l’une des conversations de la liste.

```sh
mash --db "/absolute/path/to/restored.sqlite3" index --rebuild-local-views --full
mash --db "/absolute/path/to/restored.sqlite3" search 'a word you remember' --local --intent exact-mention
```

Passez `--db` chaque fois que vous voulez utiliser la copie restaurée. Si vos
réglages ou modèles se trouvent ailleurs, passez aussi `--config PATH` ou
`--models-dir PATH`.

<!-- mash-docs-source: ec7faa9045f83b2fb013c3e419ec659c83099c2c1a0269a9e2a6cf8b62a9fc66 -->
