# Rechercher dans d’autres langues

Mash conserve les conversations dans leur langue d’origine. Pour retrouver
un message, recherchez un mot tel qu’il apparaît dans la conversation,
accents compris. Dans le [tableau de bord](../dashboard.md#find-and-read-a-chat),
ouvrez **chats** et utilisez **Search your history**. La recherche par mots
est le mode par défaut. Pour la sélectionner dans le terminal :

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

## Chinois et japonais

Mash traite une suite de caractères sans espace ni ponctuation comme un seul
mot. Si un terme court ne donne aucun résultat, essayez le fragment complet
tel qu’il apparaît dans la conversation. Par exemple :

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

Si vous vous souvenez d’une partie du **titre**, recherchez ce fragment.
La recherche dans les titres trouve aussi un fragment au milieu d’une suite
de caractères. Le titre commence souvent par les mots du premier message.
[Comment ouvrir un résultat (en anglais)](../search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## Recherche par sens

Le [modèle facultatif MiniLM](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
est conçu pour les textes en anglais. Pour la recherche par sens, saisissez
vos questions en anglais. Une question en anglais peut passer à côté d’une
conversation sur le même sujet dans une autre langue. Pour retrouver ce
message, utilisez les mots de la conversation d’origine.

Si le tableau de bord utilise la recherche par sens, lancez une des commandes
de recherche par mots ci-dessus pour une recherche ponctuelle, ou
[rétablissez la recherche par mots par défaut (en anglais)](../configuration.md#search-defaults).

<!-- mash-docs-source: 7fa76bcd3e4fa9d45ebe6ea8df3575b59a0c645f3d00f20a143478985adc5a58 -->
