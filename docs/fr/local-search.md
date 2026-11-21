# Service de recherche local (Qdrant)

Mash utilise Qdrant pour indexer vos conversations et y chercher sur cet
ordinateur. L’installateur de Mash lance le service à l’ouverture de votre
session. Vous pouvez aussi utiliser votre propre Qdrant. Une fois le
service démarré, [configurez vos historiques](installation.md#connect-your-histories).

<span id="let-mash-manage-qdrant"></span>

## Laisser Mash gérer Qdrant

```sh
mash qdrant install
```

Cette commande télécharge [Qdrant 1.20.0 pour votre système](https://github.com/qdrant/qdrant/releases/tag/v1.20.0)
(environ 30 Mo), compare le fichier à la somme SHA-256 fournie avec Mash et
lance le service en arrière-plan sous votre compte. La commande fonctionne
sans Docker ni droits administrateur. Le service écoute uniquement sur cet
ordinateur (`127.0.0.1`, ports 6333 et 6334) et redémarre à l’ouverture de
votre session. Pour une nouvelle installation, le résultat JSON contient
`"outcome": "installed"`.

<span id="where-qdrant-stores-data"></span>

### Où Qdrant conserve les données

Les chemins ci-dessous concernent le service installé par Mash. Il conserve
le texte des conversations accessible à la recherche et, si vous cherchez
par le sens, les vecteurs que ce mode utilise.

| Système | Mode d’exécution | Journaux de Qdrant | Données |
| --- | --- | --- | --- |
| Linux | Service utilisateur systemd `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant/storage` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant/storage` |
| Windows | Tâche à l’ouverture de session `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `%SystemDrive%\mash-qdrant\storage` |

Qdrant utilise aussi un dossier `snapshots` à côté de `storage`. Sous
Windows, ces dossiers se trouvent généralement sous `C:\mash-qdrant` pour
éviter les échecs dus à un chemin trop long. Le programme, les réglages et
les journaux de Qdrant restent sous `%USERPROFILE%\.mash\qdrant`.

`mash status` affiche le chemin, la version et l’état du service sous
`local_qdrant.installed`. Le résultat de `mash qdrant install` indique le
chemin sous `service.storage`. [Où Mash conserve ses autres fichiers](privacy.md#where-mash-stores-data)

<span id="check-or-restart-the-service"></span>

### Vérifier ou redémarrer le service

En cas d’échec au démarrage, consultez [la commande ou le fichier de journalisation de votre système](#where-qdrant-stores-data).

Sous Windows, la tâche s’exécute pendant qu’une session de bureau est
ouverte. Si vous accédez à l’ordinateur uniquement par SSH ou si personne
n’ouvre de session sur le bureau, la commande indique que Qdrant n’a pas
démarré à temps. Dans ce cas, [démarrez Qdrant directement sous Windows](#native-windows-without-docker).

Vous pouvez relancer `mash qdrant install`. La commande redémarre un
service arrêté, restaure les fichiers manquants et met Qdrant à jour
quand une nouvelle version de Mash fournit une version plus récente.
Les données de Qdrant sont conservées. Le résultat est `current` si le
service géré est déjà en cours d’exécution et à jour. Si un autre Qdrant compatible répond sur ces
ports, par exemple dans le conteneur Docker, le résultat est
`already_running` et Mash le réutilise. Si un autre programme occupe un
port, la commande s’arrête et indique le port concerné.

- **Linux.** La commande nécessite une [session utilisateur systemd](#linux-without-a-systemd-user-session).
  Le service tourne pendant que vous êtes connecté. Sur un ordinateur
  sur lequel personne n’ouvre de session, vous pouvez le maintenir avec
  `loginctl enable-linger`. Sur certains systèmes, seuls les administrateurs
  peuvent le faire.
- **Autres ports.** Avant l’installation, réglez `url` et `grpc_url` sur des
  adresses `127.0.0.1` dans la [configuration de Qdrant (en anglais)](../configuration.md#local-qdrant).

`mash init` vérifie que Qdrant répond. Utilisez `mash qdrant install` ou
démarrez Qdrant vous-même pour le mettre en service.

<span id="linux-without-a-systemd-user-session"></span>

### Linux sans session utilisateur systemd

L’installateur de Mash s’appuie sur votre session utilisateur Linux pour
garder Qdrant en arrière-plan. Lancez cette vérification en lecture seule
dans le terminal Linux ou WSL qui servira à configurer Mash.

```sh
systemctl --user show-environment >/dev/null
```

Si la commande se termine sans erreur, utilisez `mash qdrant install`.
Si elle signale un problème de connexion au bus utilisateur ou si
`systemctl` est introuvable, [démarrez Qdrant dans Docker](#optional-qdrant-docker-image)
ou [lancez Qdrant directement dans un terminal Linux](#native-macos-or-linux-without-docker).
Si vous préférez activer systemd sous WSL 2, suivez le
[guide de Microsoft](https://learn.microsoft.com/fr-fr/windows/wsl/systemd),
puis relancez la vérification dans votre terminal WSL.

Si l’installation rapide s’est arrêtée après avoir affiché « Installed »,
les exécutables de Mash sont déjà en place. Démarrez Qdrant, puis lancez
`mash dash` pour la configuration dans le navigateur ou `mash init` pour la
configuration dans le terminal. Utilisez le chemin de l’exécutable si nécessaire.
Pour une nouvelle installation, utilisez
[l’installateur des exécutables](installation.md#recommended-release-installers)
avant de démarrer Qdrant vous-même.

<span id="run-qdrant-yourself"></span>

## Exécuter Qdrant vous-même

Utilisez [l’image Docker officielle](#optional-qdrant-docker-image), lancez
Qdrant directement comme décrit ci-dessous ou réutilisez un service
Qdrant 1.20 ou ultérieur. Les ports HTTP et gRPC doivent aboutir à la même
instance. Si vous changez les adresses, voyez la
[configuration de Qdrant (en anglais)](../configuration.md#local-qdrant).
Mash et le tableau de bord continuent de fonctionner sur votre ordinateur.

Chaque instance de Qdrant a son propre espace de stockage. Pour passer
d’une instance native à Docker ou à `mash qdrant install`, ou inversement,
arrêtez d’abord l’ancien service pour libérer les ports 6333 et 6334.
Remplissez ensuite le nouveau avec `mash index --rebuild-local-views --full`.

<span id="native-windows-without-docker"></span>

### Windows sans Docker

Sous Windows x64, téléchargez [Qdrant 1.20.0 pour Windows](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip).
Extrayez-le dans `C:\mash-qdrant` ou dans un autre dossier au chemin court
où vous pouvez écrire. Un chemin long sous votre profil peut faire échouer
la création des index de Qdrant même si le contrôle au démarrage réussit.
Exécutez les commandes suivantes dans PowerShell.

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

Gardez cette fenêtre ouverte. Réutilisez les mêmes réglages la prochaine
fois pour conserver vos données. Dans un deuxième terminal, vérifiez que
Qdrant est prêt avec `curl.exe --fail http://127.0.0.1:6333/readyz` avant
de lancer `mash init`.

<span id="native-macos-or-linux-without-docker"></span>

### macOS ou Linux sans Docker

Téléchargez l’archive pour votre ordinateur depuis la
[version 1.20.0 de Qdrant](https://github.com/qdrant/qdrant/releases/tag/v1.20.0).

| Système | Archive |
| --- | --- |
| macOS, puce Apple | [Télécharger](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| macOS, Intel | [Télécharger](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux, x86_64 | [Télécharger](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux, ARM64 | [Télécharger](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

Extrayez l’archive dans un dossier dédié, par exemple `~/qdrant`, puis
démarrez Qdrant depuis ce dossier.

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

Gardez le terminal ouvert. La prochaine fois, lancez Qdrant depuis le
même dossier pour réutiliser les données. Dans un deuxième terminal,
vérifiez le démarrage avec `curl --fail http://127.0.0.1:6333/readyz`
avant de lancer `mash init`.

<span id="optional-qdrant-docker-image"></span>

### Qdrant dans Docker

Installez et lancez [Docker Desktop (en anglais)](https://www.docker.com/products/docker-desktop/)
sous macOS ou Windows, ou [Docker Engine (en anglais)](https://docs.docker.com/engine/install/)
sous Linux. Sous Windows, l’image nécessite des conteneurs Linux. Si votre
installation de Windows Server ou de Docker Engine ne les prend pas en
charge, [démarrez Qdrant directement sous Windows](#native-windows-without-docker).
Une fois Docker démarré, lancez la commande dans Bash ou PowerShell.

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

Le service est accessible uniquement depuis cet ordinateur. Ouvrez
[la page de vérification du service](http://127.0.0.1:6333/readyz) et
attendez qu’elle affiche « all shards are ready » avant de lancer `mash init`.

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

Qdrant redémarre avec Docker sauf si vous l’avez arrêté vous-même. Si le
conteneur existe déjà, relancez-le avec `docker start`. L’arrêt du conteneur
conserve les données dans le volume `mash-qdrant-data`. Supprimez ce volume
uniquement si vous souhaitez effacer ces données.

<!-- mash-docs-source: 5506852e48e6f2fe5fb713326fe5b9ea939c430cd352dd42c470fb85bf1aab9f -->
