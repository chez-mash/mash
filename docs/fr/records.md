# Utiliser la mémoire de Mash

Mash peut garder une décision, une préférence ou un fait avec une citation
de la conversation d’origine. Ouvrez la citation pour retrouver le contexte.

<span id="inspect-existing-records"></span>

## Consulter les souvenirs enregistrés

[Ouvrez une décision enregistrée dans la démo](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
et choisissez **open the message** sous la citation. La démo ouvre le
message sur la conservation du jeton de renouvellement dans le trousseau de l’appareil.

Pour consulter votre propre historique, lancez `mash dash`, ouvrez l’adresse
affichée et choisissez **memory**. Ouvrez un souvenir pour voir sa source,
ses révisions et les éléments de mémoire associés.
[Chercher et filtrer dans le tableau de bord](dashboard.md#inspect-conversation-records)

<span id="save-a-conclusion"></span>

## Enregistrer une conclusion

Avec le [module Mash installé](plugin.md#start-with-the-plugin), demandez
à l’agent de garder une décision dans la conversation où vous l’avez prise.

> Souviens-toi que, dans ce projet, les jetons de renouvellement restent dans le trousseau de l’appareil.

Dans Claude Code, vous pouvez aussi utiliser `/mash:remember`. Dans Codex
ou Copilot, demandez à l’agent d’utiliser la compétence `remember`. Vérifiez
ce qu’il a enregistré dans sa réponse, puis ouvrez **memory** pour lire la
citation dans son contexte.

<span id="recall-records"></span>

## Retrouver des souvenirs

Dans une autre conversation sur le même projet, demandez « Que retient
Mash à propos des jetons de renouvellement ? Montre-moi où nous l’avons
décidé. » Dans Claude Code, vous pouvez aussi utiliser `/mash:recall refresh tokens`.

Le module peut restituer quelques souvenirs à l’agent au démarrage d’une
session ou à la lecture d’un fichier. L’agent peut aussi chercher dans vos
conversations passées les décisions que Mash n’a pas encore enregistrées
en mémoire. [Utiliser Mash depuis un agent](agents.md#try-it)

<span id="how-saving-works"></span>

## Comment Mash enregistre les souvenirs

Mash vérifie la citation dans la conversation d’origine. Vous pouvez ainsi
relire la source. L’agent peut passer à côté d’un élément ou mal comprendre
ce que vous vouliez dire.

- L’agent peut établir une décision ou une préférence d’après vos propres
  mots. D’autres interprétations peuvent rester **à vérifier**.
- L’agent peut enregistrer ses propres choix. Vos décisions ont priorité sur les siennes.
- Si une conversation indexée change et que la citation ne correspond plus,
  Mash signale que la source n’est plus à jour et l’écarte du rappel ordinaire.

L’enregistrement utilise l’agent déjà en cours. Le module peut relever des
conclusions pendant votre travail et rappeler à l’agent d’enregistrer
celles qu’il a manquées. Dans Codex, examinez les modifications des hooks
dans `/hooks` pour les activer.
[Ce que fait le module sans intervention](plugin.md#what-happens-without-asking)

`MASH_RECORD_CAPTURE=off` désactive la capture automatique et laisse le rappel
actif. `MASH_RECORD_RECALL=off` désactive le rappel de la mémoire et conserve
les suggestions de conversations. [Réglages du module](plugin.md#what-happens-without-asking)

<span id="correct-or-replace"></span>

## Corriger ou remplacer un souvenir

Si un souvenir est incorrect ou si vous avez changé d’avis, demandez à
l’agent de le mettre à jour à partir de la conversation où vous l’avez
corrigé. Utilisez la compétence `review`, ou `/mash:review` dans Claude Code,
pour les souvenirs à vérifier, les sources obsolètes et les décisions en
conflit. La consultation laisse l’état d’un souvenir intact.

Vous pouvez demander à l’agent d’oublier un souvenir. Cela supprime ses
révisions et ses citations. Si vous [retirez une conversation citée](privacy.md#remove-a-chat),
Mash retire aussi les souvenirs qui en dépendent.

<span id="backfill-earlier-memory"></span>

## Extraire des souvenirs de conversations plus anciennes

Cet examen facultatif peut proposer des souvenirs à partir de vos anciens
échanges. Il utilise votre session Codex ou Claude Code ouverte et envoie
le texte sélectionné au service de l’agent, en utilisant son budget de jetons.
L’indexation ordinaire et `mash init --yes` laissent cette opération désactivée.

Vous pouvez la demander pendant la configuration interactive pour les chats
actifs au cours des trois derniers jours. Pour examiner plus tard des
conversations déjà indexées, utilisez la commande suivante.

```sh
mash init --yes --no-index --no-agents --backfill-memory codex
```

Remplacez `codex` par `claude-code` si vous le souhaitez. Vérifiez les souvenirs
proposés avant de vous y fier. Mash s’arrête après cent conversations ou
lorsqu’un examen n’aboutit pas. Chaque conversation a une limite de dix
minutes. Relancez la commande pour continuer. Mash passe les conversations
déjà examinées sauf si elles ont changé.

### Souvenirs partagés

Si vous publiez votre historique sur un [serveur Mash partagé](shared-index.md),
les souvenirs publiés sont consultables dans le tableau de bord. Ceux qui sont
liés au chemin d’un projet local restent sur cet ordinateur.
[Lire la source d’un souvenir partagé](dashboard.md#inspect-conversation-records)

## Sauvegarder les souvenirs

La [sauvegarde portable](backup.md#make-a-portable-backup) contient vos souvenirs,
leurs révisions et les citations. Gardez-la aussi privée que les conversations d’origine.

## Appeler les outils de mémoire directement

Pour les arguments et les exemples, consultez les
[outils MCP pour la mémoire (en anglais)](../record-tools.md).

<!-- mash-docs-source: 36885e948f7746e5e39820ec03eac164d61f7874ac71b5ddbaf6c01efb02afdc -->
