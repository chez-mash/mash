# Le module Mash pour Claude Code, Codex et Copilot

Le module Mash permet à Claude Code, Codex ou GitHub Copilot CLI de retrouver des travaux antérieurs pendant vos échanges, d’enregistrer une décision à votre demande et d’ouvrir le tableau de bord Mash. Il ajoute des compétences et des scripts automatiques (hooks). Le [serveur MCP](../mcp.md) de Mash fournit à l’agent les outils de recherche utilisés par ces compétences. Vous pouvez aussi utiliser MCP seul et demander à l’agent de faire une recherche avec Mash.

* **Vous utilisez déjà le tableau de bord Mash ou le terminal ?** [Connectez votre agent](agents.md#connect-claude-code-codex-or-copilot)
  sans relancer l’indexation. La configuration installe le module pour les agents qu’elle connecte.
* **Vous avez déjà installé le module depuis le catalogue de votre agent ?** [Terminez la configuration dans l’agent](#finish-setup-in-the-agent).
* **Vous partez de zéro ?** Suivez le [guide de démarrage rapide](quick-start.md), ou [installez d’abord le module](#start-with-the-plugin) et laissez votre agent vous guider pendant la configuration.

<span id="install-update-and-remove"></span>

<span id="install-the-plugin"></span>

## Installer le module

<span id="requirements"></span>

L’installation nécessite un accès à Internet. Les hooks nécessitent Python 3.9 ou une version ultérieure (`python3`, `python` ou le lanceur `py` sous Windows).

<span id="start-with-the-plugin"></span>

### Commencer par le module

Exécutez les deux commandes correspondant à votre agent dans un terminal. Si vous avez déjà ajouté le catalogue Mash, exécutez la deuxième commande.

#### Claude Code

```sh
claude plugin marketplace add chez-mash/mash
claude plugin install mash@mash
```

#### Codex

```sh
codex plugin marketplace add chez-mash/mash
codex plugin add mash@mash
```

<span id="install-in-github-copilot-cli"></span>

#### Installer dans GitHub Copilot CLI

```sh
copilot plugin marketplace add chez-mash/mash
copilot plugin install mash@mash
```

<span id="finish-setup-in-the-agent"></span>

### Terminer la configuration dans l’agent

Démarrez une nouvelle session. Dans Claude Code, exécutez `/mash:setup`. Dans Codex ou Copilot, dites « Utilise la compétence `setup` de Mash pour configurer Mash sur cet ordinateur. » L’agent passe en revue avec vous ce qui sera installé et les conversations à indexer, puis vous montre comment ouvrir le tableau de bord. La connexion des agents est facultative. Si vous choisissez de connecter Copilot CLI et que vous utilisez Mash v0.1.1-alpha, [finalisez la connexion à Copilot](#github-copilot-cli)
après la configuration.

Si vous avez connecté un agent, redémarrez-le et demandez-lui « Utilise Mash pour retrouver une ancienne conversation sur l’authentification. Montre-moi le message sur lequel tu t’appuies. » Choisissez un sujet que vous avez déjà abordé. Pour activer les hooks de Codex, ouvrez `/hooks` dans Codex, examinez les hooks de Mash et indiquez que vous leur faites confiance. [Vérifiez la connexion](agents.md#confirm-the-connection)
si l’agent ne trouve pas Mash.

<span id="skills-and-commands"></span>

## Compétences et commandes

| Compétence | Dans Claude Code | Dans Codex | Fonction |
| --- | --- | --- | --- |
| `setup` | `/mash:setup` | sélectionnez la compétence `setup` ou demandez « Configure Mash sur cet ordinateur » | Installe Mash avec votre accord, connecte votre historique et vérifie que vous pouvez retrouver une ancienne conversation. |
| `remember` | `/mash:remember what to keep` | sélectionnez la compétence `remember` ou dites « Souviens-toi de ça » | Enregistre une décision, une préférence ou un fait dans la [mémoire](records.md), en s’appuyant sur vos propres mots. Sans précision, enregistre la dernière décision de la conversation. |
| `resume` | `/mash:resume` | sélectionnez la compétence `resume` ou demandez « Où en étions-nous ? » | Lit les conversations les plus récentes du projet, tous agents et ordinateurs confondus, et vous indique ce qui était en cours, ce qui restait à faire et ce qui a été décidé. |
| `handoff` | `/mash:handoff` | sélectionnez la compétence `handoff` ou dites « Fais un point pour la personne qui reprendra le travail » | Rédige dans la conversation un résumé de la session qui indique l’objectif, ce qui est terminé, ce qui est en cours, les décisions prises et la prochaine étape. `resume` retrouve ce résumé depuis n’importe quel agent ou ordinateur une fois la conversation indexée. |
| `recall` | `/mash:recall a topic` | sélectionnez la compétence `recall` ou demandez « Que retient Mash à propos de… ? » | Affiche les entrées de [mémoire](records.md) sur un sujet ou sur le projet, avec leur source. Vous indique quand des entrées doivent être vérifiées. |
| `review` | `/mash:review` | sélectionnez la compétence `review` ou dites « Passe en revue la mémoire de Mash » | Vous soumet quelques entrées de mémoire à la fois pour que vous décidiez quoi en faire. Il peut s’agir d’entrées enregistrées par un agent sans votre confirmation, d’entrées obsolètes ou de paires d’entrées contradictoires. Vous pouvez conserver, corriger, retirer, supprimer ou ignorer chaque entrée. |
| `forget` | `/mash:forget what to remove` | sélectionnez la compétence `forget` ou demandez explicitement d’oublier ou de supprimer un élément enregistré par Mash | Retrouve une entrée de mémoire ou une conversation indexée, vous montre exactement ce qui serait supprimé et attend votre confirmation avant de procéder. Si une entrée est erronée, propose de la corriger au lieu de la supprimer. |
| `standup` | `/mash:standup`, ou `/mash:standup this week` | sélectionnez la compétence `standup` ou demandez « Sur quoi ai-je travaillé hier ? » | Résume votre travail sur une période donnée, par projet, tous agents et ordinateurs confondus. Sans précision, couvre le jour ouvré précédent. |
| `export` | `/mash:export`, ou `/mash:export a chat` | sélectionnez la compétence `export` ou dites « Exporte cette conversation » | Ouvre le [tableau de bord](dashboard.md#save-a-transcript) sur la conversation, que vous pouvez enregistrer au format Markdown, HTML ou PDF depuis le menu Export. Peut aussi créer un fichier Markdown à votre demande. |
| `status` | `/mash:status` | sélectionnez la compétence `status` ou demandez « Est-ce que Mash fonctionne ? » | Vous indique en termes simples l’état de l’index, de la recherche, des connexions aux agents et de vos autres ordinateurs. En cas de problème, vous indique la prochaine action à effectuer. |
| `dashboard` | `/mash:dashboard` | sélectionnez la compétence `dashboard` ou dites « Ouvre le tableau de bord Mash » | Repère le [tableau de bord](dashboard.md) déjà lancé ou le démarre, puis l’ouvre dans votre navigateur. Dites « cette conversation » pour accéder directement à la conversation en cours. |
| `mash-history` | `/mash:mash-history`, ou formulez simplement votre demande | sélectionnez la compétence `mash-history` ou formulez simplement votre demande | Effectue une recherche dans vos anciennes conversations et cite dans sa réponse les messages sur lesquels elle s’appuie. L’agent utilise aussi cette compétence de lui-même lorsque vous faites référence à des travaux antérieurs. |

GitHub Copilot CLI charge les mêmes compétences. Indiquez le nom de la compétence ou formulez votre demande avec vos propres mots.

Au quotidien, formulez vos demandes avec vos propres mots.

* « Qu’a-t-on décidé à propos de la migration de la base de données ? »
* « Retrouve la conversation où nous avons configuré le déploiement. »
* « Souviens-toi que nous déployons depuis `main` uniquement. »
* « Ouvre le tableau de bord Mash. »
* « Où en étions-nous sur ce projet ? »
* « Que retient Mash à propos du déploiement ? »
* « L’entrée de mémoire sur l’URL de préproduction est erronée. Corrige-la. »

Après l’enregistrement, l’agent vous confirme ce qu’il a enregistré. Il refuse d’enregistrer les identifiants et les jetons. La suppression est définitive. `forget` traite une seule entrée ou conversation à la fois, vous la montre et demande toujours votre confirmation au préalable.

<span id="what-happens-without-asking"></span>

## Actions automatiques

| Quand | Ce que fait le module | Comment le désactiver |
| --- | --- | --- |
| Une session démarre | Montre à l’agent les entrées enregistrées et les conversations récentes du projet, puis lance en arrière-plan la commande `mash index` au maximum une fois toutes les dix minutes | `MASH_AUTO_INDEX=off` désactive l’indexation automatique du module. `MASH_RECORD_RECALL=off` masque les entrées enregistrées dans les rappels de session et de fichier. Les suggestions de conversations continuent de s’afficher. |
| Vous envoyez un message qui revient sur des travaux antérieurs ou mentionne un élément encore absent de la conversation | Rappelle à l’agent de faire une recherche dans Mash avant d’émettre une hypothèse | `MASH_PROMPT_RECALL=off` |
| Vous exprimez une décision ou une préférence durable | Rappelle à l’agent comment l’enregistrer. Si la session se prolonge sans aucun enregistrement, l’agent reçoit au maximum un rappel toutes les quinze minutes | `MASH_RECORD_CAPTURE=off` |
| L’agent lit un fichier du projet | Affiche les entrées et les conversations antérieures concernant ce fichier, une fois par fichier et par session | `MASH_FILE_RECALL=off` |

Dans Copilot CLI, la note sur un fichier accompagne le contenu que l’agent vient de lire et s’affiche juste après la lecture.

Définissez la variable dans l’environnement où l’agent démarre. Si Mash est absent, pas encore configuré ou lent à répondre, les hooks restent inactifs et silencieux, ce qui évite de bloquer l’agent. Les hooks appellent Mash sur votre ordinateur. Les extraits qu’ils affichent au démarrage d’une session ou à la lecture d’un fichier entrent dans le contexte de l’agent, qui peut transmettre ce contexte au fournisseur de son modèle. Les hooks activés peuvent fonctionner sans connexion MCP. Si vous utilisez uniquement le tableau de bord ou le terminal, [supprimez tout module Mash installé](#update-or-remove-the-plugin)
et [déconnectez l’agent](agents.md#disconnect-an-agent).
[Quand l’historique peut quitter votre ordinateur](privacy.md#when-history-can-leave-your-machine)

<span id="update-or-remove-the-plugin"></span>

## Mettre à jour ou supprimer le module

Le module et le programme `mash` sortent ensemble. Un catalogue peut proposer une version plus récente du module avant la publication du binaire correspondant. Exécutez
`mash status --installation` pour savoir quel composant doit être mis à jour. Un module plus récent peut fonctionner avec le binaire actuel si l’interface des outils est compatible et si les autres vérifications de l’installation réussissent. Démarrez une nouvelle session après la mise à jour. Pour supprimer le module, exécutez la commande correspondant à votre agent.

```sh
claude plugin uninstall mash@mash
codex plugin remove mash@mash
copilot plugin uninstall mash@mash
```

Pour supprimer également les outils de recherche, [déconnectez l’agent](agents.md#disconnect-an-agent).

## GitHub Copilot CLI

Les versions de développement peuvent connecter GitHub Copilot CLI depuis `mash init` si la commande
`copilot` figure dans votre `PATH`. La version v0.1.1-alpha peut lire les conversations enregistrées par Copilot, et vous pouvez donner manuellement à Copilot accès à la recherche dans Mash. Après avoir [configuré Mash](quick-start.md), enregistrez ses outils depuis un terminal.

```sh
copilot mcp add mash -- mash mcp
```

Si Copilot ne trouve pas `mash`, remplacez l’exécutable qui suit `--` par le chemin complet vers votre installation, entre guillemets. Vous pouvez aussi [installer le module Copilot](#install-in-github-copilot-cli)
pour accéder aux compétences et aux hooks. Ouvrez une nouvelle session Copilot et demandez-lui d’utiliser Mash pour retrouver une ancienne conversation. La commande `copilot mcp list` affiche `mash`, et la commande `copilot plugin list`
affiche `mash@mash` si vous avez installé le module.

Avec la version v0.1.1-alpha, déconnectez Copilot avant de désinstaller Mash. Si vous avez installé le module, exécutez aussi sa commande de désinstallation.

```sh
copilot plugin uninstall mash@mash
copilot mcp remove mash
```

* Copilot conserve ces éléments dans `~/.copilot` (ou dans le répertoire indiqué par `COPILOT_HOME`). Les outils figurent dans `mcp-config.json` et le module installé dans
  `config.json` et `installed-plugins`.
* `mash init --backfill-memory` ne prend pas en charge Copilot. L’examen des anciennes conversations de cette manière consommerait vos requêtes premium. Seuls Claude Code et Codex sont donc proposés pour cette opération.
* Pour enregistrer une information dans la [mémoire](records.md) depuis une session Copilot, il faut une version de `mash` capable de lire la session Copilot en cours. La commande `mash --version-info` affiche `copilot`
  sous `session_citations` si cette fonctionnalité est disponible. Avec une version plus ancienne de `mash`, le module retrouve les informations enregistrées, mais ne demande pas à Copilot de les enregistrer.
* Sous Windows, les hooks de Copilot s’exécutent via PowerShell et appellent `python`. Aucune vérification des hooks sous Windows n’a encore été documentée.

<span id="check-that-it-is-working"></span>

## Vérifier que tout fonctionne

```sh
mash status --installation
```

Pour chaque agent, le champ `connection` vaut `full` si les outils et le module sont tous deux en place, et `mcp-only` si seuls les outils sont connectés. Le champ `next_action` indique la prochaine étape lorsqu’une intégration nécessite une intervention. Pour Copilot avec la version v0.1.1-alpha, utilisez [les vérifications propres à Copilot](#github-copilot-cli).
[Utiliser Mash avec votre agent](agents.md#confirm-the-connection)

<!-- mash-docs-source: 1e02f9b9b161736cf7cca776c5a49c6df0689c3d5dab9868d0a3291d2d334be9 -->
