# Historique et confidentialité

## Ce qui reste sur votre ordinateur

Par défaut, l’indexation et la recherche restent locales et les fichiers d’historique de vos agents restent intacts. Mash s’exécute sur votre ordinateur et ne nécessite aucun compte Mash. Il conserve sa propre copie des conversations indexées, qui peuvent contenir du texte sensible. La [démo publique](https://getmash.io/demo/) utilise des conversations d’exemple et n’a pas accès à votre historique local.

Par défaut, les programmes exécutés sur le même ordinateur peuvent accéder
au tableau de bord et au service de recherche installé par Mash sans
authentification. Utilisez cette configuration sur un ordinateur dont les
utilisateurs sont de confiance. Depuis un autre appareil, utilisez le
[tableau de bord avec authentification](dashboard.md#open-it-from-your-phone-or-another-computer).

Lors de la lecture de l’historique d’un agent qui utilise SQLite, comme OpenCode, des fichiers
`-wal` et `-shm` peuvent apparaître à côté de la base de données. SQLite crée ces fichiers pour toute lecture. Le contenu de la base reste intact.

<span id="where-mash-stores-data"></span>

## Où Mash conserve les données

Par défaut, Mash conserve vos conversations indexées dans
`~/.mash/mash.sqlite3` et ses réglages dans `~/.mash/config.toml`. Sous
Windows, `~` désigne votre profil utilisateur. Qdrant stocke à part les
données de recherche, dont le texte consultable. Sous Windows, Mash utilise
un chemin court sur le disque système, généralement `C:\mash-qdrant\storage`.
[Fichiers du service de recherche selon le système](local-search.md#where-qdrant-stores-data)

Exécutez `mash status` pour voir `database_path` et, si Mash a installé Qdrant,
`local_qdrant.installed.storage`. Exécutez `mash uninstall` pour voir ce qui
serait supprimé, y compris le dossier séparé de Qdrant sous Windows. Ajoutez
`--yes` quand vous êtes prêt à désinstaller Mash.

<span id="when-history-can-leave-your-machine"></span>

## Quand votre historique peut quitter votre ordinateur

* **La connexion aux agents** permet à un agent connecté de demander des résultats et des extraits. Ils entrent dans son contexte et peuvent être envoyés à son fournisseur de modèle. Les [actions automatiques du module](plugin.md#what-happens-without-asking) peuvent aussi ajouter des extraits au contexte sans connexion MCP. Pour chercher uniquement depuis le tableau de bord local ou le terminal, laissez les agents déconnectés et désactivez ou retirez le module Mash si vous l’avez installé.
* **Le partage entre ordinateurs** envoie les conversations et pièces jointes conservées dans l’index vers le serveur Mash que vous avez configuré. Chaque ordinateur doté d’un jeton pour ce groupe peut les lire. [Vérifiez ce qui sera partagé avant la publication](shared-index.md#before-sharing).
* **Un service distant de calcul des embeddings** reçoit le texte préparé pour la recherche, qui peut contenir des données sensibles. Par défaut, le calcul se fait sur votre ordinateur. Les embeddings des requêtes restent toujours locaux.
* **Un service distant de reclassement** reçoit chaque requête et les extraits à classer. Le reclassement se fait localement tant que vous ne choisissez pas de service distant.
* **L’assistant du tableau de bord** envoie vos questions, ainsi que les conversations et éléments de mémoire qu’il lit pour y répondre, au
  [service de modèle que vous configurez](../configuration.md#the-dashboard-assistant). L’assistant est disponible uniquement si vous le configurez.
* **L’extraction de souvenirs à partir d’anciennes conversations** envoie les conversations retenues au fournisseur de l’agent auquel vous êtes connecté et que vous choisissez. Mash vous demande votre accord pendant la configuration.
  [Comment sont extraits les souvenirs](../records.md#backfill-earlier-memory)

## Essayer Mash avec un historique limité

[Essayez une recherche avec des conversations d’exemple](https://getmash.io/demo/#/search?q=invoice+exports+queue).
Pour essayer Mash avec l’un de vos propres historiques :

1. Lancez le [programme d’installation du guide de démarrage](quick-start.md)
   dans votre terminal. Il démarre le service de recherche local avant d’ouvrir l’assistant.
2. Si l’assistant commence par un récapitulatif, choisissez **Customize setup**.
   Lorsqu’il demande où chercher, choisissez **Just this computer (recommended)**.
   Gardez un historique avec une conversation récente, choisissez **Custom number of days**
   et saisissez 7. Si l’assistant vous le demande, décochez tous les agents et
   refusez l’extraction de souvenirs des anciennes conversations.
3. Choisissez **Set up and index**, puis lancez `mash dash`. Ouvrez l’adresse
   affichée, sélectionnez **chats** et cherchez un mot de cette conversation.

Pour utiliser tous les historiques proposés sans connecter d’agents, choisissez
**Set up and index without connecting agents** au premier récapitulatif.
Vous pouvez aussi [faire ces choix dans le navigateur](dashboard.md#set-up-mash-from-the-dashboard).

Si vous décidez de désinstaller Mash, `mash uninstall` affiche les éléments qui seront retirés avant votre confirmation. Vos conversations d’origine restent dans les agents.
[Voir ce que retire la désinstallation](installation.md#uninstall)

## Informations sensibles

Par défaut, Mash masque les identifiants secrets qu’il reconnaît dans les résultats et l’historique qu’il envoie en partage. La détection peut en manquer. Relisez tout ce que vous comptez partager.

L’historique local et les sauvegardes portables peuvent encore contenir des identifiants secrets. Conservez-les en privé, comme les conversations d’origine.

Dans un même message, le texte entre `<private>…</private>` est exclu du texte du message que Mash enregistre. Les deux balises doivent être présentes. Le texte reste dans les fichiers de l’agent et le contenu des pièces jointes reste visible.

## Retirer une conversation

Trouvez son `chat_id` avec `mash list-chats` ou `mash search`, puis exécutez la commande suivante.

```sh
mash forget CHAT_ID
```

Mash retire la conversation de son index local et mémorise ce retrait pour ne pas la réimporter. Si le partage est activé, Mash tente aussi de transmettre la suppression. Consultez le résultat de la commande pour voir si une opération est encore en attente.

Les fichiers d’origine de l’agent, les sauvegardes précédentes et les copies indexées séparément sur un autre ordinateur restent en place. L’opération est irréversible dans Mash. Faites
[une sauvegarde](../upgrades.md) avant de continuer si vous pensez avoir besoin de cette conversation plus tard.

<!-- mash-docs-source: c658788724aed122c6f22e902d0e80c403d672a51dbf4a302fb1d990410ed8fd -->
