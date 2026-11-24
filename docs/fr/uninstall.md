# Désinstaller Mash

Mash conserve les conversations d’origine de vos agents. La désinstallation
peut supprimer la copie que Mash a créée pour la recherche, avec les
mémoires enregistrées.

| Ce que vous voulez faire | Par où commencer |
| --- | --- |
| Retirer Mash et sa copie locale de vos conversations | [Afficher l’aperçu de la désinstallation](#remove-mash-and-its-local-copy) |
| Réinstaller Mash sur le même ordinateur avec vos conversations indexées et vos mémoires | [Garder la copie de Mash en place](#keep-history-for-a-reinstall) |
| Garder une copie distincte à restaurer sur cet ordinateur ou un autre | [Créer d’abord une sauvegarde portable](backup.md#make-a-portable-backup) |

<span id="remove-mash-and-its-local-copy"></span>

## Retirer Mash et sa copie locale

Commencez par afficher l’aperçu. Cette commande indique ce qui serait
supprimé sans rien modifier.

```sh
mash uninstall
```

Vérifiez ce que Mash retirera et ce qu’il conservera. L’aperçu indique
l’espace occupé par la copie de vos conversations. Si le bilan s’affiche
en JSON, lisez `paths`, `kept` et `next`.

Lorsque vous êtes prêt, fermez les tableaux de bord ou les processus de
suivi de l’index que vous avez lancés dans un terminal, puis confirmez.

```sh
mash uninstall --yes
```

Mash efface son index, ses réglages, les jetons enregistrés et les modèles
téléchargés. Il arrête et retire les services qu’il a configurés, dont son
service de recherche et les données de ce service. Il retire aussi ses
outils de recherche et son module des [agents connectés](agents.md#disconnect-an-agent)
via les commandes de ces agents.

Redémarrez les agents connectés encore ouverts. Si une étape a échoué,
corrigez le problème indiqué et relancez la commande avec les mêmes options.
Lorsque la désinstallation réussit, suivez la dernière instruction pour
retirer le programme lui-même. En JSON, lisez `binary.remove`. Selon
l’installation, il peut s’agir de `brew uninstall mash`,
`scoop uninstall mash`, `cargo uninstall mash`
ou de fichiers à supprimer.

Si vous avez installé le paquet `mash-cli` avec apt, exécutez `sudo apt remove mash-cli`.
Le rapport de désinstallation de Mash `v0.1.1-alpha` peut indiquer un nom de paquet incorrect.

## Copies conservées

- Les conversations d’origine de vos agents et les fichiers de `~/.mash`
  qui n’appartiennent pas à Mash restent sur votre ordinateur.
- Si vous avez publié des conversations sur un serveur partagé, la copie y
  reste. Pour la supprimer et révoquer l’accès de cet ordinateur,
  [retirez l’ordinateur de l’index partagé](shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again).
- Si vous gérez Qdrant vous-même, gérez ce service et ses données de
  recherche Mash séparément.

Pour retirer uniquement le service de recherche installé par Mash, utilisez
`mash qdrant uninstall`. La commande garde les données de recherche sauf
si vous ajoutez `--delete-storage`. Vous pouvez les reconstruire à partir
de l’index local avec `mash index --rebuild-local-views --full`.

<span id="keep-history-for-a-reinstall"></span>

## Garder l’historique pour une réinstallation

Utilisez `--keep-history` si vous comptez revenir sur le même ordinateur
avec les mêmes conversations indexées et mémoires. Mash garde son index,
ses réglages, les jetons enregistrés et les données du service de recherche
qu’il gère. Il retire les services, les connexions aux agents et les modèles
téléchargés.

Utilisez la même option pour afficher l’aperçu et pour confirmer.

```sh
mash uninstall --keep-history
```

Dans la version `v0.1.1-alpha`, l’aperçu peut omettre `--keep-history` dans
la commande de confirmation proposée. Utilisez la commande complète ci-dessous.
Une confirmation avec `--yes` seul efface les conversations indexées et les
souvenirs enregistrés dans Mash.

```sh
mash uninstall --yes --keep-history
```

Avant de retirer le programme, exécutez `mash --version` et notez le numéro
de version pour la réinstallation.
Suivez [la procédure de désinstallation](#remove-mash-and-its-local-copy)
pour reprendre les étapes en échec, redémarrer les agents et retirer le programme.

<span id="reopen-the-retained-history"></span>

### Rouvrir l’historique conservé

1. Sur le même ordinateur et avec le même compte utilisateur, installez la
   version de Mash que vous avez notée. Donnez à `MASH_VERSION` la valeur de
   son tag, par exemple `v0.1.1-alpha`, avant de lancer
   [l’installateur de base](installation.md#recommended-release-installers).
   Pour installer depuis une archive, ouvrez ce tag dans les
   [versions publiées sur GitHub](https://github.com/grep-stack/mash/releases).
   Téléchargez l’archive de votre système et le fichier `SHA256SUMS` de cette
   même version. [Vérifiez l’archive](downloads.md#verify-the-checksum) avant
   de l’extraire. Pour installer une version plus récente, consultez le
   [guide de mise à niveau](upgrades.md#upgrade-an-existing-installation).
2. Si Mash avait installé le service de recherche, lancez `mash qdrant install`
   pour le réinstaller et réutiliser ses données. Si vous [gériez Qdrant vous-même](local-search.md#run-qdrant-yourself),
   redémarrez-le de la même façon qu’auparavant.
3. Lancez `mash dash` et ouvrez l’adresse affichée, ou utilisez
   `mash list-chats --local --limit 5` pour vérifier les conversations
   enregistrées dans le terminal. Mash réutilise les réglages et l’index
   conservés. Si vous utilisiez des chemins personnalisés, indiquez-les à nouveau.

Si vous devez actualiser la recherche, voyez [comment garder l’historique à jour](synchronization.md).
Si vous cherchiez par le sens, [réinstallez les modèles](indexing.md#install-the-models).
Vous pouvez aussi [reconnecter un agent](agents.md#connect-claude-code-codex-or-copilot).

<!-- mash-docs-source: 78c676e9d7c8bc968b81510a879cb7d593ba3e7d150d2620a927882b98f7d57e -->
