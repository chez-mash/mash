# Utiliser Mash avec votre agent

Connectez Claude Code, Codex ou GitHub Copilot CLI pour lui demander de
retrouver des travaux antérieurs. Un agent connecté peut parcourir tout
l’historique que vous avez indexé. Les résultats qu’il lit entrent dans
son contexte. [Agents compatibles](supported-agents.md) · [Confidentialité de l’historique](privacy.md)

<span id="connect-claude-code-codex-or-copilot"></span>

## Connecter un agent après la configuration

Pour une première installation, suivez le [guide de démarrage rapide](quick-start.md)
et choisissez les agents à connecter lorsqu’on vous le demande. Si vos
conversations sont déjà accessibles dans Mash, ouvrez un terminal et
lancez cette commande pour connecter un agent sans relancer l’indexation.

```bash
mash init --no-index
```

1. Choisissez **Use existing settings** pour conserver votre sélection d’historiques.
2. À la question **Which agents should get access to Mash search?**, utilisez
   la barre d’espace pour ne laisser cochés que les agents à connecter.
   Si l’assistant propose de créer des souvenirs à partir d’anciennes
   conversations, vous pouvez refuser.
3. Choisissez **Set up without indexing**, puis redémarrez l’agent pour
   qu’il charge la connexion. [Essayez une recherche](#try-it).

Si votre agent ne figure pas dans la liste, vérifiez que sa commande
fonctionne dans ce terminal, par exemple avec `codex --version`. Installez
son interface en ligne de commande ou ajoutez-la à votre `PATH`, puis
relancez la commande ci-dessus.

La configuration guidée de GitHub Copilot CLI est disponible sur la branche
de développement. Si votre version propose uniquement Claude Code et Codex,
vous pouvez [configurer MCP manuellement](../mcp.md#connect-another-mcp-client).

Si vous avez déjà installé le module depuis le catalogue de l’agent,
utilisez sa [compétence `setup`](../plugin.md#start-with-the-plugin)
pour terminer la configuration.

### Ce que la connexion installe

Mash utilise la commande de chaque agent choisi (`claude`, `codex` ou
`copilot`) pour installer deux composants.

- Un **serveur MCP** met les outils de recherche de Mash à la disposition de
  l’agent. MCP (Model Context Protocol) permet aux agents d’utiliser des
  outils externes.
- Le **[module Mash](../plugin.md)** fournit la compétence `mash-history`
  qui indique quand et comment chercher, la compétence `remember` qui
  [mémorise](../records.md) une information sur demande (`/mash:remember` dans
  Claude Code) et des actions automatiques qui
  [tiennent l’historique à jour](#keep-new-chats-searchable).

Ces commandes modifient les paramètres utilisateur de l’agent
(`~/.claude.json`, `~/.claude/settings.json` et `~/.claude/plugins` pour
Claude Code, `~/.codex/config.toml` et `~/.codex/plugins` pour Codex,
`~/.copilot/mcp-config.json`, `~/.copilot/config.json` et
`~/.copilot/installed-plugins` pour GitHub Copilot CLI). Mash enregistre
aussi les agents connectés dans son propre `config.toml` pour que
[`mash status --installation`](#confirm-the-connection) puisse signaler
une connexion qui ne fonctionne plus.

L’installation du module nécessite un accès à Internet. Ses actions
automatiques nécessitent Python 3.9 ou une version plus récente
(`python3`, `python` ou le lanceur `py` sous Windows).

Si l’installation du module échoue, l’agent peut quand même chercher à
votre demande via le serveur MCP. Les compétences et actions automatiques
du module restent indisponibles.

Relancez `mash init` si la connexion pointe vers un exécutable Mash
introuvable ou vers un autre exécutable.

<span id="try-it"></span>

## Demander à l’agent de retrouver un travail antérieur

- « Utilise Mash pour retrouver ce que nous avons décidé au sujet de l’authentification. »
- « Retrouve la conversation où nous avons parlé du déploiement de ce dépôt. »
- « Sur quoi avons-nous travaillé la semaine dernière ? Montre-moi les messages qui le confirment. »
- « Retrouve notre précédente conversation sur cette erreur. »

L’agent cherche, lit les messages trouvés et cite le titre, la date ou
un lien vers la conversation. Demandez-lui les identifiants de session
et de message si vous en avez besoin.

Pour invoquer la compétence directement, utilisez `/mash:mash-history` dans
Claude Code. Dans Codex ou Copilot, choisissez la compétence `mash-history`
du module ou demandez à l’agent d’utiliser Mash.

<span id="confirm-the-connection"></span>

## Vérifier la connexion de l’agent

```bash
mash status --installation
```

Dans le rapport JSON, `summary.clients` indique l’état `connection` de
chaque agent.

| `connection` | Signification |
| --- | --- |
| `full` | La connexion fonctionne. |
| `mcp-only` | Les outils Mash sont connectés. Vérifiez le module, la compétence ou les actions automatiques. |
| `none` | Mash n’a aucune connexion utilisable à cet agent. Il a peut-être été ignoré lors de la configuration. |

En cas de problème, `next_action` indique comment procéder. Une nouvelle
indexation ne répare pas la connexion. Si le module du catalogue est plus
récent que votre CLI, le rapport vérifie si les deux déclarent la même
interface d’outils. Vous pouvez garder des outils compatibles si les fichiers
du module sont intacts. Une interface différente exige une version
correspondante de la CLI.

Un nombre `unconfigured` accompagné de `next_action: null` signifie que
Mash a trouvé des données d’agents sans connexion et ne propose aucune
réparation. Vous pouvez toujours parcourir l’historique indexé dans le
tableau de bord ou le terminal.

La commande affiche le rapport et renvoie le code 1 si un agent à vérifier
a été connecté par `mash init`, configuré pour Mash ou désigné avec
`--agent`. Vous pouvez laisser les autres agents déconnectés.

Pour vérifier un agent et démarrer son serveur MCP dans un nouveau
processus, choisissez la commande correspondante.

```bash
mash status --installation --agent codex --probe
mash status --installation --agent claude-code --probe
mash status --installation --agent copilot --probe
```

Ce test vérifie une nouvelle connexion. Redémarrez l’agent après une
configuration, une réparation ou une mise à jour pour que la connexion
soit prise en compte dans vos conversations.

<span id="keep-new-chats-searchable"></span>

## Actualiser votre historique

Le module indexe les nouveautés en arrière-plan à l’ouverture d’une
session, au plus une fois toutes les dix minutes. Pour désactiver cette
mise à jour, définissez `MASH_AUTO_INDEX=off` dans l’environnement de l’agent.

Lancez `mash index` pour actualiser l’historique maintenant, ou laissez
`mash index --watch` tourner dans un autre terminal pour une actualisation
continue. [Maintenir l’historique à jour](../synchronization.md)

<span id="disconnect-an-agent"></span>

## Déconnecter un agent

Utilisez les commandes de l’agent. `mash init` les affiche dans le champ
`undo`.

```bash
claude mcp remove --scope user mash
claude plugin uninstall mash@mash
codex mcp remove mash
codex plugin remove mash@mash
copilot mcp remove mash
copilot plugin uninstall mash@mash
```

Si vous voulez aussi retirer Mash de la machine, consultez
[la désinstallation](installation.md#uninstall). Si vous gardez Mash,
retirez l’agent de [`[agents] connected`](../configuration.md#connected-agents)
pour que Mash cesse de vérifier cette connexion.

## Autres clients MCP et configuration manuelle

Tout client qui accepte un serveur MCP sur l’entrée et la sortie standard
peut lancer `mash mcp`. Suivez
[l’exemple de configuration MCP](../mcp.md#connect-another-mcp-client) et
ajoutez les variables d’environnement dont votre configuration a besoin.

[Résoudre les problèmes de connexion](troubleshooting.md#my-agent-cannot-find-mash)

<!-- mash-docs-source: d890419b26152a4e78be6e843a0c4789881de03d39205a6b214101d802a2e9a2 -->
