# Rechercher sur plusieurs machines

Si votre ordinateur portable et votre ordinateur de bureau enregistrent tous deux des conversations avec des agents, un index partagé vous permet de les retrouver depuis l’un ou l’autre. Chaque ordinateur conserve une copie locale dans laquelle vous pouvez effectuer des recherches et publie son index sur un serveur Mash que vous hébergez.

| Ce que vous voulez faire | Par où commencer |
| --- | --- |
| Lire les conversations d’un ordinateur depuis votre téléphone ou un autre navigateur | [Ouvrir à distance le tableau de bord de cet ordinateur](dashboard.md#open-it-from-your-phone-or-another-computer) |
| Rechercher dans les historiques de conversations indexés sur plusieurs ordinateurs | [Configurer un index partagé](#the-quick-way) |
| Rechercher uniquement dans l’historique de cet ordinateur | [Utiliser le tableau de bord local](dashboard.md#find-and-read-a-chat) |

Pour voir à quoi ressemble un résultat provenant d’une autre machine, lancez une recherche dans la démo et ouvrez **Choose a queue for invoice exports**, une conversation provenant du poste de travail.
[Essayer la recherche d’exemple](https://getmash.io/demo/#/search?q=invoice+exports+queue)

<span id="before-sharing"></span>

## Avant de partager

**La publication envoie au serveur vos conversations indexées et les pièces jointes conservées.**
Tout ordinateur disposant d’un jeton pour le même groupe peut consulter cet historique et y effectuer des recherches. Utilisez un serveur et un réseau auxquels vous faites confiance. Avant de commencer à publier, passez en revue les conversations déjà indexées, y compris les plus anciennes que Mash a conservées.
[Dans quels cas l’historique peut quitter votre machine](privacy.md#when-history-can-leave-your-machine)

<span id="what-runs-where"></span>

## Quels composants tournent sur chaque machine

| Emplacement | Composants | Fréquence de configuration |
| --- | --- | --- |
| Une machine serveur | `mash-server` et une instance Qdrant 1.20 ou ultérieure qui lui est dédiée, tous deux avec un stockage persistant | une seule fois |
| Chaque ordinateur qui indexe des conversations | `mash` avec son index local et son instance Qdrant locale, comme décrit dans le guide d’ [installation](installation.md) | une fois par ordinateur |

Le serveur peut tourner sur l’un de vos ordinateurs ou sur une machine distincte allumée en permanence.
`mash-server` est fourni avec `mash`. Utilisez la même version sur le serveur et sur chaque ordinateur.

Si vous avez déjà un serveur, demandez à son administrateur une chaîne de connexion pour chaque ordinateur. Vous pouvez aussi vous connecter à l’aide de l’URL du serveur, de l’identifiant du groupe, d’un nom pour l’ordinateur (son identifiant de nœud) et du jeton propre à cet ordinateur. [Connecter chaque ordinateur](#2-connect-each-computer)

<span id="the-quick-way"></span>

## Configuration rapide

Suivez cette méthode si le serveur fonctionne sous Linux ou macOS, si Qdrant y est déjà en cours d’exécution et si vous pouvez utiliser Tailscale Serve. Connectez le serveur et les ordinateurs à votre tailnet. Activez MagicDNS et les certificats HTTPS pour ce réseau.
[Démarrer Qdrant](installation.md#start-the-local-search-service) · [Activer HTTPS dans Tailscale](https://tailscale.com/docs/how-to/set-up-https-certificates)

Sur le serveur, ajoutez votre ordinateur portable à un groupe nommé `personal`.

```sh
mash-server --setup --add-node personal/laptop
```

Cette commande enregistre les paramètres du serveur dans `~/.mash/server`, démarre un service utilisateur et affiche son adresse HTTPS privée ainsi que la chaîne de connexion `join`. Pour ajouter un autre ordinateur, relancez la commande sur le serveur avec un autre nom, par exemple
`personal/desktop`. Utilisez une chaîne de connexion distincte pour chaque ordinateur. Elle contient son secret d’accès. Transmettez-la en privé. Si vous relancez la configuration avec un nom déjà ajouté, la commande répare le service et réutilise l’enregistrement existant.

<span id="join-and-review-each-computer"></span>

### Connecter et vérifier chaque ordinateur

Sur l’ordinateur à connecter, installez Mash et démarrez son instance Qdrant locale. Suspendez l’indexation en arrière-plan pendant la vérification. Exécutez ensuite la commande avec la chaîne de connexion complète de cet ordinateur.

```sh
mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents
```

La connexion enregistre les paramètres du serveur, du groupe et du nœud, ainsi que le secret d’accès dans `~/.mash/node.env`, que vous seul pouvez lire. Ces options permettent de sauter l’indexation et de conserver les paramètres actuels des agents. La commande `mash init --join` utilisée seule connecte aussi les agents détectés qui sont pris en charge, puis lance l’indexation et la publication sans passer par l’assistant.

Exécutez `mash status --installation` pour vérifier les dossiers d’historique configurés, puis consultez les conversations déjà indexées dans le tableau de bord local. Quand vous êtes prêt à envoyer cet historique au serveur, [publiez et vérifiez chaque ordinateur](#3-publish-and-check).

Dans cette configuration, le serveur effectue des recherches par mots et n’a besoin de télécharger aucun modèle. Si l’hôte n’utilise pas Tailscale ou si le serveur doit offrir la recherche sémantique, suivez la procédure manuelle ci-dessous. Pour mettre hors ligne un serveur créé avec `--setup`, exécutez
`mash-server --setup --remove`. Les paramètres et l’historique seront conservés jusqu’au prochain démarrage du serveur.

<span id="1-set-up-the-server"></span>

## 1. Configurer le serveur

Sur la machine serveur, suivez les étapes ci-dessous.

1. Démarrez Qdrant pour le serveur.
   [`mash qdrant install`](installation.md#start-the-local-search-service)
   démarre une instance sur `127.0.0.1`, sur les ports 6333 (HTTP) et 6334 (gRPC). Vous pouvez aussi utiliser l’
   [image Docker](local-search.md#optional-qdrant-docker-image) ou un service Qdrant 1.20 ou ultérieur déjà en place. Limitez l’accès aux ports de Qdrant à cette machine.

2. Créez un jeton par ordinateur. Chaque jeton doit contenir entre 32 et 4 096 caractères ASCII imprimables, sans espace. Voici un exemple.

   ```sh
   openssl rand -hex 32
   ```

3. Enregistrez cette configuration dans `server.toml`. Elle répertorie tous les ordinateurs autorisés à se connecter. Elle active uniquement la recherche par mots, ce qui dispense le serveur de télécharger un modèle.

   ```toml
   version = 1
   listen = "127.0.0.1:8787"
   qdrant_url_env = "MASH_QDRANT_URL"
   qdrant_grpc_url_env = "MASH_QDRANT_GRPC_URL"

   [views.build]
   representations = "lexical-only"

   [[nodes]]
   group_id = "personal"
   node_id = "laptop"
   token_env = "MASH_LAPTOP_TOKEN"

   [[nodes]]
   group_id = "personal"
   node_id = "desktop"
   token_env = "MASH_DESKTOP_TOKEN"
   ```

   Le fichier contient uniquement les noms des variables d’environnement. Gardez les jetons en dehors de Git.

4. Définissez ces variables et démarrez le serveur. Les deux variables Qdrant doivent pointer vers la **même** instance Qdrant.

   ```sh
   export MASH_QDRANT_URL=http://127.0.0.1:6333
   export MASH_QDRANT_GRPC_URL=http://127.0.0.1:6334
   export MASH_LAPTOP_TOKEN='the token you made for the laptop'
   export MASH_DESKTOP_TOKEN='the token you made for the desktop'
   mash-server --server-config /absolute/path/to/server.toml
   ```

   Laissez tourner le serveur sous le contrôle de votre gestionnaire de services, avec les mêmes variables.

5. Rendez le serveur accessible. `listen = "127.0.0.1:8787"` autorise uniquement les connexions depuis la machine serveur. Placez un proxy inverse HTTPS devant le serveur ou donnez à
   `listen` comme valeur l’adresse IP de la machine sur le réseau privé et utilisez cette adresse comme URL du serveur.

Cette configuration est destinée à un nouveau serveur. Si vous avez déjà un serveur, sauvegardez-le avant de modifier ses paramètres.

<span id="2-connect-each-computer"></span>

## 2. Connecter chaque ordinateur

Suivez ces étapes sur chaque ordinateur qui doit publier son historique ou effectuer des recherches dans l’historique partagé. Mash doit déjà y être [installé](installation.md) et son instance Qdrant locale doit être en cours d’exécution.

<span id="with-a-join-string"></span>

### Avec une chaîne de connexion

Sur le serveur, enregistrez chaque ordinateur avec un identifiant de nœud unique dans le groupe. Indiquez l’adresse que l’ordinateur utilisera pour joindre le serveur.

```sh
mash-server --server-config /absolute/path/to/server.toml \
  --add-node personal/laptop --api-url https://mash.example.net
```

La commande affiche une valeur `join` qui commence par `mash-join.`. Redémarrez le serveur pour qu’il accepte le nouvel ordinateur, puis [connectez et vérifiez cet ordinateur](#join-and-review-each-computer). Le serveur enregistre une empreinte du secret d’accès dans un fichier distinct, dans le même dossier que sa configuration, par exemple `server.toml.nodes`.
[Remplacer ou révoquer un secret de connexion](#revoke-or-replace-a-computers-credential)

<span id="by-hand"></span>

### Configuration manuelle

Vous pouvez aussi définir vous-même le jeton et les paramètres.

Placez d’abord le jeton de cet ordinateur dans une variable d’environnement, par exemple
`MASH_NODE_TOKEN`. Mash enregistre uniquement le nom de la variable. Définissez-la donc dans chaque environnement où Mash s’exécute, notamment votre shell, le
[processus de surveillance](synchronization.md) et les clients d’agents qui démarrent le serveur MCP de Mash.

Utilisez ensuite l’assistant ou modifiez la configuration.

**Assistant.** Exécutez `mash init --no-index --no-agents` dans un terminal. Si l’assistant s’ouvre sur un écran de récapitulatif, choisissez **Customize setup** (intitulé **Review and edit settings**
dans la version v0.1.1-alpha). Lorsque l’assistant vous demande où effectuer les recherches, choisissez **Connect to an existing shared server**. Saisissez l’URL du serveur, l’identifiant du groupe, l’identifiant de nœud de cet ordinateur et le nom de la variable du jeton. Vérifiez les dossiers d’historique et choisissez **Set up without indexing**. L’étape suivante envoie l’historique au serveur.

**Configuration manuelle.** Modifiez la configuration du nœud créée par `mash init`. Conservez les fournisseurs et les paramètres de l’instance Qdrant locale. Dans la table existante `[index]`, renseignez
`destination`et ajoutez une table `[shared]` comme dans l’exemple suivant.

```toml
[index]
destination = "both"
since_days = 90

[shared]
group_id = "personal"
node_id = "laptop"
api_url = "https://mash.example.net"
token_env = "MASH_NODE_TOKEN"
```

`group_id` et `node_id` doivent correspondre à une même entrée `[[nodes]]` sur le serveur, et la variable doit contenir le jeton de cette entrée. Sur l’ordinateur de bureau, utilisez
`node_id = "desktop"` et le jeton de cet ordinateur.

Les paramètres de recherche doivent être compatibles avec le serveur. Pour le serveur de recherche par mots configuré ci-dessus, chaque ordinateur utilise `[views.build] representations = "lexical-only"` et
`[views.query] intent = "exact-mention"`. Ce sont les paramètres que `mash init` enregistre lorsqu’aucun modèle n’est installé. Pour la recherche sémantique, demandez à l’administrateur du serveur quels paramètres de modèle utiliser.

<span id="3-publish-and-check"></span>

## 3. Publier et vérifier

Sur **chaque** ordinateur, exécutez les commandes suivantes.

```sh
mash index
mash status --shared
```

`mash index` met à jour l’index local, puis publie l’historique qu’il conserve. La période configurée détermine les fichiers d’historique que Mash lit lors de cette exécution. Les conversations plus anciennes déjà présentes dans l’index local peuvent aussi être publiées.
`mash status --shared` indique ce que le serveur a reçu dans la section `shared_index`. Si vous voyez `shared_error`, vérifiez l’adresse du serveur, la connexion et le jeton.

Les conversations d’un ordinateur apparaissent sur les autres une fois qu’il les a publiées. Si la publication échoue, l’historique local est conservé. Corrigez l’erreur signalée, puis relancez `mash index`. Pour continuer à publier les nouvelles conversations, lancez un
[processus de surveillance](synchronization.md) sur chaque ordinateur.

<span id="search"></span>

## Rechercher

Ouvrez `mash dash`, accédez à **chats** et recherchez un détail dont vous vous souvenez sur l’autre ordinateur. Le résultat indique de quelle machine il provient. Pour limiter la recherche à un seul ordinateur, utilisez le filtre par machine et choisissez **Apply filters**.
[Lire une conversation dans le tableau de bord](dashboard.md#find-and-read-a-chat)

Depuis le terminal, utilisez les commandes suivantes.

```sh
mash search 'authentication'            # this computer and the shared index
mash search 'authentication' --shared   # the shared index only, including this computer's published chats
mash search 'authentication' --local    # this computer only; the server is not contacted
```

Ajoutez `--node desktop` pour afficher uniquement les conversations de ce nœud. Les demandes de recherche dans l’historique partagé sont envoyées au serveur, même si l’ordinateur de bureau est en veille. Pour y accéder sur mobile,
[ouvrez le tableau de bord à distance](dashboard.md#open-it-from-your-phone-or-another-computer)
depuis un ordinateur déjà configuré pour effectuer des recherches dans ce groupe.

<span id="remove-a-machine-or-let-a-rebuilt-one-publish-again"></span>

## Retirer une machine ou reprendre la publication après avoir reconstruit son index

Le serveur associe chaque identifiant de nœud au premier index local qui a publié sous cet identifiant. Cela empêche une autre machine d’écraser cet historique en réutilisant le nom. Pour la même raison, si vous supprimez puis reconstruisez l’index d’une machine, la publication échoue avec le message « shared node ownership or revision changed ». Si vous restaurez l’index à partir d’une [sauvegarde](upgrades.md#replace-an-index-that-cannot-be-upgraded) avec
`--keep-identity`, la machine conserve son nom. Sinon, retirez le nom sur le serveur. Pour retirer définitivement un ordinateur, arrêtez la publication sur cet ordinateur et retirez son historique du serveur avant de révoquer son secret d’accès. Si vous désinstallez Mash sur un ordinateur, le serveur conserve sa copie.

Arrêtez `mash-server` et laissez son instance Qdrant tourner. Exécutez la commande ci-dessous avec la même configuration serveur et les mêmes variables d’environnement que le service.

```sh
# On the server machine, with mash-server stopped.
mash-server --server-config /absolute/path/to/server.toml --retire-node personal/laptop
```

**Cette commande supprime définitivement tout ce que cette machine a publié, y compris** ses conversations et ses enregistrements sur le serveur, les données de recherche associées et les pièces jointes partagées. La commande affiche le nombre de conversations et de messages supprimés, puis se termine sans démarrer le serveur. Les autres machines et l’historique local de la machine retirée restent intacts.

Si vous reconstruisez l’index d’un ordinateur, redémarrez `mash-server`, puis exécutez
`mash index` sur cet ordinateur pour publier à partir de son index actuel. Si vous retirez l’ordinateur, révoquez son secret d’accès avant de redémarrer le serveur.

<span id="revoke-or-replace-a-computers-credential"></span>

### Révoquer ou remplacer le secret d’accès d’un ordinateur

Sur le serveur, repérez le bloc `[[nodes]]` qui contient les valeurs `group_id` et
`node_id` de cet ordinateur. Si l’ordinateur a été ajouté avec `--add-node`, le bloc se trouve dans un fichier situé dans le même dossier que la configuration du serveur, généralement `server.toml.nodes`. Si l’ordinateur a été ajouté manuellement, le bloc se trouve dans `server.toml`. Supprimez le bloc et redémarrez le serveur pour révoquer l’accès. Si c’était le dernier ordinateur configuré, laissez le serveur arrêté. Pour conserver les fichiers publiés sur le serveur, omettez la commande de retrait.

Pour remplacer une chaîne de connexion perdue ou divulguée, supprimez l’ancien bloc, ajoutez de nouveau le même groupe et le même nœud, puis redémarrez le serveur. Reconnectez le même ordinateur avec la nouvelle chaîne. Il pourra continuer à publier sous le même nom s’il a conservé son index local.

Avant d’installer une nouvelle version de Mash, consultez le guide des
[mises à niveau et des sauvegardes](upgrades.md#upgrade-a-shared-index).

<!-- mash-docs-source: d53981430b20b1e97415d553067b960e398181e3c769b414ed3965a9f7296bd1 -->
