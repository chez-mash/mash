# Agents compatibles

Recherchez vos conversations enregistrées dans le
[tableau de bord web](dashboard.md) avec `mash dash` ou dans le
[terminal](../search.md). Vous pouvez utiliser les deux pour tous les
historiques du tableau.

Vous pouvez aussi connecter un agent et lui demander de retrouver des travaux
antérieurs. La connexion est facultative. Un agent connecté peut chercher dans
tous les historiques que vous avez indexés, y compris ceux des autres agents.
[Instructions de connexion](agents.md#connect-claude-code-codex-or-copilot)

[Configurez Mash et retrouvez votre première conversation](quick-start.md).

| Agent | Conversations enregistrées que Mash peut lire | Connexion avec `mash init` |
| --- | --- | --- |
| Claude Code | Conversations locales de Claude Code | Oui |
| Codex | Conversations locales de Codex | Oui, via la CLI de Codex |
| GitHub Copilot CLI | Sessions locales en ligne de commande | Versions de développement. [Connexion manuelle avec v0.1.1-alpha](plugin.md#github-copilot-cli) |
| GitHub Copilot Chat dans VS Code | Conversations enregistrées par VS Code sur votre ordinateur | — |
| OpenCode | Conversations dans la base de données locale d’OpenCode | — |
| Cursor | Conversations en ligne de commande et formats compatibles de l’éditeur | — |
| Antigravity | Conversations enregistrées dans les bases locales de l’application et de l’IDE | — |

Si votre client prend en charge les serveurs MCP locaux via les flux d’entrée
et de sortie standard, consultez la [configuration manuelle (en anglais)](../mcp.md#connect-another-mcp-client).

<span id="if-a-chat-is-missing"></span>

## Si une conversation manque

Lancez `mash init` pour voir quels agents Mash a détectés et quels historiques
il va lire. Si Mash est déjà configuré, choisissez **Customize setup** pour
modifier votre sélection. Si vous conservez un historique ailleurs, ajoutez
son emplacement en suivant le [guide de configuration (en anglais)](../configuration.md#choose-histories-and-an-age-limit).

Lancez `mash index` pour lire les nouvelles conversations. Par défaut, Mash
commence par les 90 derniers jours. Découvrez
[comment inclure les conversations plus anciennes](../indexing.md#add-older-history)
et [où sont conservées les sessions distantes de VS Code](../indexing.md#copilot-in-vs-code-remote-windows).
Ces deux guides sont en anglais.

<span id="tested-with"></span>

## Tests par système d’exploitation

Voici comment l’indexation et la recherche ont été testées pour ces historiques.

| Agent | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Claude Code | Usage quotidien | Usage quotidien | Usage ponctuel |
| Codex | Usage quotidien | Usage quotidien | Usage ponctuel |
| GitHub Copilot | Usage ponctuel | Usage ponctuel | Fichiers de test seuls |
| OpenCode | Fichiers de test seuls | Usage ponctuel | Simulation en CI |
| Antigravity | Fichiers de test seuls | Usage ponctuel | Fichiers de test seuls |
| Cursor | Fichiers de test seuls | Partiel | Fichiers de test seuls |

- **Usage quotidien** couvre des centaines de conversations réelles d’un agent installé sur ce système.
- **Usage ponctuel** couvre moins de vingt conversations réelles d’un agent installé.
- **Simulation en CI** signifie qu’un test automatisé écrit l’historique comme
  le ferait l’agent pendant que Mash l’indexe.
- **Partiel** signifie que Mash a lu les conversations d’une véritable
  installation de Cursor CLI. Certains anciens formats de l’éditeur sont incompatibles.
- **Fichiers de test seuls** signifie que les tests utilisent des données au format
  de l’agent. La vérification sur une véritable installation reste à faire sur ce système.

Les tests [Linux](https://github.com/mashium/mash/actions/workflows/ci.yml) et
[Windows](https://github.com/mashium/mash/actions/workflows/windows.yml) sont
exécutés à chaque envoi de changements. Les
[vérifications de compilation et de démarrage sur les Mac à puce Apple](https://github.com/mashium/mash/actions/workflows/macos-build.yml)
sont lancées à la demande. Si certaines conversations manquent ou s’affichent
mal, vous pouvez [signaler le problème](https://github.com/mashium/mash/issues)
en indiquant le nom de l’agent, sa version et le système d’exploitation.

<!-- mash-docs-source: 103ac8fd4f6f882272bc1d643a5e8f893430be72a0b0625ebac257e648edca39 -->
