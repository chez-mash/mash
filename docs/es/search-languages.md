# Buscar en otros idiomas

Mash conserva las conversaciones en su idioma original. Para encontrar un
mensaje, busca una palabra tal como aparece en la conversación, con sus
tildes. En el [panel web](dashboard.md#find-and-read-a-chat), abre
**chats** y usa **Search your history**. La búsqueda por palabras está
activada de forma predeterminada. Desde el terminal puedes seleccionarla así:

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

Prueba la búsqueda por palabras en la demo con conversaciones de ejemplo en [español](https://getmash.io/demo/#/search?q=autenticaci%C3%B3n) o [francés](https://getmash.io/demo/#/search?q=corrig%C3%A9).

## Chino y japonés

Mash trata como una sola palabra una secuencia de caracteres sin espacios
ni signos de puntuación. Si una búsqueda corta no da resultados, prueba
con el fragmento completo de la conversación. Por ejemplo:

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

Prueba un fragmento completo en [chino](https://getmash.io/demo/#/search?q=%E6%88%91%E4%BB%AC%E6%9B%B4%E6%96%B0%E4%BA%86%E8%BA%AB%E4%BB%BD%E9%AA%8C%E8%AF%81%E4%BB%A4%E7%89%8C) o [japonés](https://getmash.io/demo/#/search?q=%E8%AA%8D%E8%A8%BC%E3%83%88%E3%83%BC%E3%82%AF%E3%83%B3%E3%81%AE%E6%9B%B4%E6%96%B0%E6%89%8B%E9%A0%86%E3%82%92%E8%A6%8B%E7%9B%B4%E3%81%97%E3%81%9F) en la demo.

Si recuerdas parte del **título**, busca ese fragmento directamente. Esta
búsqueda puede encontrar texto en medio de una secuencia de caracteres.
El título suele empezar con el primer mensaje.
[Cómo abrir un resultado](search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## Búsqueda por significado

El [modelo opcional MiniLM](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
está diseñado para texto en inglés. Escribe en inglés las preguntas que
quieras buscar por significado. Una pregunta en inglés puede pasar por alto
una conversación sobre el mismo tema en otro idioma. Para encontrar ese
mensaje, usa las palabras de la conversación original.

Para una sola consulta en el panel, elige **words** en el
[menú del modo de búsqueda](dashboard.md#choose-a-search-mode)
y selecciona **Apply filters**. Si tu panel no muestra ese menú, usa los
comandos de búsqueda por palabras de arriba.

<!-- mash-docs-source: 5b106b1548f31cf83fd1530542f3bcd86db96597bd5ac3728288286ce9cfddc0 -->
