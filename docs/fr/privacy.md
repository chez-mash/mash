# Historique et confidentialité

## Ce qui reste sur votre ordinateur

Par défaut, l’indexation et la recherche restent locales et les fichiers d’historique de vos agents restent intacts. Mash s’exécute sur votre ordinateur et ne nécessite aucun compte Mash. Il conserve sa propre copie des conversations indexées, qui peuvent contenir du texte sensible. Par défaut, le service de recherche installé par Mash n’écoute que sur cet ordinateur. La [démo publique](https://getmash.io/demo/) utilise des conversations d’exemple et n’a pas accès à votre historique local.

Lors de la lecture de l’historique d’un agent qui utilise SQLite, comme OpenCode, des fichiers
`-wal` et `-shm` peuvent apparaître à côté de la base de données. SQLite crée ces fichiers pour toute lecture. Le contenu de la base reste intact.

## Quand votre historique peut quitter votre ordinateur

* **La connexion aux agents** permet à un agent connecté de demander des résultats et des extraits. Ils entrent dans son contexte et peuvent être envoyés à son fournisseur de modèle. Pour chercher vous-même, utilisez le tableau de bord local ou le terminal.
* **Le partage entre ordinateurs** transfère l’historique vers le serveur Mash que vous avez configuré.
* **Un service distant de calcul des embeddings** reçoit le texte préparé pour la recherche, qui peut contenir des données sensibles. Par défaut, le calcul se fait sur votre ordinateur. Les embeddings des requêtes restent toujours locaux.
* **Un service distant de reclassement** reçoit chaque requête et les extraits à classer. Le reclassement se fait localement tant que vous ne choisissez pas de service distant.
* **L’assistant du tableau de bord** envoie vos questions, ainsi que les conversations et éléments de mémoire qu’il lit pour y répondre, au
  [service de modèle que vous configurez](../configuration.md#the-dashboard-assistant). L’assistant est disponible uniquement si vous le configurez.
* **L’extraction de souvenirs à partir d’anciennes conversations** envoie les conversations retenues au fournisseur de l’agent auquel vous êtes connecté et que vous choisissez. Mash vous demande votre accord pendant la configuration.
  [Comment sont extraits les souvenirs](../records.md#backfill-earlier-memory)

Le tableau de bord local s’ouvre sans authentification. Gardez son adresse par
défaut ou limitez l’accès à un réseau privé de confiance.

## Essayer Mash avec un historique limité

Pendant la configuration, choisissez l’historique d’un seul agent et une période courte. Laissez les connexions aux agents désactivées pour éviter que des extraits se retrouvent dans leurs requêtes. Vous pouvez chercher vous-même dans le tableau de bord ou le terminal, puis étendre la période plus tard. [Configurer Mash dans le tableau de bord](../dashboard.md#set-up-mash-from-the-dashboard)

Pour garder les historiques et la période proposés lors de la première configuration, choisissez
**Set up and index without connecting agents** dans le terminal, ou choisissez
**Skip agent connections** sur l’écran de vérification dans le navigateur avant de confirmer. Si votre version commence par une série de questions, désactivez les connexions au moment du choix des agents.

Si vous décidez de désinstaller Mash, `mash uninstall` affiche les éléments qui seront retirés avant votre confirmation. Vos conversations d’origine restent dans les agents.
[Voir ce que retire la désinstallation](../installation.md#uninstall)

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

<!-- mash-docs-source: 6092dc50a7b4767e4c0a16575170916dbda9351910f9c86d02785225361ea9c1 -->
