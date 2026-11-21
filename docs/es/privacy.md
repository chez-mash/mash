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

<span id="when-history-can-leave-your-machine"></span>

## Cuándo puede salir parte del historial

- **Agentes conectados.** Cuando un agente usa Mash, recibe resultados y
  fragmentos de las conversaciones. Esos fragmentos pasan al contexto del
  agente y pueden enviarse a su proveedor de modelos. Las [acciones automáticas del complemento](plugin.md#what-happens-without-asking)
  también pueden añadir fragmentos al contexto sin una conexión MCP. Para
  buscar solo desde el panel local o la terminal, deja los agentes sin conectar
  y desactiva o elimina el complemento de Mash si lo instalaste.
- **Historial compartido.** Mash envía al servidor que configures las conversaciones
  y los adjuntos que conserva en el índice. Cada equipo con un token para ese grupo
  puede leerlos. [Revisa qué se compartirá antes de publicar](shared-index.md#before-sharing).
- **Servicio remoto para calcular representaciones semánticas.** Si lo
  configuras, recibe el texto que Mash prepara para la búsqueda, que puede
  contener datos sensibles. Por defecto, Mash hace este trabajo en el equipo.
  Las representaciones de tus consultas siempre se calculan en local.
- **Servicio remoto para reordenar resultados.** Si lo activas, recibe cada
  consulta y los fragmentos que va a ordenar. Por defecto, Mash los ordena
  en tu equipo.
- **Asistente del panel.** Solo se activa al configurar un
  [servicio de modelos (en inglés)](../configuration.md#the-dashboard-assistant).
  Envía tus preguntas y los fragmentos y recuerdos que consulta a ese servicio.
- **Recuperar recuerdos de chats anteriores.** Si eliges esta opción en la
  configuración inicial, los chats seleccionados se envían al proveedor del
  agente que elijas. [Cómo funciona la extracción de recuerdos](records.md#backfill-earlier-memory)

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

<!-- mash-docs-source: 1a3708325e6cbe8a9565dfe0b23eaf7f944ee2ad2d9d72c6073510ba11e3a4e4 -->
