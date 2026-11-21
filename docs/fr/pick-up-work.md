# Reprendre là où vous en étiez

Ouvrez une ancienne conversation pour retrouver la prochaine étape.

<span id="follow-a-sample-handoff"></span>

## Suivre un exemple de reprise

1. [Ouvrez dans la démo la décision enregistrée concernant la connexion](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop).
2. Choisissez **open the message** pour lire la conversation d’origine dans Claude Code.
   Il reste à stocker le jeton de renouvellement de l’application mobile dans le trousseau de l’appareil.
3. Choisissez **See the Codex follow-up** pour voir comment un autre exemple de conversation reprend ce travail.

<span id="find-the-next-step-in-your-chats"></span>

## Trouver la prochaine étape dans vos conversations

1. Ouvrez **chats** dans le tableau de bord. Recherchez un nom de fichier, une erreur ou une expression liée au travail en cours. Utilisez le filtre par projet si plusieurs projets contiennent des conversations similaires.
2. Ouvrez un résultat et lisez les derniers messages. Repérez le travail terminé, les décisions que vous avez validées et les tests ou tâches encore en suspens. Développez les résultats des outils qui vous intéressent pour voir ce qui s’est passé.
3. Passez vous-même à l’étape suivante ou fournissez un bref récapitulatif dans une nouvelle session avec un agent. Précisez l’objectif, la décision qui s’applique toujours et ce qu’il reste à faire.

Pour retrouver la bonne conversation, consultez [la recherche dans le tableau de bord](dashboard.md#find-and-read-a-chat)
et [les vérifications à faire lorsqu’une conversation est introuvable](troubleshooting.md#search-returns-no-results).

<span id="reopen-the-original-session"></span>

## Rouvrir la session d’origine

Pour l’une de vos conversations Codex ou Claude Code, choisissez **Copy session ID** dans la transcription. Sur l’ordinateur où la session d’origine est enregistrée, ouvrez un terminal dans le dossier du projet et remplacez `SESSION_ID` dans la commande correspondant à votre agent.

```sh
codex resume SESSION_ID
claude --resume SESSION_ID
```

Si l’agent ne parvient pas à ouvrir la session avec cet identifiant, utilisez son sélecteur de sessions pour retrouver la conversation par son titre ou démarrez une nouvelle session avec un récapitulatif.

<span id="ask-an-agent-for-a-recap"></span>

## Demander un récapitulatif à un agent

Une fois le module Mash et les outils de recherche configurés, démarrez une session dans le projet que vous souhaitez reprendre. Exécutez `/mash:resume` dans Claude Code. Dans Codex ou Copilot, demandez « Utilise la compétence `resume` de Mash pour me dire où nous en étions sur ce projet. »
[Configurer Mash dans votre agent](plugin.md#finish-setup-in-the-agent)

La compétence tente d’actualiser l’index, puis vérifie s’il existe une note de passation et lit les conversations récentes du projet. Elle résume ce qui s’est passé, ce qui reste en suspens et la prochaine étape. Demandez-lui d’afficher le message d’origine pour toute décision que vous souhaitez vérifier.

<span id="leave-a-handoff-for-the-next-session"></span>

## Laisser une note de passation pour la prochaine session

1. Avant de terminer une session, exécutez `/mash:handoff` dans Claude Code ou demandez à votre agent d’utiliser la compétence Mash `handoff`. Elle rédige une courte note dans la conversation en cours. Vérifiez que la note précise les fichiers ou la branche concernés, les décisions prises, le travail restant et la prochaine étape.
2. Une fois que l’agent a terminé de rédiger la note, exécutez `mash index` sur l’ordinateur où la conversation est enregistrée. Sur ce même ordinateur, la compétence `resume` peut aussi actualiser l’index lorsque vous reprenez le travail.
3. Ouvrez le projet lors de votre prochaine session et demandez à l’agent d’utiliser `resume`. Vous pouvez aussi rechercher `"Mash handoff"` dans le tableau de bord et lire la note vous-même.

Si vous changez d’ordinateur, configurez d’abord le partage, puis lancez l’indexation sur l’ordinateur où la note a été rédigée pour qu’il puisse publier la conversation. Pour une passation manuelle, copiez la note dans la nouvelle session. Partagez uniquement les informations que vous souhaitez transmettre au prochain agent. [Rechercher sur plusieurs ordinateurs](shared-index.md) · [Ce qu’un agent peut voir](privacy.md#when-history-can-leave-your-machine)

<!-- mash-docs-source: 29c9c1b1f656694eef0aab022b8ba56fec30c9d8b1ebf6737868e0a8c28d226c -->
