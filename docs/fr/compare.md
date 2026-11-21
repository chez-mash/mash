# Comparez les outils d’historique pour agents de code

Mash vous aide à retrouver les conversations que vos agents ont déjà
enregistrées et à ouvrir le message à l’origine d’une décision. Vous pouvez
effectuer la recherche dans un navigateur, dans le terminal ou depuis un agent
connecté.

Dans la démo, [recherchez les conversations d’exemple sur l’export des factures](https://getmash.io/demo/#/search?q=invoice+exports+queue)
et ouvrez le résultat de la station de travail pour lire la discussion.
Ouvrez ensuite la [décision enregistrée sur la connexion](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
et choisissez **open the message** pour voir d’où elle vient.

<span id="when-mash-fits"></span>

## Quand choisir Mash

- **Vous avez déjà des conversations que vous souhaitez conserver.** Mash lit
  les conversations enregistrées sur disque par les agents pris en charge.
  Recherchez un mot ou une expression et ouvrez le résultat dans sa conversation
  pour lire le dialogue et les sorties d’outils. Utilisez le tableau de bord,
  le terminal ou un agent que vous choisissez de connecter.
  [Agents pris en charge](supported-agents.md)
- **Vous voulez vérifier sur quoi repose une décision enregistrée.** Avec le
  module Mash, votre agent peut enregistrer des décisions et des préférences
  avec une citation vérifiée dans la conversation. Le tableau de bord ouvre
  le message cité dans son contexte. [Comment Mash enregistre les éléments de mémoire](records.md#how-saving-works)
- **Vous travaillez sur plusieurs ordinateurs.** Chaque ordinateur peut
  interroger son index hors ligne. Vous pouvez aussi publier votre historique
  sur un serveur Mash que vous gérez pour retrouver sur votre portable les
  conversations d’une station de travail. [Comment fonctionne le partage](shared-index.md)

Mash utilise un service de recherche local appelé Qdrant. L’installation
guidée peut le démarrer. La recherche par mots fonctionne sans compte Mash,
sans téléchargement de modèles et sans connecter d’agent. Vous pouvez
ajouter des modèles locaux pour la recherche par sens et connecter un agent
quand vous le souhaitez. [Installation](installation.md) · [Confidentialité](privacy.md) ·
[Recherche par sens](indexing.md#enable-semantic-search)

<span id="other-workflows"></span>

## Autres façons de travailler

| Si vous voulez… | Essayez | Ce que l’outil propose |
| --- | --- | --- |
| Rechercher et reprendre des sessions ou récupérer une modification de fichier | Déjà Vu | Un seul exécutable indexe l’historique enregistré et permet d’y faire des recherches en local. Reprenez une session dans son agent d’origine, transmettez le travail à un autre agent ou récupérez du texte à partir de modifications enregistrées. [Vue d’ensemble](https://github.com/vshulcz/deja-vu#overview) · [Recherche interactive](https://github.com/vshulcz/deja-vu#interactive-search) |
| Rechercher au clavier dans les historiques de nombreux agents | CASS | L’outil propose une recherche interactive dans le terminal et des résultats structurés pour les scripts ou les agents. [Dépôt CASS](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| Retrouver la session d’agent à l’origine d’une ligne de code ou d’un commit | ctx | L’outil indexe l’historique enregistré en local pour y faire des recherches. `ctx blame` s’appuie sur l’historique Git et l’activité enregistrée de l’agent pour citer les éléments de la session. [Recherche](https://ctx.rs/search/) · [Blame](https://ctx.rs/blame/) |
| Réutiliser des résumés automatiques du travail dans les nouvelles sessions | Claude-Mem | Les intégrations capturent l’activité et un agent observateur la résume. La prochaine session peut charger les notes pertinentes. [Fonctionnement de Claude-Mem](https://github.com/thedotmack/claude-mem#how-it-works) |
| Enregistrer les conversations en Markdown dans votre projet ou les reprendre dans un agent | SpecStory | Ses extensions et son interface en ligne de commande peuvent enregistrer les sessions capturées en Markdown dans le projet. La ligne de commande permet de rouvrir une session dans son agent d’origine ou de la reconstituer pour un autre agent compatible. [Enregistrer les conversations](https://docs.specstory.com/specstory/features) · [Rechercher et reprendre une session](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[Installez Mash et retrouvez votre première conversation](quick-start.md).

<!-- mash-docs-source: e46b86ce4739cba4e7caa3de5458a8a8928d9850fd953a45ddffc1f1d8788368 -->
