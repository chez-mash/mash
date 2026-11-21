# Privacidad del historial

[Privacy in English](../privacy.md) · [Guía de inicio rápido](quick-start.md)

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

<span id="where-mash-stores-data"></span>

## Dónde guarda Mash los datos

Por defecto, Mash guarda las conversaciones indexadas en
`~/.mash/mash.sqlite3` y la configuración en `~/.mash/config.toml`. En Windows,
`~` es tu perfil de usuario. Qdrant guarda aparte los datos de búsqueda,
incluido el texto en el que se puede buscar. En Windows, Mash usa una ruta
corta en la unidad del sistema, normalmente `C:\mash-qdrant\storage`.
[Archivos del servicio de búsqueda por plataforma (en inglés)](../local-search.md#where-qdrant-stores-data)

Ejecuta `mash status` para consultar `database_path` y, si Mash instaló Qdrant,
`local_qdrant.installed.storage`. Usa `mash uninstall` para ver qué se
eliminaría, incluida la carpeta separada de Qdrant en Windows. Añade `--yes`
cuando quieras desinstalar Mash.

## Cuándo puede salir parte del historial

- **Agentes conectados.** Cuando un agente usa Mash, recibe resultados y
  fragmentos de las conversaciones. Esos fragmentos pasan al contexto del
  agente y pueden enviarse a su proveedor de modelos. Para buscar por tu
  cuenta, puedes usar el panel local o la terminal.
- **Historial compartido.** Si habilitas la búsqueda entre equipos, Mash
  envía el historial al servidor que configures y alojes.
- **Servicios remotos opcionales.** Si configuras un servicio remoto para
  preparar el índice o reordenar resultados, ese servicio puede recibir
  texto de las conversaciones. Si habilitas el asistente del panel, enviará
  tu pregunta y los fragmentos que consulte al servicio de modelos que
  configures.
- **Recuperar recuerdos de chats anteriores.** Si eliges esta opción en la
  configuración inicial, los chats seleccionados se envían al proveedor del
  agente que elijas.

## Prueba con menos historial

[Prueba una búsqueda con conversaciones de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue).
Para probar Mash con uno de tus historiales:

1. Ejecuta el [instalador de la guía de inicio](quick-start.md) en tu terminal.
   El instalador inicia el servicio de búsqueda local antes de abrir el asistente.
2. Si el asistente empieza con un resumen, elige **Customize setup**. Cuando
   pregunte dónde buscar, elige **Just this computer (recommended)**. Conserva
   un historial con una conversación reciente, elige **Custom number of days**
   e introduce 7. Si te lo pregunta, desmarca todos los agentes y rechaza la
   extracción de recuerdos de conversaciones anteriores.
3. Elige **Set up and index** y ejecuta `mash dash`. Abre la dirección que
   aparezca, selecciona **chats** y busca una palabra de esa conversación.

Para usar todos los historiales sugeridos sin conectar agentes, elige
**Set up and index without connecting agents** en el primer resumen.
También puedes [elegir estas opciones en el navegador](dashboard.md#set-up-mash-from-the-dashboard).

`mash uninstall` muestra qué se eliminaría antes de confirmar y conserva las
conversaciones originales de tus agentes.
[Cómo desinstalar Mash](installation.md#uninstall)

## Credenciales y copias

Mash oculta las credenciales que reconoce en los resultados y al compartir
el historial. La detección puede pasar por alto datos sensibles. Revisa lo
que vayas a compartir y guarda las copias de seguridad con el mismo cuidado
que las conversaciones originales.

Para quitar un chat del índice, consulta la
[guía completa de privacidad (en inglés)](../privacy.md#remove-a-chat).
