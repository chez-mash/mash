# Guide de démarrage rapide

[Essayez une recherche dans la démo](https://getmash.io/demo/#/search?q=invoice+exports+queue),
puis ouvrez un résultat dans sa conversation. Configurez Mash ci-dessous
pour chercher dans votre historique depuis le navigateur, le terminal ou un
agent que vous connectez.

Avant de lancer une commande, voyez [ce que la configuration change](installation.md#what-setup-changes)
et [comment vérifier le téléchargement](downloads.md#verify-the-checksum).
Pour commencer avec un seul agent et une période courte, consultez le
[guide d’essai local](privacy.md#essayer-mash-avec-un-historique-limité).

## Demandez à votre agent

Collez ce texte dans Claude Code, Codex ou un autre agent de programmation.

> Lisez <https://getmash.io/setup.md> et configurez Mash sur cet ordinateur.
> Passez d’abord les choix en revue avec moi.

Vous pouvez commencer par le tableau de bord et choisir si vous souhaitez connecter des agents.

Si vous avez installé le [plugin Mash](plugin.md#start-with-the-plugin),
demandez à votre agent d’utiliser la compétence `setup`.

<span id="install-it-yourself"></span>

## Installez Mash vous-même

Vous devez avoir au moins une conversation enregistrée avec un [agent compatible](supported-agents.md).
Les commandes ci-dessous installent Mash, démarrent la recherche locale et
lancent le tableau de bord pour configurer Mash dans votre navigateur.
Pour Homebrew, apt, Scoop ou un autre gestionnaire de paquets, consultez les
[commandes d’installation](install-methods.md#install-with-a-package-manager).
Vous pouvez aussi [suivre l’assistant dans le terminal](installation.md#set-up-in-a-terminal).

Sous macOS (puce Apple) ou Linux (x86-64 ou ARM64), exécutez cette commande.
Sous Linux ou WSL, elle nécessite une session utilisateur systemd pour démarrer
Qdrant. [Vérifiez votre session ou consultez les autres options](local-search.md#linux-without-a-systemd-user-session).

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --dashboard
```

Sous Windows x64, les données de recherche sont placées dans un dossier au
chemin court sur le disque système, généralement `C:\mash-qdrant\storage`.
[Où Mash conserve les données](privacy.md#where-mash-stores-data). Utilisez PowerShell.

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Dashboard
```

Les scripts vérifient les téléchargements et installent Mash dans le dossier
`.local/bin`. Qdrant (environ 30 Mo) s’exécute sous votre compte et démarre à
l’ouverture de votre session. La recherche par mots fonctionne sans téléchargement
de modèles.

Ouvrez dans votre navigateur l’adresse affichée par l’installateur,
généralement <http://127.0.0.1:8790>. Laissez le terminal ouvert.
Vérifiez les historiques détectés et la période, qui couvre par défaut les
90 derniers jours. Si vous arrivez d’abord sur le choix des historiques,
cliquez sur **Next** pour atteindre le récapitulatif.

- **Cherchez vous-même dans le tableau de bord.** Vérifiez qu’aucun agent n’est sélectionné. Si votre version en sélectionne au départ, choisissez **Skip agent connections** ou désactivez-les sur l’écran des agents.
- **Demandez à un agent de chercher pour vous.** Utilisez **Choose agents to connect** ou sélectionnez les agents sur l’écran prévu à cet effet. Vérifiez lesquels figurent dans le récapitulatif.

Choisissez **Set up and index**. [Étapes de configuration dans le tableau de bord](dashboard.md#set-up-mash-from-the-dashboard)

Si votre service de recherche tourne déjà ou si vous voulez seulement les
exécutables, consultez le [guide d’installation](installation.md).

<span id="try-it"></span>

## Retrouver une première conversation

Lorsque la recherche est prête, ouvrez **chats**. Saisissez un mot dont vous
vous souvenez, par exemple un nom de fonction ou un message d’erreur, puis
appuyez sur Entrée. Ouvrez un résultat pour lire le message dans sa conversation.

Pour rouvrir le tableau de bord plus tard, lancez `mash dash` et ouvrez l’adresse affichée.
Si votre terminal ne trouve pas `mash`, utilisez le chemin complet affiché pendant l’installation.
Vous pouvez aussi [ajouter Mash au chemin de recherche des commandes](troubleshooting.md#mash-command-not-found).
Si une conversation manque, consultez [les vérifications pour une recherche sans résultat](troubleshooting.md#search-returns-no-results).
Consultez le [guide du tableau de bord](dashboard.md#find-and-read-a-chat) ou apprenez à
[chercher depuis le terminal](search.md#your-first-search).

**Cherchez depuis un agent.** Redémarrez les agents que vous avez connectés. Dans Codex, vous pouvez demander « Utilise Mash pour retrouver ce que nous avons décidé sur l’authentification dans Claude Code. Montre-moi le message qui explique la décision. »
[Vérifiez la connexion de l’agent](agents.md#confirm-the-connection) si Mash n’est pas accessible.
Si vous avez passé cette étape, vous pouvez [connecter un agent plus tard](agents.md#connect-claude-code-codex-or-copilot).

Pour rendre les nouvelles conversations accessibles à la recherche, utilisez [`mash index` ou la surveillance en arrière-plan](synchronization.md). Vous pouvez aussi [sauvegarder votre historique](backup.md).

Les menus de Mash sont en anglais. Pour trouver les mots de vos conversations,
consultez [la recherche dans d’autres langues](search-languages.md).

<!-- mash-docs-source: 62b75bc52e78a840f0a366d876bd1ca8c22b81cbfcbb81453c7673ba7f07a347 -->
