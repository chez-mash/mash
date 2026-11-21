# Dépannage

| Ce que vous constatez | Par où commencer |
| --- | --- |
| Le terminal ne trouve pas `mash` ou PowerShell ne reconnaît pas la commande | [Retrouver la commande installée](#mash-command-not-found) |
| L’assistant n’arrive pas à joindre le service de recherche | [Démarrer Qdrant](#qdrant-connection-failed) |
| L’adresse du tableau de bord ne s’ouvre pas | [Vérifier son démarrage](#the-dashboard-doesnt-open) |
| Mash ne trouve aucun historique ou l’index n’est pas encore prêt | [Vérifier les historiques configurés](#nothing-is-indexed-yet) |
| Vous pouvez lire les conversations, mais une recherche ne donne rien | [Vérifier la requête, les filtres et la période](#search-returns-no-results) |
| Un agent de programmation ne trouve pas Mash | [Vérifier la connexion de l’agent](#my-agent-cannot-find-mash) |
| Après une mise à jour, Mash indique que le format de l’index est incompatible | [Récupérer ou mettre à jour l’index](#the-index-schema-is-older-or-newer-than-mash) |

Le problème persiste ou vous soupçonnez une faille de sécurité ? [Obtenir de l’aide](#get-help).

<span id="check-what-mash-sees"></span>

## Vérifier ce que Mash détecte

Si la cause n’est pas claire, lancez le diagnostic en lecture seule.

```sh
mash doctor
```

Dans `findings`, cherchez les entrées avec `"state": "problem"`. Le champ
`summary` décrit le problème. Lorsqu’il est présent, `repair` indique une
commande ou une étape à suivre. Si Mash trouve un problème, la commande
renvoie le code 1. Les entrées `warning` indiquent des points à vérifier.
`mash doctor` conserve vos réglages et votre historique.

Pour vérifier une partie précise de votre installation, utilisez la commande correspondante.

```sh
mash status                  # index and search readiness
mash status --installation   # history locations and agent connections
mash status --shared         # the index shared with your other machines
```

`mash status --installation` fonctionne même si l’index n’est pas prêt.
En cas d’erreur, cherchez ci-dessous son `error.kind` ou une expression
de son `error.message`.

<span id="mash-command-not-found"></span>

## Le terminal ne trouve pas la commande mash

Utilisez le chemin complet affiché par l’installateur ou ajoutez son dossier
à `PATH` dans le terminal déjà ouvert. Les commandes ci-dessous utilisent le
dossier par défaut de l’installateur.

```sh
export PATH="$HOME/.local/bin:$PATH"
mash --version
```

Sous Windows, lancez ces commandes dans votre fenêtre PowerShell.

```powershell
$env:Path = "$HOME\.local\bin;$env:Path"
mash --version
```

Si vous avez choisi un autre dossier, utilisez-le à la place de `.local/bin`.
Avec `cargo install`, le dossier habituel est `.cargo/bin` dans votre dossier
personnel. Dès que `mash --version` affiche une version, poursuivez dans ce
terminal. Pour conserver le réglage dans de nouveaux terminaux, ajoutez le
dossier au profil du shell ou à la variable utilisateur **Path** dans les
variables d’environnement de Windows.

<span id="the-dashboard-doesnt-open"></span>

## Le tableau de bord ne s’ouvre pas

Laissez `mash dash` tourner et ouvrez l’adresse affichée dans le navigateur
du même ordinateur. Si vous avez déjà lancé Mash dans un autre terminal,
ouvrez l’adresse affichée dans ce terminal. Si un autre programme occupe
l’adresse, laissez Mash choisir un port local libre.

```sh
mash dash --listen 127.0.0.1:0
```

Le `0` demande un port libre. Ouvrez l’adresse affichée par Mash. Si vous
avez besoin d’un port fixe, utilisez un port disponible comme `127.0.0.1:8791`.

Si la page indique que le tableau de bord n’a pas été compilé, installez une archive de version
ou [incluez le tableau de bord dans votre compilation](install-methods.md#include-the-web-dashboard).

<span id="qdrant-connection-failed"></span>

## La connexion à Qdrant a échoué

Les erreurs peuvent afficher `qdrant-unavailable`, `qdrant-install` ou
« Could not reach Qdrant at … ». Dans `mash status`, `local_qdrant` peut
indiquer `unreachable` avec la valeur `start-qdrant` dans `recommended_action`.

Si l’installation a échoué sous Linux ou WSL avec une erreur de `systemctl`,
[vérifiez votre session et choisissez comment démarrer Qdrant](local-search.md#linux-without-a-systemd-user-session).

Démarrez Qdrant de la façon dont vous l’avez installé.

- **`mash qdrant install`**. Relancez la commande. Elle redémarre un service
  arrêté et restaure les fichiers manquants. Dans `mash status`,
  `local_qdrant.installed` indique l’état et où lire les journaux, par
  exemple avec `journalctl --user --unit mash-qdrant.service` sous Linux.
- **Docker**. Démarrez le conteneur existant et lisez ses journaux.

  ```sh
  docker start mash-local-qdrant
  docker logs mash-local-qdrant
  ```

- **Votre propre Qdrant**. Relancez-le depuis son dossier d’origine et
  lisez ce qu’il affiche dans le terminal.

Si vous n’avez jamais installé Qdrant, suivez
[les étapes d’installation](installation.md#start-the-local-search-service).
Le service est prêt lorsque [la page de vérification](http://127.0.0.1:6333/readyz)
affiche « all shards are ready ».

Si `local_qdrant` indique `rejected`, un service a répondu mais a refusé la
requête ou annonce une version incompatible. Le statut indique la raison.
Les ports HTTP et gRPC doivent correspondre à la même instance de Qdrant
1.20 ou ultérieure. Voyez [comment changer les adresses (en anglais)](../configuration.md#local-qdrant).

`qdrant-failure` signifie que Qdrant a répondu avec une erreur. Lisez ses
journaux pour repérer un problème de stockage, d’identifiants ou de format
de données.

Lorsque Qdrant est arrêté, vous pouvez toujours lister les conversations,
lire les messages et exporter une sauvegarde. Si Mash reste à configurer,
choisissez **Save configuration only** ou lancez `mash init --no-index`.
Lancez ensuite `mash index` pour conserver vos conversations sur
l’ordinateur. La commande s’arrête avec `qdrant-unavailable` et donne
le nombre de conversations enregistrées. Lisez-les avec `mash list-chats`
et `mash view`. Une fois Qdrant en marche, relancez `mash index` pour mettre
la recherche à jour.

Sur une installation native de Windows, un journal Qdrant qui mentionne
`gridstore`, « path specified » et `os error 3` peut signaler un chemin de
stockage trop long. Utilisez les chemins absolus courts du
[guide Windows](local-search.md#native-windows-without-docker).
Sur une installation existante, arrêtez Qdrant et sauvegardez ses données
avant de les déplacer. Conservez les données d’origine. Si vous changez
uniquement le chemin dans les réglages, Qdrant ouvrira un nouveau dossier
vide et les données resteront dans l’ancien dossier.

<span id="nothing-is-indexed-yet"></span>

## Rien n’est encore indexé

Repérez l’erreur `database-missing`.

| Message | Commande |
| --- | --- |
| “Mash is not set up on this machine” | `mash init` |
| “Mash is configured, but nothing is indexed yet” | `mash index` |

Si Mash affiche « database does not exist », lancez `mash init` pour une
première configuration ou `mash index` si vos réglages sont déjà enregistrés.
Les deux ont besoin de Qdrant pour préparer la recherche.

Si Mash ne trouve aucune conversation, enregistrez une conversation dans
un agent pris en charge sur cet ordinateur, puis relancez `mash init`.
Si vous avez déjà des conversations, `mash status --installation` montre
les dossiers d’historique détectés. Revoyez les agents et dossiers
sélectionnés dans [la configuration](installation.md#connect-your-histories).
Pour des conversations plus anciennes, [vérifiez la période couverte (en anglais)](../indexing.md#add-older-history).

<span id="my-agent-cannot-find-mash"></span>

## Mon agent ne trouve pas Mash

Vérifiez l’agent avec lequel vous voulez utiliser Mash. Pour Claude Code,
lancez cette commande.

```sh
mash status --installation --agent claude-code
```

Utilisez `--agent codex` pour Codex. Pour Copilot CLI, consultez [les vérifications adaptées à votre version de Mash](agents.md#confirm-the-connection). Suivez
l’étape dans `next_action` si le bilan propose une solution. Vous pouvez
chercher dans le tableau de bord pendant que vous
[configurez la connexion](agents.md#connect-claude-code-codex-or-copilot).

Redémarrez l’agent après la configuration, une réparation ou la mise à
jour de Mash ou du plugin. Une conversation ouverte garde son ancienne
connexion. Pour tester une nouvelle connexion, lancez cette commande.

```sh
mash status --installation --agent claude-code --probe
```

Un test réussi confirme qu’un nouveau processus MCP a pu se connecter.
[Configurer un agent](agents.md)

<span id="search-returns-no-results"></span>

## La recherche ne donne aucun résultat

1. Vérifiez le [mode de recherche](dashboard.md#choose-a-search-mode).
   En recherche par mots, tous les mots saisis doivent apparaître dans le
   même message. Essayez un seul mot distinctif. Pour chercher par le sens,
   essayez une courte question en anglais. Si la langue n’utilise pas
   d’espaces entre les mots, voyez [la recherche dans d’autres langues](search-languages.md).
2. Si le tableau de bord affiche des filtres, choisissez **Clear filters**
   pour relancer la requête. Si vous revenez à la liste, ressaisissez la
   requête. Dans le terminal, relancez la recherche sans les options de filtre.
3. Vérifiez si la conversation est dans la liste. Choisissez **Browse chats**
   pour ouvrir la liste avec les filtres actuels. Si vous connaissez une
   partie du titre, saisissez-la dans la zone de recherche et regardez les
   suggestions sous **chats**. Dans le terminal, utilisez la commande ci-dessous.
4. Si une conversation récente manque ou est tronquée, lancez `mash index`
   et cherchez de nouveau. Pour une conversation ancienne,
   [élargissez la période indexée (en anglais)](../indexing.md#add-older-history).
   Si les conversations d’un agent manquent,
   [vérifiez ses dossiers d’historique](#nothing-is-indexed-yet).

La recherche couvre le dialogue entre vous et votre agent. Pour lire le
raisonnement ou la sortie d’un outil, cherchez un message proche et ouvrez
la conversation.

Pour trouver un titre dans le terminal, adaptez la commande à la partie
dont vous vous souvenez.

```sh
mash list-chats --title-contains 'part of the title'
```

Pour lire depuis le premier message, utilisez `mash view` et la valeur
`lookup.view.id` de la ligne. Vous pouvez aussi chercher dans la
conversation avec son identifiant.

<span id="models-are-missing"></span>

## Il manque les modèles

Repérez « semantic models are not installed » (`model`) ou
« related-discussion needs semantic search, which this configuration turns off »
(`usage`).

La recherche par le sens est facultative et nécessite des modèles.
Vous pouvez chercher des mots ou expressions avec cette commande.

```sh
mash search 'auth_token' --intent exact-mention
```

Pour chercher par le sens, [installez les modèles (en anglais)](../indexing.md#install-the-models)
et [préparez les données de recherche (en anglais)](../indexing.md#build-search-data).
L’installation des modèles conserve vos réglages. Le guide indique comment
activer l’indexation par le sens dans votre configuration existante.

<span id="embedding-failed-during-indexing"></span>

## Le calcul des vecteurs a échoué pendant l’indexation

Si le résultat contient `embedding_error`, il faut relancer `mash index`.
La commande a enregistré les messages, mais le calcul des vecteurs a
échoué, par exemple parce que les modèles manquent ou qu’un service distant
n’a pas répondu. Les nouveaux messages sont déjà accessibles par recherche
de mots (`exact-mention`). Pour les chercher par le sens
(`related-discussion`), corrigez le problème indiqué, puis relancez
`mash index`.

<span id="new-chats-are-missing-or-search-is-behind"></span>

## Des conversations récentes manquent ou la recherche est en retard

Mash lit les nouvelles conversations lors de l’indexation. Si des
conversations récentes manquent ou si `search-not-ready` apparaît, vérifiez
le message.

- « search is catching up with new history » indique qu’une indexation
  est en cours. Réessayez dans un instant.
- « search is behind the stored history and nothing is refreshing it »
  indique qu’il faut lancer la commande ci-dessous.

```sh
mash index
```

Le plugin lance une mise à jour à l’ouverture d’une session, au maximum
une fois toutes les dix minutes. Pour des mises à jour continues, voyez
[comment maintenir l’historique à jour (en anglais)](../synchronization.md).
Si les conversations d’un agent n’ont jamais figuré dans Mash,
[vérifiez les historiques locaux pris en charge et leur sélection](supported-agents.md#if-a-chat-is-missing).

<span id="a-chat-still-shows-what-an-older-release-read"></span>

## Une conversation affiche encore ce qu’une ancienne version a lu

Mash passe les conversations dont les fichiers n’ont pas changé depuis
leur dernière lecture. Si une mise à jour modifie la lecture de
l’historique, par exemple le dossier auquel une conversation est
rattachée, les conversations déjà indexées gardent l’ancien résultat
jusqu’au prochain changement. Pour tout relire tout de suite, lancez ceci.

```sh
mash index --reread
```

Mash conserve les données de recherche des conversations identiques,
ce qui est plus rapide qu’une première indexation. Pour limiter
l’opération à un agent, ajoutez par exemple `--provider codex`.

<span id="search-views-are-missing-or-stale"></span>

## Les données de recherche manquent ou sont anciennes

Si la base est compatible et contient déjà l’historique voulu,
reconstruisez les données de recherche locale à partir de celui-ci.

```sh
mash index --rebuild-local-views --full
```

Cette commande utilise l’historique déjà enregistré par Mash. Pour lire
de nouveaux fichiers ou réessayer une source en échec, lancez `mash index`
et vérifiez le résultat. Si la base est incompatible, suivez les étapes
de mise à jour ci-dessous.

<span id="the-index-schema-is-older-or-newer-than-mash"></span>

## Le format de l’index est plus ancien ou plus récent que Mash

Si `schema-incompatible` apparaît, arrêtez-vous et suivez la procédure
de [mise à jour et de sauvegarde (en anglais)](../upgrades.md). Gardez
l’ancien exécutable et les données. Exportez avec la version qui a écrit
l’index et validez l’import avec la nouvelle version dans un autre dossier.

Conservez la base et son numéro de format. `index --full` sert à
indexer. Pour changer le format, suivez la procédure de mise à jour.
Si une mise à jour est marquée en cours, terminez-la ou récupérez-la avant
de créer un index.

<span id="indexing-is-busy-incomplete-or-shared-pending"></span>

## L’indexation est occupée, incomplète ou en attente de partage

- **Occupée** (`index-busy`, « another Mash operation is updating this
  local index »). Une autre opération modifie l’index. Attendez qu’elle
  termine ou arrêtez votre processus `mash index --watch` avant de
  réessayer. Conservez les fichiers de verrouillage.
- **Incomplète**. Lisez l’erreur de la source, corrigez-la et relancez
  `mash index`. Mash garde l’historique déjà enregistré.
- **Partage en attente**. L’indexation locale peut avoir réussi.
  Vérifiez la connexion, le réglage du jeton et `mash status --shared`,
  puis relancez `mash index`.

Une recherche sur plusieurs ordinateurs peut fournir des résultats
locaux avec un `shared_error`. Utilisez `--local` pour chercher uniquement
sur cet ordinateur ou réparez la connexion partagée avant de vous fier
aux résultats de tous les ordinateurs.

<span id="get-help"></span>

## Obtenir de l’aide

[Signalez le problème sur GitHub](https://github.com/mashium/mash/issues/new?template=bug_report.yml)
si vous rencontrez une difficulté à l’installation, à la configuration ou à
l’utilisation. Indiquez votre système d’exploitation, la méthode d’installation,
l’étape qui a échoué et le résultat attendu. Si `mash --version` fonctionne, ajoutez
son résultat. Si l’installation a échoué, le message d’erreur suffit pour ouvrir
un ticket.

Les tickets GitHub sont publics. Partagez seulement l’erreur concernée ou un
court extrait de [`mash doctor`](#check-what-mash-sees). Retirez les chemins et
adresses privés, les conversations et les identifiants secrets. Si un exemple
est nécessaire, utilisez une conversation fictive.

Si vous pensez que Mash a exposé un historique privé, permis de contourner
l’authentification ou publié un paquet dont la somme de contrôle est incorrecte,
utilisez le [formulaire privé de GitHub](https://github.com/mashium/mash/security/advisories/new).
Consultez les [informations à joindre à un signalement de sécurité](../../SECURITY.md#report-a-vulnerability) (en anglais).

<!-- mash-docs-source: fe8614403eeed65a2fb4a5aaab7fbd91ec486017544e7a9dc1c6e36383dacbe9 -->
