# Buscar en otros idiomas

Mash conserva las conversaciones en su idioma original. Para encontrar un
mensaje, busca una palabra tal como aparece en la conversación, con sus
tildes. En el [panel web](../dashboard.md#find-and-read-a-chat), abre
**chats** y usa **Search your history**. La búsqueda por palabras está
activada de forma predeterminada. Desde el terminal puedes seleccionarla así:

```sh
mash search 'autenticación' --intent exact-mention
mash search 'corrigé' --intent exact-mention
```

## Chino y japonés

Mash trata como una sola palabra una secuencia de caracteres sin espacios
ni signos de puntuación. Si una búsqueda corta no da resultados, prueba
con el fragmento completo de la conversación. Por ejemplo:

```sh
mash search '我们更新了身份验证令牌' --intent exact-mention
mash search '認証トークンの更新手順を見直した' --intent exact-mention
```

Si recuerdas parte del **título**, busca ese fragmento directamente. Esta
búsqueda puede encontrar texto en medio de una secuencia de caracteres.
El título suele empezar con el primer mensaje.
[Cómo abrir un resultado (en inglés)](../search.md#find-a-chat-by-title)

```sh
mash list-chats --title-contains '身份验证'
```

## Búsqueda por significado

El [modelo opcional MiniLM](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
está diseñado para texto en inglés. Escribe en inglés las preguntas que
quieras buscar por significado. Una pregunta en inglés puede pasar por alto
una conversación sobre el mismo tema en otro idioma. Para encontrar ese
mensaje, usa las palabras de la conversación original.

Si configuraste el panel para buscar por significado, usa los comandos de
búsqueda por palabras de arriba para una consulta puntual o
[vuelve a establecer la búsqueda por palabras como predeterminada (en inglés)](../configuration.md#search-defaults).

<!-- mash-docs-source: 7fa76bcd3e4fa9d45ebe6ea8df3575b59a0c645f3d00f20a143478985adc5a58 -->
