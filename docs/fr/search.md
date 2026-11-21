# Rechercher des conversations avec Mash

Mash cherche dans les [conversations enregistrées par vos agents de programmation](supported-agents.md) sur cet ordinateur et sur ceux que vous reliez à un [index partagé](shared-index.md). La recherche porte sur vos messages et ceux de vos agents. Ouvrez la conversation pour lire les appels d’outils et leurs résultats.

Utilisez le [tableau de bord pour retrouver et lire une conversation](dashboard.md#find-and-read-a-chat) ou [demandez à un agent connecté](agents.md#try-it). Les exemples de ce guide utilisent le terminal.

<span id="your-first-search"></span>

## Votre première recherche

Après avoir [configuré la recherche locale](installation.md#connect-your-histories), essayez un mot distinctif, un nom de fichier ou un identifiant tiré d’une conversation. Cet exemple affiche jusqu’à trois résultats.

```bash
mash search 'authentication token' --limit 3
```

La sortie JSON regroupe les résultats par conversation et présente la meilleure correspondance en premier. Repérez le champ `title` de la conversation et le champ `snippet` du message correspondant. Copiez le champ `lookup.view.id` de ce message et utilisez-le à la place de `MESSAGE_ID` pour lire le message dans son contexte.

```bash
mash view MESSAGE_ID --context 3
```

<details>
<summary>Exemple de résultat</summary>

```json
{
  "results": [
    {
      "chat_id": "01a1a77a-8105-7cf1-849b-37524f39e223",
      "provider": "codex",
      "title": "Add token authentication to the dashboard",
      "project": "/home/you/project",
      "matches": [
        {
          "lookup": { "view": { "id": "709e029507a46e56c1e1cf40" } },
          "role": "assistant",
          "snippet": "It's an authentication gate: the dashboard asks for a token…"
        }
      ]
    }
  ],
  "next_offset": null
}
```

</details>

La recherche utilise l’index existant. Pour retrouver une conversation que vous venez d’avoir, lancez d’abord `mash index`.

<span id="read-the-original-messages"></span>

## Lire les messages d’origine

`MESSAGE_ID` correspond au champ `lookup.view.id` d’un résultat de recherche ou
de `list-chats`. L’option `--context 3` ajoute jusqu’à trois messages avant et
trois après. Si l’identifiant ne correspond à aucun message, Mash renvoie
l’erreur `not-found`. Si vous donnez l’identifiant d’une conversation, l’erreur
indique l’identifiant du premier message à utiliser.

Pour l’historique local, la sortie fournit le champ `chat_id` du message, le champ `message_count` de la conversation, les `messages` et le nombre de messages laissés de côté avant et après (`omitted_before`, `omitted_after`). Si des messages sont omis, le champ `notice` le signale. Augmentez `--context` pour en lire davantage.

Un résultat provenant d’un autre ordinateur contient aussi `lookup.view.shared`, l’identifiant de l’ordinateur qui a publié cet historique. Passez-le avec `--shared`.

```bash
mash view MESSAGE_ID --shared NODE_ID --context 3
```

<span id="search-in-other-languages"></span>

## Rechercher dans d’autres langues

Pour chercher par mots, reprenez l’orthographe de la conversation d’origine. Pour des exemples et des conseils en chinois ou en japonais, consultez la [recherche de conversations dans d’autres langues](search-languages.md). La recherche par sens utilise des modèles conçus pour l’anglais et fonctionne mieux sur des conversations dans cette langue.

<span id="find-a-chat-by-title"></span>

## Retrouver une conversation par son titre

```bash
mash list-chats --title-contains 'authentication' --limit 10
```

Cette commande liste les conversations sans chercher dans leurs messages. Elle fonctionne sans modèle et même si l’index de recherche est indisponible. Le titre reprend le premier message ou le nom de la session Claude Code si elle en a un. Un nouveau nom apparaît lors de la prochaine indexation de la session.

Chaque conversation contient le champ `lookup.view.id` de son premier message. Vous pouvez ainsi commencer la lecture depuis le début.

```bash
mash view FIRST_MESSAGE_ID --context 10
```

Pour chercher dans une conversation, passez son champ `chat_id` et demandez des résultats par message.

```bash
mash search 'token' --chat CHAT_ID --unit messages
```

<span id="narrow-the-results"></span>

## Affiner les résultats

| Je me souviens…                 | Exemple                                                                       |
| ------------------------------- | ----------------------------------------------------------------------------- |
| De l’agent                      | `mash search 'authentication' --provider codex`                               |
| Du moment approximatif          | `mash search 'authentication' --days 7`                                       |
| D’une période                   | `mash search 'authentication' --after 2026-10-01 --before 2026-11-01`         |
| Du dossier du projet            | `mash search 'authentication' --project /absolute/path/to/project`            |
| Du dépôt                        | `mash search 'authentication' --repository https://github.com/owner/repo.git` |
| De quelque chose que j’ai écrit | `mash search 'authentication' --role user`                                    |
| De l’orthographe exacte         | `mash search 'auth_token' --match-mode strict`                                |
| De l’expression exacte          | `mash search '"rotate the logs"'`                                             |

Les filtres se combinent et chaque résultat doit tous les satisfaire. Pour accepter plusieurs valeurs pour un filtre, répétez l’option, par exemple `--provider codex --provider claude-code`.

`--project` et `--repository` doivent correspondre exactement aux valeurs enregistrées. Copiez la valeur du champ `project` ou `repository` d’un résultat de `list-chats` ou de recherche.

`--after` et `--before` acceptent une date (à minuit UTC) ou un horodatage RFC 3339. `--after` inclut cette limite et `--before` l’exclut. Dans `list-chats`, ces options prennent des secondes Unix et filtrent selon l’activité de la conversation.

<span id="search-by-meaning"></span>

## Rechercher par sens

Par défaut, Mash cherche par mots sans modèle. Passez `--intent exact-mention` pour choisir ce mode si votre configuration définit un autre [mode par défaut (en anglais)](../configuration.md#search-defaults).

Après avoir [activé la recherche par sens](indexing.md#enable-semantic-search), décrivez ce dont vous vous souvenez avec vos propres mots.

```bash
mash search 'why did we choose a separate database' --intent related-discussion
```

Pour l’instant, formulez vos recherches par sens en anglais.

Choisissez le mode selon ce dont vous vous souvenez.

|                   | `exact-mention`                                                       | `related-discussion`                                      |
| ----------------- | --------------------------------------------------------------------- | --------------------------------------------------------- |
| Pour              | Des mots, des identifiants, des noms de fichiers ou du texte d’erreur | Une question ou une idée qui peut être formulée autrement |
| Besoin de modèles | Non                                                                   | Oui                                                       |
| Plusieurs mots    | Tous doivent figurer dans le message, dans n’importe quel ordre       | Ils peuvent être absents du message                       |

La recherche par mots cherche les termes saisis. Par exemple, « log rotation » ne trouvera pas « rotate the application logs ». Essayez « rotate » ou la recherche par sens. Un terme comme `auth_token` ou `voyage-code-4` est cherché tel qu’il est écrit, en un seul mot. Par défaut, les préfixes et les petites fautes de frappe sont acceptés. `--match-mode strict` exige l’orthographe exacte.

<span id="search-for-an-exact-phrase"></span>

## Rechercher une expression exacte

Placez l’expression entre guillemets doubles à l’intérieur des guillemets utilisés par le shell.

```bash
mash search '"rotate the logs"'
```

Une requête entre guillemets cherche l’expression complète avec cette orthographe. En l’absence de résultat, retirez les guillemets pour chercher les mots séparément.

<span id="more-results-and-local-only-searches"></span>

## Afficher plus de résultats ou chercher sur cet ordinateur

Chaque page contient 20 résultats, sauf si vous choisissez une autre limite avec `--limit`. Pour ouvrir la page suivante, répétez la commande avec `--offset` et la valeur renvoyée par `next_offset`. Une valeur nulle indique la dernière page.

`--sort recent` affiche les résultats les plus récents en premier sur chaque page de correspondances. Pour lister les conversations dans l’ordre chronologique, utilisez `list-chats`.

Avec un [index partagé](shared-index.md), Mash cherche dans l’historique local et dans celui publié par les autres ordinateurs. `--local` cherche sur cet ordinateur et `--shared` dans l’historique publié uniquement. `--node NODE_ID` filtre les résultats par ordinateur à l’origine de l’historique, sans s’y connecter.

En cas de résultats manquants ou obsolètes, consultez le [guide de dépannage](troubleshooting.md). Pour les autres opérations, consultez les [commandes utiles (en anglais)](../cli.md).

<!-- mash-docs-source: fe6be95f1cae06f55d2ce70951ed7c053b4a2ed4ad7f911f5d36d5fc9a56e42e -->
