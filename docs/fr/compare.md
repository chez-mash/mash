# Mash est-il fait pour vous ?

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

[Installez Mash et retrouvez votre première conversation](quick-start.md).

<!-- mash-docs-source: 0eb4bc99fad7c3d96d03dc79b728b346692efb0c9dfc92e68ee20e9ca4c5d290 -->
