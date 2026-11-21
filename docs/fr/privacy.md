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

<span id="choose-agent-access"></span>

## Choisir ce à quoi vos agents peuvent accéder

Vous pouvez chercher dans vos conversations depuis le tableau de bord local ou
le terminal sans connecter d’agent. Le choix des historiques que Mash lit est
distinct du choix des agents à connecter.

- **Pendant la configuration**, laissez tous les agents décochés pour ignorer
  les nouvelles connexions. Les outils et les modules déjà installés restent
  en place.
  [Configurer Mash sans connecter d’agent](installation.md#skip-agent-connections).
- **Pour retirer une intégration existante**, retirez les outils Mash, puis
  désactivez ou retirez le module Mash et redémarrez l’agent. Faites de même si
  vous avez installé le module séparément.
  [Retirer la connexion d’un agent](agents.md#disconnect-an-agent).
- **Pour arrêter d’indexer un historique**, désactivez-le dans **agent histories**
  sous **settings**, enregistrez, puis relancez l’indexation continue si elle
  était en cours. Les connexions restent actives et les conversations déjà
  indexées restent consultables. [Retirer une conversation de Mash](#remove-a-chat).

Lorsque vous connectez un agent pendant la configuration, Mash enregistre
ses outils et tente d’installer son module.
[Ce que la connexion installe](agents.md#what-the-connection-installs).

<span id="when-history-can-leave-your-machine"></span>

## Quand votre historique peut quitter votre ordinateur

| Comment vous utilisez Mash | Ce qui peut être lu ou envoyé |
| --- | --- |
| Recherche dans le tableau de bord local ou le terminal | Mash consulte l’index sur votre ordinateur et vous affiche les résultats. |
| Outils connectés à un agent (MCP) | L’agent peut chercher dans les conversations indexées et lire les messages correspondants dans leur contexte. Il peut transmettre ce qu’il lit à son fournisseur de modèle. |
| Module Mash avec actions automatiques activées | Au démarrage d’une session, le module peut fournir des détails sur des conversations récentes et des souvenirs enregistrés. Il peut aussi ajouter des extraits quand l’agent lit un fichier associé. Ces éléments peuvent parvenir au fournisseur du modèle, même sans connexion MCP. [Les actions automatiques et comment les désactiver](plugin.md#what-happens-without-asking). |
| Partage entre ordinateurs | Mash envoie les conversations indexées et les pièces jointes conservées à votre serveur. Un ordinateur muni d’un jeton du même groupe peut les lire. Pour arrêter les prochains envois, ouvrez **settings**, choisissez **this machine only** sous **index into**, enregistrez, puis relancez l’indexation continue si elle était en cours. Les copies déjà envoyées restent sur le serveur. [Vérifier ce qui sera partagé](shared-index.md#before-sharing). |
| Mashy dans le tableau de bord | Si vous l’activez, Mashy envoie vos questions, vos passages sélectionnés, ainsi que le texte et les souvenirs qu’il consulte au [service de modèle que vous configurez (en anglais)](../configuration.md#the-dashboard-assistant). Pour interrompre ces envois, désactivez **Mashy** sous **settings** et enregistrez, ou retirez la section `[assistant]` du fichier de configuration. |

- **Un service distant de calcul des embeddings** reçoit le texte préparé pour
  la recherche, qui peut contenir des données sensibles. Par défaut, le calcul
  se fait sur votre ordinateur. Les embeddings des requêtes restent toujours
  locaux. [Calcul des embeddings (en anglais)](../configuration.md#embedding-inference).
- **Un service distant de reclassement** reçoit chaque requête et les extraits
  à classer. Le reclassement se fait sur votre ordinateur par défaut.
  [Reclassement distant (en anglais)](../configuration.md#remote-reranking).
- **L’extraction de souvenirs à partir d’anciennes conversations** envoie les
  conversations retenues au fournisseur de l’agent auquel vous êtes connecté
  et que vous choisissez. Mash vous demande votre accord pendant la configuration.
  [Comment sont extraits les souvenirs](records.md#backfill-earlier-memory).

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

<span id="try-a-small-local-setup"></span>

## Essayer Mash avec un historique limité

[Essayez une recherche avec des conversations d’exemple](https://getmash.io/demo/#/search?q=invoice+exports+queue).
Pour essayer Mash avec l’un de vos propres historiques sur une nouvelle installation :

1. Lancez le [programme d’installation du guide de démarrage](quick-start.md#install-it-yourself).
   Ouvrez l’adresse affichée dans votre navigateur et laissez le terminal ouvert.
2. Si le récapitulatif apparaît d’abord, choisissez **Customize setup**.
   Gardez uniquement l’historique d’un agent utilisé ces sept derniers jours,
   puis choisissez **Next**.
3. Choisissez **another number of days** et saisissez **7** pour sélectionner
   l’historique mis à jour au cours des sept derniers jours. Mash se fie à
   l’activité enregistrée de la conversation ou, à défaut, à la date de
   dernière modification du fichier d’historique. Les conversations
   indexées peuvent contenir des messages plus anciens. Choisissez **Next**,
   désactivez les connexions aux agents sélectionnées et passez au récapitulatif.
4. Vérifiez que le récapitulatif indique **this machine only** et **no agents**.
   Choisissez **Set up and index**. Lorsque l’indexation est terminée, ouvrez
   **chats** et cherchez un mot de cette conversation.

Vous pourrez élargir la période ou connecter un agent plus tard.
[Configuration dans le tableau de bord](dashboard.md#set-up-mash-from-the-dashboard)

Si vous décidez de désinstaller Mash, `mash uninstall` affiche les éléments qui seront retirés avant votre confirmation. Vos conversations d’origine restent dans les agents.
[Voir ce que retire la désinstallation](uninstall.md)

<span id="shorten-the-history-range"></span>

## Réduire la période de l’historique

Ouvrez **settings** dans le tableau de bord. Dans **history range**, choisissez
**recent updates** (**recent only** dans les versions précédentes) et indiquez
le nombre de jours. Choisissez **Review changes**, puis **Save to file**.
Pour appliquer les paramètres enregistrés, exécutez
`mash index` ou redémarrez l’indexation continue si elle est en cours.

La période s’applique aux prochaines indexations. Si vous passez de 90 à
7 jours, une conversation déjà indexée dont la dernière mise à jour remonte
à 30 jours reste accessible par la recherche, même après une nouvelle
indexation. Une conversation mise à jour récemment peut contenir des messages
plus anciens. Pour retirer une conversation précise de Mash, consultez
[Retirer une conversation](#remove-a-chat).

## Informations sensibles

Par défaut, Mash masque les identifiants secrets qu’il reconnaît dans les résultats et l’historique qu’il envoie en partage. La détection peut en manquer. Relisez tout ce que vous comptez partager.

L’historique local et les sauvegardes portables peuvent encore contenir des identifiants secrets. Conservez-les en privé, comme les conversations d’origine.

Dans un même message, le texte entre `<private>…</private>` est exclu du texte du message que Mash enregistre. Les deux balises doivent être présentes. Le texte reste dans les fichiers de l’agent et le contenu des pièces jointes reste visible.

<span id="remove-a-chat"></span>

## Retirer une conversation

Trouvez son `chat_id` avec `mash list-chats` ou `mash search`, puis exécutez la commande suivante.

```sh
mash forget CHAT_ID
```

Mash retire la conversation de sa copie locale et supprime les souvenirs
enregistrés qui l’ont citée. Il mémorise ce retrait pour éviter de réimporter
la conversation. Si le partage est activé, Mash tente aussi de transmettre
la suppression. Consultez le résultat de la commande pour voir si une
opération est encore en attente.

Les fichiers d’origine de l’agent, les sauvegardes précédentes et les copies indexées séparément sur un autre ordinateur restent en place. L’opération est irréversible dans Mash. Faites
[une sauvegarde](backup.md#make-a-portable-backup) avant de continuer si vous pensez avoir besoin de cette conversation plus tard.

<!-- mash-docs-source: 8ece8fc94aa9d9e86e4d6bb12c10445fe9ed0dbbac23a62ccb44e9bfd12a8496 -->
