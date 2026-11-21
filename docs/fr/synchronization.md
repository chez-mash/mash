# Garder l’historique à jour

Une nouvelle conversation devient consultable dès que Mash indexe
l’historique enregistré par votre agent. Utilisez
[Actualiser maintenant](#refresh-now) pour retrouver une conversation récente.
Pour actualiser au fil du travail, [laissez tourner la surveillance](#keep-refreshing-while-you-work).

<span id="refresh-now"></span>

## Actualiser maintenant

Exécutez ces commandes sur l’ordinateur où l’agent a enregistré la conversation.
Si vous n’avez pas encore choisi les historiques que Mash peut lire,
[terminez d’abord la configuration](installation.md#connect-your-histories).

```bash
mash index
mash status
```

`mash index` lit l’historique nouveau ou modifié et affiche sa progression.
Vérifiez le résultat final pour repérer une lecture incomplète ou une
publication en attente. Corrigez le problème indiqué, puis relancez `mash index`.
`mash status` indique si la recherche locale est prête.

La liste des conversations du tableau de bord s’actualise après l’indexation.
Pour actualiser une recherche ouverte, choisissez **Search again**. Si
ce bouton n’est pas affiché, rechargez la page pour relancer la même recherche.

<span id="keep-refreshing-while-you-work"></span>

## Actualiser au fil du travail

Exécutez cette commande dans un autre terminal et laissez-le ouvert :

```bash
mash index --watch
```

La commande lance une première indexation, puis actualise l’index après chaque
série de changements dans l’historique. Elle attend au moins 30 secondes entre
deux passes, et parfois plus après une passe lente. Ctrl-C laisse la passe en cours se
terminer, puis arrête la surveillance. Fermer le terminal l’arrête aussi.

Les agents, la période et la destination locale ou partagée viennent de votre
configuration. Pour inclure un historique plus ancien :

```bash
mash index --watch --all-history
```

Pour modifier définitivement la période, éditez la
[configuration (en anglais)](../configuration.md). `--watch` ne se combine
ni avec `--full` ni avec `--reread`.

Quand vous mettez Mash à jour, la surveillance termine la passe en cours et
lance le nouveau `mash` avec les mêmes options. Le terminal ou le service peut
continuer à surveiller. Si le nouvel exécutable ne se lance pas, la surveillance
s’arrête avec une erreur. Vous pouvez alors la redémarrer.

<span id="what-the-agent-plugin-does-automatically"></span>

## Ce que fait automatiquement le module Mash

Le [module Mash](plugin.md) lance une indexation de faible priorité au début
d’une session, au maximum une fois toutes les dix minutes. L’agent continue
immédiatement. Si une indexation est déjà en cours, la tentative se termine
sans apporter de changement.

Ce rattrapage se fait au démarrage de la session. Pour des mises à jour au cours
d’une session longue, utilisez la surveillance. Pour désactiver le rattrapage,
définissez `MASH_AUTO_INDEX=off` dans l’environnement de l’agent.

<span id="understand-freshness-warnings"></span>

## Comprendre les avertissements de mise à jour

Vous pouvez actualiser manuellement après `mash init` ou un premier `mash index`.
La recherche utilise l’historique de la dernière indexation. Si une ancienne
version affiche « Live indexing is inactive », l’actualisation se fera à la
prochaine indexation.

Une recherche peut encore renvoyer d’anciennes conversations si la dernière
lecture était incomplète ou si Mash n’a pas pu lire son état. Suivez l’avertissement
pour vérifier ou actualiser l’index. S’il manque une conversation récente,
lancez `mash index`, puis relancez la recherche.

- Gardez la surveillance active si vous avez besoin de mises à jour continues
  et vérifiez ses résultats pour savoir si tous les historiques ont été lus.
- Si l’avertissement suit une mise à jour ou indique un index incompatible,
  suivez le [guide de mise à jour](upgrades.md) avant de réessayer.

<span id="in-the-dashboard"></span>

## Dans le tableau de bord

Après l’indexation, le nouvel historique est disponible dans le tableau de bord.
Une conversation ouverte suit les nouveaux messages lorsque vous êtes à la fin.
Ailleurs dans la conversation, choisissez **N new messages** pour les voir.
Un ancien message reste affiché tel quel jusqu’à ce que vous choisissiez
**Show updated messages**. [Comment le tableau de bord s’actualise](dashboard.md#keep-your-place)

<span id="across-machines"></span>

## Sur plusieurs ordinateurs

S’il manque une conversation d’un autre ordinateur, vérifiez d’abord cet
ordinateur. C’est lui qui indexe la conversation et publie la copie partagée.
Lancez `mash status --shared` sur cet ordinateur pour vérifier la publication.
Si elle est en attente, rétablissez la connexion et relancez `mash index` sur
cet ordinateur.

[Configuration pour plusieurs ordinateurs](shared-index.md)

<!-- mash-docs-source: 56a328baf0148ab0de7ea34a1c484a9c9c22f2c01f15fd990a45f1550dda4071 -->
