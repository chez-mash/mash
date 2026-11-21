# Guide de démarrage rapide

[Essayez une recherche dans la démo](https://getmash.io/demo/#/search?q=invoice+exports+queue),
puis ouvrez un résultat dans sa conversation. Configurez Mash ci-dessous
pour chercher dans votre historique depuis le navigateur, le terminal ou un
agent que vous connectez.

Avant de lancer une commande, vous pouvez
[examiner le script d’installation et vérifier le fichier téléchargé](../installation.md#check-the-release-download).
Pour commencer avec un seul agent et une période courte, consultez le
[guide d’essai local](privacy.md#essayer-mash-avec-un-historique-limité).

**Demandez à votre agent.** Collez ce texte dans Claude Code, Codex ou un autre agent de programmation.

> Lisez <https://getmash.io/setup.md> et configurez Mash sur cet ordinateur.

Vous pouvez commencer par le tableau de bord et choisir si vous souhaitez connecter des agents.

Si vous avez installé le [plugin Mash](../plugin.md#start-with-the-plugin),
demandez à votre agent d’utiliser la compétence `setup`.

**Configurez Mash vous-même.** Pour une première installation locale, exécutez
l’une de ces commandes dans un terminal. Elle installe Mash et le service de
recherche local, puis vous invite à vérifier quels historiques lire et quels
agents connecter avant l’indexation.

Sous macOS (puce Apple) ou Linux (x86-64 ou ARM64), exécutez cette commande.

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

Sous Windows x64, les données de recherche sont placées dans un dossier au
chemin court sur le disque système, généralement `C:\mash-qdrant\storage`.
[Où Mash conserve les données](privacy.md#where-mash-stores-data). Utilisez PowerShell.

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

Les scripts vérifient les téléchargements et installent Mash dans le dossier
`.local/bin`. Qdrant (environ 30 Mo) s’exécute sous votre compte et démarre à
l’ouverture de votre session. La recherche par mots fonctionne sans téléchargement
de modèles. Sous Linux et WSL, cette configuration nécessite une session
utilisateur systemd.

La période proposée couvre les 90 derniers jours. Vérifiez les options, puis
choisissez **Set up and index** quand vous êtes prêt. Si l’assistant commence
par vous demander où chercher, choisissez **Just this computer (recommended)**.
Si vous utilisez WSL sans systemd, si votre service de recherche tourne déjà
ou si vous voulez seulement les exécutables, consultez le
[guide d’installation](../installation.md). Vous
pouvez aussi [vérifier les options dans le tableau de bord](../dashboard.md#set-up-mash-from-the-dashboard).

<span id="try-it"></span>

## Retrouver une première conversation

Lorsque l’assistant indique que la recherche est prête, lancez le tableau de bord et laissez le terminal ouvert.

```bash
mash dash
```

1. Ouvrez dans votre navigateur l’adresse affichée, généralement <http://127.0.0.1:8790>.
2. Ouvrez **chats**, saisissez un mot dont vous vous souvenez, par exemple un nom de fonction ou un message d’erreur, puis appuyez sur Entrée.
3. Ouvrez un résultat pour lire le message original dans sa conversation.

Si votre terminal ne trouve pas `mash`, utilisez le chemin complet affiché pendant l’installation.
Vous pouvez aussi [ajouter Mash au chemin de recherche des commandes (en anglais)](../troubleshooting.md#mash-command-not-found).
Si une conversation manque, consultez [les vérifications pour une recherche sans résultat (en anglais)](../troubleshooting.md#search-returns-no-results).
Consultez le [guide du tableau de bord](../dashboard.md#find-and-read-a-chat) ou apprenez à
[chercher depuis le terminal](../search.md#your-first-search).

**Cherchez depuis un agent.** Redémarrez les agents que vous avez connectés. Dans Codex, vous pouvez demander « Utilise Mash pour retrouver ce que nous avons décidé sur l’authentification dans Claude Code. Montre-moi le message qui explique la décision. »
[Vérifiez la connexion de l’agent](../agents.md#confirm-the-connection) si Mash n’est pas accessible.

Pour rendre les nouvelles conversations accessibles à la recherche, utilisez [`mash index` ou la surveillance en arrière-plan](../synchronization.md). Vous pouvez aussi [exporter et sauvegarder votre historique](../upgrades.md).

Les menus de Mash sont en anglais. Vous pouvez rechercher les mots français tels qu’ils apparaissent dans la conversation. La recherche par sens nécessite une requête en anglais. [Guide de recherche en anglais](../search.md#search-in-other-languages)

<!-- mash-docs-source: 0a82cdf33baacdf0463346d54110973710b9e53bc2dfbbf946d2455b15978041 -->
