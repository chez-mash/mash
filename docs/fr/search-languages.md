# Rechercher dans d’autres langues

Mash conserve les conversations dans leur langue d’origine. Pour retrouver
un message, recherchez un mot tel qu’il apparaît dans la conversation,
accents compris. Dans le [tableau de bord](dashboard.md#find-and-read-a-chat),
ouvrez **chats** et utilisez **Search your history**. La recherche par mots
est le mode par défaut. Pour la sélectionner dans le terminal :

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

Essayez la recherche par mots dans la démo avec des conversations d’exemple en [espagnol](https://getmash.io/demo/#/search?q=autenticaci%C3%B3n) ou en [français](https://getmash.io/demo/#/search?q=corrig%C3%A9).

## Chinois et japonais

Mash traite une suite de caractères sans espace ni ponctuation comme un seul
mot. Si un terme court ne donne aucun résultat, essayez le fragment complet
tel qu’il apparaît dans la conversation. Par exemple :

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

Dans la démo, essayez une suite complète de caractères en [chinois](https://getmash.io/demo/#/search?q=%E6%88%91%E4%BB%AC%E6%9B%B4%E6%96%B0%E4%BA%86%E8%BA%AB%E4%BB%BD%E9%AA%8C%E8%AF%81%E4%BB%A4%E7%89%8C) ou en [japonais](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F).

Si vous vous souvenez d’une partie du **titre**, recherchez ce fragment.
La recherche dans les titres trouve aussi un fragment au milieu d’une suite
de caractères. Le titre commence souvent par les mots du premier message.
[Comment ouvrir un résultat](search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## Recherche par sens

Le [modèle facultatif MiniLM](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
est conçu pour les textes en anglais. Pour la recherche par sens, saisissez
vos questions en anglais. Une question en anglais peut passer à côté d’une
conversation sur le même sujet dans une autre langue. Pour retrouver ce
message, utilisez les mots de la conversation d’origine.

Pour une recherche ponctuelle dans le tableau de bord, choisissez **words**
dans le [menu du mode de recherche](dashboard.md#choose-a-search-mode),
puis sélectionnez **Apply filters**. Si votre tableau de bord ne propose pas
ce menu, utilisez les commandes de recherche par mots ci-dessus.

<!-- mash-docs-source: 5b106b1548f31cf83fd1530542f3bcd86db96597bd5ac3728288286ce9cfddc0 -->
