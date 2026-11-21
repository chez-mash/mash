# Guide de démarrage rapide

<span id="quick-start"></span>

<span id="install-it-yourself"></span>

## Installez Mash vous-même

Exécutez la commande correspondant à votre système pour télécharger Mash et
lancer la configuration dans votre navigateur. Il vous faut au moins une
conversation enregistrée par un [agent compatible](supported-agents.md).
Consultez [ce que la configuration change](installation.md#what-setup-changes)
et [comment vérifier le téléchargement](downloads.md#verify-the-checksum).

**macOS (puce Apple) ou Linux (x86-64 ou ARM64)**

Sous Linux ou WSL, une [session utilisateur systemd](local-search.md#linux-without-a-systemd-user-session) est nécessaire.

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --dashboard
```

**Windows x64 (PowerShell)**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Dashboard
```

Ouvrez le lien affiché par l’installateur et laissez le terminal ouvert.
Suivez les étapes dans votre navigateur, puis choisissez **Set up and index**.
Vous pouvez passer l’étape de connexion des agents ou décocher ceux déjà
sélectionnés. [Étapes de configuration dans le tableau de bord](dashboard.md#set-up-mash-from-the-dashboard)

<span id="try-it"></span>
<span id="find-your-first-chat"></span>

## Retrouver une première conversation

Lorsque la recherche est prête, ouvrez **chats**. Saisissez un mot dont vous
vous souvenez, par exemple un nom de fonction ou un message d’erreur, puis
appuyez sur Entrée. Ouvrez un résultat pour lire le message dans sa conversation.

Pour rouvrir le tableau de bord plus tard, lancez `mash dash` et ouvrez l’adresse affichée.
Si le terminal ne trouve pas `mash`, utilisez le chemin complet affiché par
l’installateur ou [ajoutez Mash au chemin de recherche des commandes](troubleshooting.md#mash-command-not-found).
[Trouvez et lisez une conversation dans le tableau de bord](dashboard.md#find-and-read-a-chat) ·
[Cherchez depuis le terminal](search.md#your-first-search) ·
[Que faire si la recherche ne renvoie aucun résultat](troubleshooting.md#search-returns-no-results)

<span id="ask-your-agent"></span>

## Demandez à votre agent

Collez ce texte dans Claude Code, Codex ou un autre agent de programmation.

> Lisez <https://getmash.io/setup.md> et configurez Mash sur cet ordinateur.
> Passez d’abord les choix en revue avec moi.

Si vous avez déjà installé le [plugin Mash](plugin.md#start-with-the-plugin),
demandez à votre agent d’utiliser la compétence `setup`. Si vous avez installé
Mash vous-même, vous pouvez [connecter un agent plus tard](agents.md#connect-claude-code-codex-or-copilot).

Pour utiliser un gestionnaire de paquets, compiler depuis le code source ou
configurer Mash dans le terminal, consultez le [guide d’installation](installation.md).
Pour commencer avec moins d’historique, suivez le
[guide d’essai local](privacy.md#essayer-mash-avec-un-historique-limité).

<!-- mash-docs-source: d57d2515612c6e32065f147541df1dc3c18acd5e701eed2c2fd739454faeaf4a -->
