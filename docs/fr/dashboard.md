# Utiliser le tableau de bord de Mash

Dans la démo, [recherchez les exports de factures](https://getmash.io/demo/#/search?q=invoice+exports+queue). Ouvrez un résultat pour retrouver la conversation d’origine. Vous pouvez aussi [ouvrir une décision enregistrée](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop) et choisir **open the message** pour voir d’où elle vient. La démo recherche les mots et expressions exacts dans des conversations d’exemple.

Les versions précompilées de Mash incluent le tableau de bord. Si vous compilez Mash, suivez les étapes pour [inclure le tableau de bord](../install-methods.md#include-the-web-dashboard). Après avoir [installé Mash](quick-start.md), lancez cette commande dans un terminal.

```sh
mash dash
```

Ouvrez dans votre navigateur l’adresse affichée, généralement **[http://127.0.0.1:8790](http://127.0.0.1:8790)**. Une fois Mash configuré, la vue d’ensemble affiche les conversations et souvenirs récents. Laissez le terminal ouvert. Ctrl-C arrête le tableau de bord. Si la page ne se charge pas, [vérifiez le démarrage du tableau de bord](troubleshooting.md#the-dashboard-doesnt-open).

Avec le [module Mash](plugin.md), utilisez `/mash:dashboard` dans Claude Code ou demandez à votre agent d’ouvrir le tableau de bord.

Avec les réglages par défaut, le tableau de bord local s’ouvre sans authentification. Consultez [comment limiter l’accès](#keep-access-private) avant de changer l’adresse d’écoute. Les libellés de l’interface sont en anglais.

<span id="set-up-mash-from-the-dashboard"></span>

## Configurer Mash depuis le tableau de bord

Vous pouvez faire la première configuration dans le navigateur à la place de `mash init`. Lancez `mash dash` et ouvrez le tableau de bord. Sur une machine sans configuration, l’assistant s’affiche sur un écran sans barre latérale ni recherche. **Skip to the dashboard** permet d’accéder au tableau de bord. Vous pouvez revenir à la configuration depuis **settings**.

Vérifiez les historiques trouvés, la période et les agents que Mash peut connecter pour qu’ils puissent faire des recherches. **Show history folders** affiche les emplacements. Par défaut, une nouvelle configuration indexe les 90 derniers jours sur cette machine. Choisissez **Set up and index** pour valider ou **Customize setup** pour modifier les historiques, la période ou les connexions des agents.

Sur le premier écran de confirmation, **Skip agent connections** indexe les mêmes historiques pour vos recherches dans le navigateur ou le terminal. Si votre version commence par le choix des historiques, utilisez **Next** pour accéder à la confirmation.

Mash applique les changements quand vous choisissez **Set up and index** ou **Set up without indexing** sur l’écran de confirmation. Il écrit sa configuration, utilise la commande de chaque agent choisi pour le connecter et lance l’indexation en arrière-plan si vous l’avez demandée. La page indique ce qui a été enregistré, les fichiers modifiés par les agents et les commandes pour annuler leurs connexions. Elle suit aussi l’indexation jusqu’à l’apparition des conversations.

- L’indexation nécessite le service de recherche. S’il est arrêté, lancez `mash qdrant install` puis choisissez **Check search service again**, ou utilisez **Set up without indexing** pour indexer plus tard. La confirmation de la configuration personnalisée propose aussi l’enregistrement sans indexation lorsque le service fonctionne.
- Pour chercher par le sens, [installez les modèles et préparez l’index](../indexing.md#enable-semantic-search). Vous pourrez ensuite [choisir la recherche par le sens pour une requête](#choose-a-search-mode).
- Si vous refaites la configuration sur une machine déjà configurée, Mash enregistre vos choix et conserve le fichier précédent à côté du nouveau.
- Pour partager entre machines, extraire des souvenirs d’anciennes conversations ou lire des historiques stockés ailleurs, utilisez `mash init` ou modifiez le fichier de configuration.
- Si le tableau de bord est accessible depuis d’autres machines, la configuration dans le navigateur requiert une configuration de services au démarrage. Il en va de même pour la page des paramètres.

<span id="the-rail"></span>

## Se repérer dans la barre latérale

Sur un téléphone, utilisez le bouton de menu pour l’ouvrir.

- **overview** affiche vos dernières conversations et vos souvenirs, ainsi que le nombre de conversations par machine et par agent.
- **chats** liste les conversations de la plus récente à la plus ancienne et propose un champ de recherche. **viewing history** liste celles ouvertes dans cette session du navigateur et les rouvre à l’endroit où vous vous êtes arrêté.
- **agents**, **machines** et **projects** ont chacun leur page, avec les conversations récentes et leur répartition entre les deux autres catégories.
- **memory** réunit les décisions, préférences et faits enregistrés.
- **services** et **fleet manager** affichent l’état des processus et des bases, la configuration des agents par machine et les actions de gestion. Le gestionnaire de parc s’affiche lorsque la gestion de plusieurs machines est configurée. [Services](../services.md)
- **tool explorer** affiche les outils à la disposition des agents et leurs paramètres. Vous pouvez tester un outil en lecture seule pour voir la réponse que reçoit l’agent. Les outils qui écrivent des données figurent aussi dans la liste.
- **settings** affiche la configuration de la machine et l’état de l’index. Vous pouvez choisir les agents indexés, la période, la publication vers l’index partagé, la capture de souvenirs et le masquage des identifiants secrets. Si [Mashy](#ask-mashy) est configuré, vous pouvez l’activer, le désactiver ou changer son modèle. Mash conserve le fichier précédent sous le nom `config.toml.previous`. Redémarrez l’indexation continue et reconnectez les agents pour appliquer ces paramètres. Mashy applique les siens aussitôt. Les paramètres de recherche et de stockage sont consultables. Pour les modifications depuis un tableau de bord accessible à d’autres machines, démarrez-le avec `--services-config`.

Le bouton en haut à droite alterne entre les thèmes clair et sombre. Le navigateur mémorise votre choix.

<span id="find-and-read-a-chat"></span>

## Rechercher et lire une conversation

Ouvrez **chats**, tapez un mot dont vous vous souvenez dans **Search your history**, puis appuyez sur Entrée. Ouvrez un résultat pour aller au message correspondant dans sa conversation.

Sans requête, la page liste vos conversations. Les filtres par agent, machine, projet et date s’appliquent à la liste comme aux résultats de recherche. Un filtre modifié a une bordure en pointillés jusqu’à ce que vous choisissiez **Apply filters**. Les filtres appliqués apparaissent sous forme d’étiquettes dans le champ de recherche. Cliquez sur une étiquette pour la retirer.

Si une recherche filtrée reste vide, choisissez **Clear filters** pour relancer la même recherche sur vos conversations. Si votre version revient à la liste, saisissez de nouveau la requête. Choisissez **Browse chats** pour ouvrir la liste avec les filtres actuels. Utilisez le bouton Précédent du navigateur pour revenir à la recherche. Pour aller plus loin, [voyez comment retrouver la conversation](troubleshooting.md#search-returns-no-results).

Dans une conversation, vos messages sont à droite et les réponses de l’agent à gauche. Les appels d’outils et le raisonnement sont regroupés sur une ligne qui s’ouvre pour montrer chaque étape et sa sortie. Le panneau latéral liste vos demandes de la page en cours, les autres conversations du projet et les souvenirs tirés de celle-ci. **Copy session ID** copie l’identifiant pour reprendre la conversation dans son agent.

Les vues des conversations et des souvenirs sont en lecture seule.

<span id="choose-a-search-mode"></span>

### Choisir un mode de recherche

Après une recherche, repérez **search: default** près des filtres. À l’arrivée des résultats, le libellé affiche **default (words)** ou **default (meaning)**. Ouvrez le menu, choisissez **words** ou **meaning**, puis **Apply filters** pour relancer la recherche. **words** cherche les termes de la conversation. **meaning** utilise des [modèles facultatifs](../indexing.md#enable-semantic-search) et accepte les requêtes en anglais. Le choix reste actif sur les différentes pages de résultats. Une nouvelle requête reprend le [mode par défaut](../configuration.md#search-defaults).

Si le tableau de bord n’affiche pas le menu, utilisez [`mash search --intent`](../search.md).

<span id="inspect-conversation-records"></span>

## Consulter les souvenirs enregistrés

Ouvrez **memory** dans la barre latérale ou choisissez une catégorie. Les boutons en haut changent la catégorie. Avec **find a statement**, recherchez du texte et filtrez par état, révision ou origine. L’origine correspond à cette machine ou au groupe partagé. Choisissez **Apply filters** pour chercher et **Next page** pour poursuivre.

Ouvrez un souvenir pour voir ses révisions, les messages cités et les souvenirs associés. Sous une citation, **open the message** ouvre le message source. Dans une conversation, **memory from this chat** liste les souvenirs qui citent cette conversation dans une révision conservée. Les messages cités affichent **Saved as**.

Les listes contiennent aussi des souvenirs en attente ou retirés. Vérifiez l’état, la date d’expiration, les avertissements sur les sources et les conflits signalés avant de vous y fier. Pour confirmer ou modifier un souvenir, demandez à votre agent d’utiliser les [outils de mémoire](../records.md). Si la source d’un souvenir partagé est indisponible, son lien affiche une erreur.

<span id="keep-your-place"></span>

## Garder sa place pendant la lecture

Le tableau de bord suit les mises à jour de cette machine et de celles qui publient dans l’index partagé. Votre lecture reste à la même position.

- **Les listes de conversations et de souvenirs** se mettent à jour en quelques secondes. Cela comprend la vue d’ensemble, la barre latérale, les conversations récentes, la liste à côté d’une conversation et les pages des agents, machines, projets et souvenirs.
- **Une conversation ouverte** ajoute les messages à la fin et vous y amène si vous êtes déjà tout en bas. Ailleurs, utilisez le bouton **N new messages** pour accéder aux nouveaux messages. Si Mash a relu un message ancien, **Show updated messages** applique les changements au texte affiché.
- **Une recherche envoyée**, de conversations ou de souvenirs, garde ses résultats et ses filtres jusqu’à ce que vous la relanciez, même après avoir ouvert un résultat puis utilisé le bouton Précédent.
- **Un souvenir ouvert** se met à jour quand vous choisissez **check for updates**.

Les conversations et souvenirs des autres machines apparaissent quand celles-ci les publient. Le serveur partagé doit être au moins aussi récent que le tableau de bord. Avec un serveur plus ancien ou plus de 16 onglets ouverts, utilisez **reload chat list** au bas de la barre latérale ou le bouton de rafraîchissement d’une conversation.

Si la connexion du navigateur ou celle vers le serveur partagé est coupée, le tableau de bord se reconnecte et rattrape les mises à jour.

Les nouvelles conversations apparaissent après indexation. Lancez `mash index` ou laissez tourner `mash index --watch` dans un autre terminal. Le tableau de bord affiche les mises à jour dans les deux cas.

Si une requête échoue, le tableau de bord indique ce qui n’a pas pu être chargé et propose **Retry**. Une erreur de connexion indique que le navigateur ne parvient pas à joindre Mash. Vérifiez la connexion et que `mash dash` tourne encore, puis choisissez **Retry**.

<span id="ask-mashy"></span>

## Poser une question à Mashy

Si vous avez [configuré un modèle](../configuration.md#the-dashboard-assistant), la mascotte de Mash apparaît dans un cercle en bas à droite de chaque page. Ouvrez-la et posez une question sur votre historique avec vos propres mots.

- « Sur quoi ai-je travaillé hier ? »
- « Qu’avons-nous décidé pour le test de redémarrage ? »
- Sur la page d’une conversation, demandez par exemple « Résume cette conversation. »
- « Pourquoi je ne retrouve pas la conversation de ce matin ? »

Sur un grand écran, déplacez la fenêtre par sa barre supérieure pour dégager le texte que vous lisez. Double-cliquez sur la barre pour remettre la fenêtre à sa place. Sur un téléphone, elle occupe tout l’écran.

Mashy cherche dans les conversations et souvenirs et répond brièvement. Ses références s’ouvrent dans le tableau de bord.

- Une conversation ou un souvenir apparaît sous forme d’étiquette. Si la réponse s’appuie sur un message, l’étiquette ouvre la conversation à ce message.
- Les citations sont affichées avec leur auteur et leur date. Vos propres mots portent une bordure colorée.
- Un projet, un agent ou une machine ouvre sa page.
- Si la réponse couvre seulement une partie des résultats, **See all** ouvre la recherche complète avec les mêmes mots.
- Une commande comme `mash index` a un bouton **Copy**.

Mashy propose des questions à la suite de sa réponse et indique s’il n’a rien trouvé. Utilisez le bouton d’arrêt pour interrompre une réponse.

Pour poser une question sur un passage, sélectionnez le texte et choisissez **Ask Mashy**. Le passage reste épinglé au-dessus du chat et accompagne chaque question jusqu’à ce que vous le retiriez.

Mashy peut aussi vérifier si la recherche est disponible, quelle est la conversation la plus récente de l’index, quels agents sont connectés et quand les autres machines se sont synchronisées. En cas de problème, il indique la commande à lancer.

Il répond également sur Mash à partir des guides intégrés. Demandez par exemple « Comment partager mon index avec mon portable ? » ou « Que change `mash init` ? ». Les guides cités s’ouvrent sur ce site.

Les dates et heures suivent l’horloge du navigateur. Pour les questions sur une période, Mashy examine les conversations qui ont reçu des messages pendant cette période. Il lit les longues conversations par parties et précise celle que couvre la réponse.

Mashy utilise des outils en lecture seule pour consulter votre historique et les guides intégrés. Il ne peut ni indexer, ni supprimer, ni enregistrer des souvenirs, ni lancer des commandes ou modifier les paramètres. Sa conversation dure le temps de l’onglet et reste hors de votre historique.

À chaque question, le texte que Mashy lit, y compris le passage sélectionné, est envoyé au modèle que vous avez configuré. Mashy s’affiche et envoie du texte lorsque vous avez configuré ce modèle.

Si Mashy échoue ou répond mal, lancez le tableau de bord avec `MASH_TRACE=1` et reposez la question. Le journal détaille les consultations, la quantité de résultats, le temps passé dans le modèle, le nombre de jetons utilisés et la raison de l’arrêt. Les questions, les réponses et le texte de l’historique restent hors du journal.

<span id="save-a-transcript"></span>

## Enregistrer une conversation

Utilisez **Export** dans la conversation pour télécharger le contenu complet en Markdown ou HTML, y compris les résultats des outils. **Export to PDF** ouvre une version imprimable. Choisissez **Save as PDF** dans la boîte de dialogue d’impression.

Attendez la fin de l’indexation pour obtenir une copie stable. Les téléchargements peuvent contenir des informations sensibles.

Chaque conversation a son adresse, `http://127.0.0.1:8790/#/chat/<chat id>`. Avec le [module Mash](plugin.md), `/mash:export` trouve la conversation que vous indiquez et l’ouvre à cet emplacement.

<span id="export-many-chats-at-once"></span>

## Exporter plusieurs conversations à la fois

Ouvrez **chats**, puis **export chats**. Réduisez la liste avec les filtres par agent, machine, projet et période. Les conversations correspondantes sont exportées sur toutes les pages, sauf celles que vous décochez. Choisissez **Markdown** pour la lecture ou **JSON** pour des scripts, puis **Export**. Le navigateur télécharge un fichier `.zip` avec un dossier par projet et un fichier par conversation.

Laissez la page ouverte jusqu’au début du téléchargement. Si une conversation est illisible, l’export s’arrête sans enregistrer de fichier. Les exports comprennent le texte complet, y compris les passages sensibles. Les pièces jointes restent hors de l’archive.

Pour restaurer des conversations dans Mash, faites une [sauvegarde portable](../upgrades.md#make-a-portable-backup). Les exports du tableau de bord servent à lire les conversations. Pour voir les conversations stockées sur d’autres machines, suivez le [guide de partage](../shared-index.md).

<span id="open-it-from-your-phone-or-another-computer"></span>

## Ouvrir le tableau de bord depuis un téléphone ou un autre ordinateur

Par défaut, seule votre machine peut accéder au tableau de bord. Sur Linux ou macOS, si vous êtes connecté à [Tailscale](https://tailscale.com), utilisez cette commande pour y accéder depuis vos autres appareils sur votre réseau.

```sh
mash dash remote
```

Le tableau de bord tourne en service utilisateur avec authentification, via systemd sous Linux ou un LaunchAgent sous macOS. Mash affiche une adresse HTTPS dans votre réseau Tailscale, un code à usage unique et un QR code.

```json
{
  "dashboard": "https://laptop.your-tailnet.ts.net:8790/",
  "service": "mash-dash.service",
  "code": "K7M-2QX",
  "code_minutes": 10
}
```

Depuis un téléphone connecté à Tailscale, scannez le QR code ou ouvrez l’adresse et saisissez le code. Le navigateur reste connecté pendant 30 jours.

| Pour | Commande |
| --- | --- |
| Connecter un autre appareil | `mash dash remote --code` |
| Afficher de nouveau le QR code | `mash dash remote --qr` |
| Utiliser un autre port | `mash dash remote --port 8791` (dans chaque commande) |
| Héberger le tableau de bord sur une machine sans historique local | `mash dash remote --shared-only` |
| Ouvrir depuis un appareil sans Tailscale | `mash dash remote --public --port 8443` |
| Arrêter le service | `mash dash remote --remove` |

Le code s’utilise une fois et expire après dix minutes. Le QR code contient le jeton durable après le `#` dans l’adresse. Le navigateur n’envoie pas ce fragment au serveur lorsqu’il charge la page. Le tableau de bord vous connecte puis retire le jeton de l’adresse. Traitez une image du QR code comme un mot de passe.

Le tableau de bord cherche dans l’historique de la machine et, si vous avez un [index partagé](../shared-index.md), dans celui des autres machines. Pour l’index partagé, lancez `mash dash remote` depuis un terminal où la variable du jeton de la machine est définie. Mash l’enregistre dans `~/.mash/dashboard.env`, que vous seul pouvez lire. Sous macOS, le fichier `~/Library/LaunchAgents/dev.mash.dash.plist` contient les mêmes jetons et bénéficie de la même protection. La sortie est enregistrée dans `~/.mash/dashboard.log`.

Une machine sans historique local peut aussi héberger le tableau de bord, par exemple le serveur qui détient l’index partagé. Avec `--shared-only`, les conversations, les recherches et les souvenirs proviennent de l’index partagé.

`--public` publie l’adresse sur Internet via [Tailscale Funnel](https://tailscale.com/kb/1223/funnel). Elle devient accessible depuis un appareil hors de votre réseau. Tailscale doit autoriser Funnel pour cette machine et seuls les ports 443, 8443 et 10000 peuvent être publiés. Tout le monde peut alors ouvrir la page de connexion. Le jeton et les codes à usage unique protègent tout votre historique. Dix essais incorrects en une minute bloquent temporairement la connexion pour tout le monde. Préférez le réseau privé chaque fois que possible.

<span id="set-it-up-by-hand"></span>

### Configurer l’accès manuellement

Sous Windows ou derrière un autre proxy inverse HTTPS tel que Caddy, suivez ces étapes.

1. Choisissez un jeton d’au moins 32 caractères et placez-le dans une variable d’environnement, par exemple `MASH_DASH_TOKEN`.
2. Donnez une adresse HTTPS au tableau de bord. Avec Tailscale, utilisez `tailscale serve --bg 8790`.
3. Lancez le tableau de bord en indiquant le nom et la variable du jeton.

   ```sh
   mash dash --allow-host laptop.your-tailnet.ts.net --access-token-env MASH_DASH_TOKEN
   ```

`--allow-host` indique le nom que les navigateurs utilisent, avec le port s’il figure dans l’adresse. Répétez l’option pour ajouter des noms. Le changement de jeton déconnecte tous les navigateurs.

<span id="keep-access-private"></span>

## Limiter l’accès au tableau de bord

Avec `--access-token-env`, l’authentification est requise pour chaque page d’historique. Dix jetons incorrects en une minute bloquent la connexion pour tout le monde jusqu’à la fin de la minute.

L’adresse par défaut est accessible seulement depuis votre machine. Si vous utilisez `mash dash --listen` avec une autre adresse sans configurer l’authentification, toute personne qui peut joindre le tableau de bord peut lire l’historique. Mash le signale au démarrage. `--allow-host` exige un jeton.

Préférez un réseau privé. Sur Internet, le jeton protège tout l’historique. Un service qui termine HTTPS à votre place, comme Cloudflare Tunnel, peut lire les données qui y transitent.

<!-- mash-docs-source: 28b1f1fb651fa45b7f3dc82f3643b520feb7262ce55353be612903201cf431d5 -->
