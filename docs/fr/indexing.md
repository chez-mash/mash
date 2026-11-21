# Inclure les conversations plus anciennes et chercher par le sens

<span id="add-older-history"></span>

## Inclure un historique plus ancien

Lors d’une première installation, Mash lit l’historique que vos agents ont
mis à jour au cours des 90 derniers jours. Une conversation mise à jour
récemment peut contenir des messages plus anciens. Pour inclure les
conversations plus anciennes qui se trouvent encore sur le disque :

```sh
mash index --all-history
```

Pour remonter d’un nombre précis de jours :

```sh
mash index --since-days 365
```

Pour mettre à jour l’historique d’un seul agent :

```sh
mash index --provider codex
```

Les noms des fournisseurs sont `claude-code`, `codex`, `opencode`, `copilot`,
`cursor` et `antigravity`.

Le service de recherche local (Qdrant) doit être en cours d’exécution pour
indexer. Mash lit les fichiers d’origine sans les modifier. Vérifiez si le
résultat final de la commande signale une erreur.

Ces options s’appliquent à une seule exécution. Pour modifier définitivement
la période, voyez la [configuration (en anglais)](../configuration.md).
Pour des mises à jour continues, voyez [comment garder l’historique à jour (en anglais)](../synchronization.md).

<span id="copilot-in-vs-code-remote-windows"></span>

### Copilot dans les fenêtres distantes de VS Code

Indexez chaque type de session sur la machine où elle est enregistrée :

- Les sessions **Copilot Chat** ouvertes dans une fenêtre Remote-SSH, un
  conteneur ou Codespaces sont enregistrées sur l’ordinateur qui exécute VS
  Code. Leur projet correspond au dossier sur la machine distante.
- Les sessions **Copilot CLI** lancées depuis le chat d’une fenêtre distante
  s’exécutent sur la machine distante et sont enregistrées dans son `~/.copilot/session-state`.

Si Copilot écrit encore dans une session pendant que Mash la lit, Mash
conserve les messages déjà indexés et relit la session à la prochaine mise à jour.

<span id="enable-semantic-search"></span>

## Activer la recherche par le sens

La recherche par le sens, ou recherche sémantique, permet de poser une
question quand on ne se souvient pas des termes exacts. Posez ces questions
en anglais. La recherche de mots ou d’expressions fonctionne sans modèle
et reste le mode par défaut jusqu’à ce que vous modifiiez ce réglage. Pour
une conversation dans une autre langue, voyez les
[exemples de recherche par mots](search-languages.md).

<span id="install-the-models"></span>

### Installer les modèles

```sh
mash models install
```

La commande télécharge environ 177 Mo depuis `https://huggingface.co` dans
`~/.mash/models`. Sur Apple silicon et Windows, elle télécharge aussi le
modèle d’embeddings non quantifié pour le GPU, soit environ 90 Mo de plus.
Réglez `HF_ENDPOINT` pour utiliser un miroir de Hugging Face, ou passez
`--models-dir PATH` pour choisir un autre dossier.

<span id="build-search-data"></span>

### Préparer l’index pour la recherche par le sens

Si vous configurez Mash pour la première fois, lancez `mash init` après
l’installation des modèles. Mash active l’indexation par le sens à la
création des réglages et prépare les données pendant la première indexation.
[Première configuration](installation.md#connect-your-histories)

Si vous utilisez déjà Mash, modifiez la table `[views.build]` dans votre
[configuration (en anglais)](../configuration.md). Ouvrez **settings** dans
le tableau de bord ou cherchez `configuration.path` dans le résultat de
`mash status` pour trouver le fichier. Gardez vos autres réglages et ajoutez
la table seulement si elle est absente.

```toml
[views.build]
representations = "lexical-and-dense"
```

Lancez ensuite `mash index`. La première indexation prend plus de temps, car
Mash prépare votre historique pour la recherche par le sens. Lors des
suivantes, Mash réutilise le travail déjà fait pour les conversations inchangées.

Vérifiez ensuite `mash status`. Le champ `realized_mode` affiche `hybrid`
lorsque la recherche par mots et par le sens est disponible sur cet ordinateur.

<span id="try-a-question"></span>

### Essayer une question

```sh
mash search 'why did we choose a separate database' --intent related-discussion
```

<span id="use-search-by-meaning-by-default"></span>

### Chercher par le sens par défaut

Pour utiliser ce mode dans le tableau de bord et pour les recherches qui
n’indiquent aucun mode, définissez `intent` dans la table `[views.query]`.
Ajoutez la table si elle est absente. Une fois l’index prêt, ce réglage
s’applique sans réindexer l’historique.

```toml
[views.query]
intent = "related-discussion"
```

Pour faire une recherche par mots même avec ce réglage, passez
`--intent exact-mention`.

```sh
mash search 'auth_token' --intent exact-mention
```

En cas de problème, voyez le [guide de dépannage](troubleshooting.md).

<!-- mash-docs-source: 6724bc90be8c2bf0b6959aaeb8d9bf61cc977de3fcd373c007efa07572341d58 -->
