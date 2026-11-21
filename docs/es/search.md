# Busca conversaciones con Mash

Mash busca en las [conversaciones guardadas por tus agentes de programación](supported-agents.md) de este equipo y de los que conectes mediante un [índice compartido](shared-index.md). La búsqueda cubre los mensajes que escribiste tú o tu agente. Abre la conversación para leer las llamadas a herramientas y sus resultados.

Usa el [panel para encontrar y leer una conversación](dashboard.md#find-and-read-a-chat) o [pídeselo a un agente conectado](agents.md#try-it). Los ejemplos de esta guía usan la terminal.

<span id="your-first-search"></span>

## Tu primera búsqueda

Después de [configurar la búsqueda local](installation.md#connect-your-histories), prueba con una palabra distintiva, un nombre de archivo o un identificador de una conversación. Este ejemplo muestra hasta tres resultados.

```bash
mash search 'authentication token' --limit 3
```

La salida JSON agrupa los resultados por conversación, con la mejor coincidencia primero. Busca el campo `title` de la conversación y el campo `snippet` del mensaje que coincide. Copia el campo `lookup.view.id` de ese mensaje y úsalo en lugar de `MESSAGE_ID` para leer el mensaje en contexto.

```bash
mash view MESSAGE_ID --context 3
```

<details>
<summary>Ejemplo de resultado</summary>

```json
{
  "results": [
    {
      "chat_id": "01a1a77a-8105-7cf1-849b-37524f39e223",
      "provider": "codex",
      "title": "Add token authentication to the dashboard",
      "project": "/home/you/project",
      "matches": [
        {
          "lookup": { "view": { "id": "709e029507a46e56c1e1cf40" } },
          "role": "assistant",
          "snippet": "It's an authentication gate: the dashboard asks for a token…"
        }
      ]
    }
  ],
  "next_offset": null
}
```

</details>

La búsqueda consulta el índice existente. Para encontrar una conversación que acabas de tener, ejecuta `mash index` primero.

<span id="read-the-original-messages"></span>

## Lee los mensajes originales

`MESSAGE_ID` es un valor de `lookup.view.id` de un resultado de búsqueda o de
`list-chats`. Con `--context 3` se incluyen hasta tres mensajes anteriores y tres
posteriores. Si el identificador no corresponde a ningún mensaje, obtendrás el
error `not-found`. Si das un identificador de conversación, el error te indicará
el identificador del primer mensaje que puedes usar.

Para el historial local, la salida incluye el campo `chat_id` del mensaje, el campo `message_count` de la conversación, los `messages` y cuántos quedan antes y después (`omitted_before`, `omitted_after`). Si quedan mensajes fuera, lo indica un campo `notice`. Aumenta `--context` para leer más.

Los resultados de otro equipo también incluyen `lookup.view.shared`, el identificador del equipo que publicó el historial. Pásalo con `--shared`.

```bash
mash view MESSAGE_ID --shared NODE_ID --context 3
```

<span id="search-in-other-languages"></span>

## Busca en otros idiomas

Para buscar por palabras, usa la misma ortografía que en la conversación original. Para ver ejemplos y consejos para chino o japonés, consulta [cómo buscar conversaciones en otros idiomas](search-languages.md). La búsqueda por significado usa modelos para inglés y funciona mejor con conversaciones en ese idioma.

<span id="find-a-chat-by-title"></span>

## Encuentra una conversación por el título

```bash
mash list-chats --title-contains 'authentication' --limit 10
```

Este comando lista conversaciones sin buscar en sus mensajes. Funciona sin modelos y aunque el índice de búsqueda no esté disponible. El título se toma del primer mensaje o del nombre que tenga la sesión de Claude Code. Si cambias ese nombre, aparecerá al volver a indexar la sesión.

Cada conversación incluye el campo `lookup.view.id` de su primer mensaje. Puedes usarlo para empezar a leer desde el principio.

```bash
mash view FIRST_MESSAGE_ID --context 10
```

Para buscar dentro de una conversación, pasa su campo `chat_id` y pide resultados por mensaje.

```bash
mash search 'token' --chat CHAT_ID --unit messages
```

<span id="narrow-the-results"></span>

## Acota los resultados

| Recuerdo…               | Ejemplo                                                                       |
| ----------------------- | ----------------------------------------------------------------------------- |
| El agente               | `mash search 'authentication' --provider codex`                               |
| Más o menos cuándo fue  | `mash search 'authentication' --days 7`                                       |
| Un intervalo de fechas  | `mash search 'authentication' --after 2026-10-01 --before 2026-11-01`         |
| La carpeta del proyecto | `mash search 'authentication' --project /absolute/path/to/project`            |
| El repositorio          | `mash search 'authentication' --repository https://github.com/owner/repo.git` |
| Algo que escribí        | `mash search 'authentication' --role user`                                    |
| La ortografía exacta    | `mash search 'auth_token' --match-mode strict`                                |
| La frase exacta         | `mash search '"rotate the logs"'`                                             |

Los filtros se combinan y cada resultado debe cumplirlos todos. Para aceptar varias opciones de un filtro, repite la misma opción, como en `--provider codex --provider claude-code`.

`--project` y `--repository` deben coincidir exactamente con el valor registrado. Copia ese valor del campo `project` o `repository` de un resultado de `list-chats` o de búsqueda.

`--after` y `--before` aceptan una fecha (a medianoche UTC) o una marca de tiempo RFC 3339. `--after` incluye ese momento y `--before` lo excluye. En `list-chats`, estas opciones reciben segundos Unix y filtran por la actividad de la conversación.

<span id="search-by-meaning"></span>

## Busca por significado

Por defecto, Mash busca por palabras sin usar modelos. Pasa `--intent exact-mention` para elegir este modo si tu configuración tiene otro [modo predeterminado (en inglés)](../configuration.md#search-defaults).

Cuando hayas [activado la búsqueda por significado (guía en inglés)](../indexing.md#enable-semantic-search), describe lo que recuerdas con tus propias palabras.

```bash
mash search 'why did we choose a separate database' --intent related-discussion
```

Por ahora, escribe en inglés las consultas por significado.

Elige el modo según lo que recuerdes.

|                  | `exact-mention`                                          | `related-discussion`                                           |
| ---------------- | -------------------------------------------------------- | -------------------------------------------------------------- |
| Sirve para       | Palabras, identificadores, nombres de archivos y errores | Una pregunta o una idea que puede estar escrita de otra manera |
| Necesita modelos | No                                                       | Sí                                                             |
| Varias palabras  | Deben aparecer todas en el mensaje, en cualquier orden   | Pueden no aparecer en el mensaje                               |

La búsqueda por palabras busca los términos escritos. Por ejemplo, “log rotation” no encontrará “rotate the application logs”. Prueba “rotate” o busca por significado. Un término como `auth_token` o `voyage-code-4` se busca tal como está escrito, como una sola palabra. Por defecto se aceptan prefijos y pequeños errores de escritura. `--match-mode strict` exige la ortografía exacta.

<span id="search-for-an-exact-phrase"></span>

## Busca una frase exacta

Pon la consulta entre comillas dobles, dentro de las comillas del intérprete de comandos.

```bash
mash search '"rotate the logs"'
```

Una consulta entre comillas busca la frase completa con esa ortografía. Si no aparece ningún resultado, quita las comillas para buscar las palabras por separado.

<span id="more-results-and-local-only-searches"></span>

## Más resultados y búsquedas solo en este equipo

Cada página tiene 20 resultados salvo que elijas otro límite con `--limit`. Para ver la página siguiente, repite el comando con `--offset` y el valor que devuelve `next_offset`. Si ese campo es nulo, ya no hay más páginas.

`--sort recent` coloca primero los resultados más recientes dentro de cada página de coincidencias. Para listar las conversaciones en orden cronológico, usa `list-chats`.

Con un [índice compartido](shared-index.md), la búsqueda combina el historial local y el publicado por otros equipos. `--local` busca en este equipo y `--shared` busca solo en el historial publicado. `--node NODE_ID` filtra por el equipo que registró el historial, sin conectarse a él.

Si faltan resultados o están desactualizados, consulta la [guía de solución de problemas](troubleshooting.md). Para otras operaciones, consulta los [comandos útiles (en inglés)](../cli.md).

<!-- mash-docs-source: fe6be95f1cae06f55d2ce70951ed7c053b4a2ed4ad7f911f5d36d5fc9a56e42e -->
