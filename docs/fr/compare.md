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
  le message cité dans son contexte. [Comment Mash enregistre les éléments de mémoire (en anglais)](../records.md#how-saving-works)
- **Vous travaillez sur plusieurs ordinateurs.** Chaque ordinateur peut
  interroger son index hors ligne. Vous pouvez aussi publier votre historique
  sur un serveur Mash que vous gérez pour retrouver sur votre portable les
  conversations d’une station de travail. [Comment fonctionne le partage](shared-index.md)

Mash utilise un service de recherche local appelé Qdrant. L’installation
guidée peut le démarrer. La recherche par mots fonctionne sans compte Mash,
sans téléchargement de modèles et sans connecter d’agent. Vous pouvez
ajouter des modèles locaux pour la recherche par sens et connecter un agent
quand vous le souhaitez. [Installation](installation.md) · [Confidentialité](privacy.md) ·
[Recherche par sens (en anglais)](../indexing.md#enable-semantic-search)

<span id="other-workflows"></span>

## Autres façons de travailler

| Si vous voulez… | Essayez | Ce que l’outil propose |
| --- | --- | --- |
| Rechercher et reprendre des sessions ou récupérer une modification de fichier | Déjà Vu | Un seul exécutable indexe l’historique enregistré et permet d’y faire des recherches en local. L’outil transmet le travail à un autre agent et peut récupérer du texte à partir de modifications enregistrées. [README de Déjà Vu](https://github.com/vshulcz/deja-vu#overview) |
| Rechercher au clavier dans les historiques de nombreux agents | CASS | L’outil propose une recherche interactive dans le terminal et des résultats structurés pour les scripts ou les agents. [Dépôt CASS](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| Réutiliser des résumés automatiques du travail dans les nouvelles sessions | Claude-Mem | Les intégrations capturent l’activité et un agent observateur la résume. La prochaine session peut charger les notes pertinentes. [Fonctionnement de Claude-Mem](https://github.com/thedotmack/claude-mem#how-it-works) |
| Enregistrer les conversations en Markdown dans votre projet ou les reprendre dans un agent | SpecStory | Ses extensions et son interface en ligne de commande écrivent des fichiers Markdown au fil du travail. La ligne de commande permet de rouvrir les sessions enregistrées. [Enregistrer les conversations](https://docs.specstory.com/specstory/features) · [Rechercher et reprendre une session](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[Installez Mash et retrouvez votre première conversation](quick-start.md).

<!-- mash-docs-source: 93cb840d39d49b21496d27385d3b09d3c74fe1f6c462801f5a43717fbbfbc6c9 -->
