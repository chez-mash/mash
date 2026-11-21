# Privacidad del historial

<span id="what-stays-on-your-machine"></span>

## Qué se queda en tu equipo

Con la configuración predeterminada, Mash lee e indexa el historial y ejecuta
las búsquedas en tu equipo. No modifica el contenido de los archivos de
historial de tus agentes. No necesitas una cuenta de Mash y el sitio público
de documentación no recibe tus conversaciones indexadas. La
[demo pública](https://getmash.io/demo/) usa conversaciones de ejemplo y
no accede a tu historial local.

Con la configuración predeterminada, los programas que se ejecutan en el
mismo equipo pueden acceder al panel y al servicio de búsqueda instalado por
Mash sin iniciar sesión. Usa esta configuración en un equipo cuyos usuarios
sean de confianza. Para abrir el panel desde otro dispositivo, usa el
[panel con inicio de sesión](dashboard.md#open-it-from-your-phone-or-another-computer).

Cuando Mash lee una base de datos SQLite de un agente, como OpenCode, SQLite
puede crear archivos `-wal` y `-shm` junto a la base. Esto también ocurre con
otros lectores de SQLite. El contenido original se conserva.

<span id="choose-agent-access"></span>

## Elige a qué pueden acceder tus agentes

Puedes buscar tus conversaciones desde el panel local o la terminal sin conectar
un agente. Elige por separado qué historiales lee Mash y qué agentes conectas.

- **Durante la configuración**, deja todos los agentes sin marcar para omitir
  las nuevas conexiones. Las herramientas y los complementos ya instalados
  se conservan.
  [Configura Mash sin conectar agentes](installation.md#skip-agent-connections).
- **Para quitar una integración existente**, retira las herramientas de Mash,
  desactiva o desinstala su complemento y reinicia el agente. Hazlo también si
  instalaste el complemento por separado.
  [Quita la conexión de un agente](agents.md#disconnect-an-agent).
- **Para dejar de indexar un historial**, desactívalo en **agent histories** de
  **settings**, guarda los cambios y reinicia el proceso de indexación continua
  si está en marcha. Las conexiones existentes siguen activas y las conversaciones
  ya indexadas siguen disponibles en la búsqueda.
  [Quita una conversación de Mash](#remove-a-chat).

Al conectar un agente desde la configuración, Mash registra sus herramientas e
intenta instalar su complemento.
[Qué instala la conexión](agents.md#what-the-connection-installs).

<span id="when-history-can-leave-your-machine"></span>

## Cuándo puede salir parte del historial

| Cómo usas Mash | Qué se puede leer o enviar |
| --- | --- |
| Búsqueda desde el panel local o la terminal | Mash consulta el índice de tu equipo y te muestra los resultados. |
| Herramientas conectadas al agente (MCP) | El agente puede buscar en las conversaciones indexadas y leer los mensajes que coincidan junto con su contexto. Puede enviar lo que lea a su proveedor de modelos. |
| Complemento de Mash con acciones automáticas activadas | Al iniciar una sesión, el complemento puede aportar detalles de conversaciones recientes y recuerdos guardados. También puede añadir extractos cuando el agente lee un archivo relacionado. Esos datos pueden llegar al proveedor de modelos aunque no haya una conexión MCP. [Qué hace automáticamente y cómo desactivarlo](plugin.md#what-happens-without-asking). |
| Historial compartido | Mash envía las conversaciones indexadas y los adjuntos conservados a tu servidor. Cualquier equipo con un token para el mismo grupo puede leerlos. Para detener los envíos, elige **this machine only** en **index into** de **settings**, guarda y reinicia el proceso de indexación continua si está en marcha. Las copias que ya están en el servidor se conservan. [Revisa qué se compartirá](shared-index.md#before-sharing). |
| Mashy en el panel | Si lo activas, Mashy envía tus preguntas, los fragmentos que selecciones y el texto y los recuerdos que consulta al [servicio de modelos que configures (en inglés)](../configuration.md#the-dashboard-assistant). Para detener los envíos, desactiva **Mashy** en **settings** y guarda, o elimina la sección `[assistant]` del archivo de configuración. |

- **Servicio remoto para calcular representaciones semánticas.** Si lo
  configuras, recibe el texto preparado para la búsqueda, que puede contener
  datos sensibles. Por defecto, Mash lo calcula en tu equipo. Las representaciones
  de las consultas siempre se calculan en local.
  [Cálculo de representaciones semánticas (en inglés)](../configuration.md#embedding-inference).
- **Servicio remoto para reordenar resultados.** Si lo activas, recibe cada
  consulta y los fragmentos que va a ordenar. Por defecto, se ordenan en tu equipo.
  [Reordenamiento remoto (en inglés)](../configuration.md#remote-reranking).
- **Recuperar recuerdos de chats anteriores.** Si lo autorizas durante la
  configuración inicial, los chats seleccionados se envían al proveedor del
  agente que elijas entre aquellos en los que hayas iniciado sesión.
  [Cómo se recuperan recuerdos](records.md#backfill-earlier-memory).

<span id="where-mash-stores-data"></span>

## Dónde guarda Mash los datos

Por defecto, Mash guarda las conversaciones indexadas en
`~/.mash/mash.sqlite3` y la configuración en `~/.mash/config.toml`. En Windows,
`~` es tu perfil de usuario. Qdrant guarda aparte los datos de búsqueda,
incluido el texto en el que se puede buscar. En Windows, Mash usa una ruta
corta en la unidad del sistema, normalmente `C:\mash-qdrant\storage`.
[Archivos del servicio de búsqueda por plataforma](local-search.md#where-qdrant-stores-data)

Ejecuta `mash status` para consultar `database_path` y, si Mash instaló Qdrant,
`local_qdrant.installed.storage`. Usa `mash uninstall` para ver qué se
eliminaría, incluida la carpeta separada de Qdrant en Windows. Añade `--yes`
cuando quieras desinstalar Mash. [Qué se elimina al desinstalar](uninstall.md)

<span id="try-a-small-local-setup"></span>

## Prueba con menos historial

[Prueba una búsqueda con conversaciones de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue).
Para probar Mash con uno de tus historiales en una instalación nueva:

1. Ejecuta el [instalador de la guía de inicio](quick-start.md#install-it-yourself).
   Abre en tu navegador la dirección que muestra y deja el terminal abierto.
2. Si ves primero el resumen, elige **Customize setup**. Conserva solo el
   historial de un agente que hayas usado en la última semana y pulsa **Next**.
3. Elige **another number of days** e introduce **7** para seleccionar el
   historial actualizado en los últimos siete días. Mash se basa en la
   actividad registrada de la conversación o, si no está disponible, en la
   última modificación del archivo de historial. Las conversaciones que
   indexa pueden contener mensajes más antiguos. Pulsa **Next**, desactiva
   las conexiones a agentes seleccionadas y pasa al resumen.
4. Comprueba que el resumen diga **this machine only** y **no agents**.
   Elige **Set up and index**. Cuando termine, entra en **chats** y busca
   una palabra de esa conversación.

Más adelante puedes ampliar el período o conectar un agente.
[Configuración en el panel](dashboard.md#set-up-mash-from-the-dashboard)

`mash uninstall` muestra qué se eliminaría antes de confirmar y conserva las
conversaciones originales de tus agentes.
[Cómo desinstalar Mash](uninstall.md)

<span id="shorten-the-history-range"></span>

## Acorta el período del historial

Abre **settings** en el panel. En **history range**, elige **recent updates**
(**recent only** en versiones anteriores) e introduce el número de días.
Pulsa **Review changes** y luego **Save to file**.
Ejecuta `mash index` para usar el nuevo período, o reinicia la vigilancia si
está en marcha para que lea la configuración guardada.

El período se aplica a las próximas indexaciones. Si pasas de 90 a 7 días,
una conversación que se actualizó por última vez hace 30 días y que Mash ya
indexó sigue apareciendo en las búsquedas, incluso después de volver a
indexar. Una conversación actualizada hace poco puede contener mensajes
más antiguos. Para quitar una en concreto de Mash, consulta
[Quita una conversación](#remove-a-chat).

<span id="sensitive-information"></span>

## Credenciales y copias

Mash oculta las credenciales que reconoce en los resultados y al compartir
el historial. La detección puede pasar por alto datos sensibles. Revisa lo
que vayas a compartir y guarda las copias de seguridad con el mismo cuidado
que las conversaciones originales.

Mash omite de su copia el texto entre `<private>` y `</private>` si las
dos etiquetas están en el mismo mensaje. El texto sigue en los archivos
originales del agente. Las etiquetas se aplican al texto del mensaje.

<span id="remove-a-chat"></span>

## Quita una conversación

Busca su `chat_id` con `mash list-chats` o `mash search` y ejecuta este comando.

```sh
mash forget CHAT_ID
```

Mash quita la conversación de la copia local y elimina los recuerdos
guardados que la hayan citado. También evita volver a importar la conversación.
Si compartes tu historial, Mash intenta aplicar la eliminación en el servidor.
Revisa el resultado del comando para ver si quedó algo pendiente.

Se conservan los archivos originales del agente, las copias de seguridad
anteriores y las copias indexadas por separado en otros equipos. La operación
no se puede deshacer. Si puedes necesitar la conversación más adelante,
haz antes [una copia de seguridad](backup.md#make-a-portable-backup).

<!-- mash-docs-source: 8ece8fc94aa9d9e86e4d6bb12c10445fe9ed0dbbac23a62ccb44e9bfd12a8496 -->
