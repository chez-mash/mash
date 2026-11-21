# Usa el panel web de Mash

Prueba una [búsqueda sobre la exportación de facturas](https://getmash.io/demo/#/search?q=invoice+exports+queue) en la demo. Abre un resultado para ver la conversación original. También puedes [abrir una decisión guardada](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop) y elegir **open the message** para ver de dónde salió. La demo busca palabras y frases exactas en conversaciones de ejemplo.

Las versiones precompiladas de Mash incluyen el panel web. Si compilas Mash, sigue los pasos para [incluir el panel en la compilación](install-methods.md#include-the-web-dashboard). Una vez [instalado Mash](quick-start.md), abre una terminal y ejecuta este comando.

```sh
mash dash
```

Abre en el navegador la dirección que muestra, normalmente **[http://127.0.0.1:8790](http://127.0.0.1:8790)**. Una vez configurado Mash, la vista general muestra tus conversaciones y recuerdos recientes. Deja abierta la terminal. Ctrl-C detiene el panel. Si la página no carga, [comprueba cómo se inicia el panel](troubleshooting.md#the-dashboard-doesnt-open).

Si la salida aparece en formato JSON, la dirección está en el campo `dashboard`.
Mash usa ese formato cuando rediriges stderr a un archivo o una tubería. Las
versiones anteriores también lo usan en la terminal.

Con el [complemento](plugin.md) instalado, usa `/mash:dashboard` en Claude Code o pídele a tu agente que abra el panel web de Mash.

El panel local se abre sin iniciar sesión en la configuración predeterminada. Lee [cómo mantener el acceso privado](#keep-access-private) antes de cambiar la dirección en la que escucha. Los nombres de los controles se muestran en inglés en el panel.

<span id="set-up-mash-from-the-dashboard"></span>

## Configura Mash desde el panel

Puedes hacer la configuración inicial en el navegador en vez de ejecutar `mash init` en la terminal. Ejecuta `mash dash` y abre el panel. En un equipo sin configuración se abre directamente el asistente, en una pantalla sin barra lateral ni búsqueda. **Skip to the dashboard** te lleva al panel. Desde **settings** puedes volver al asistente cuando quieras.

Revisa los historiales encontrados, el intervalo de fechas y los agentes que puedes conectar para que busquen por ti. **Show history folders** muestra las rutas. En una instalación nueva se indexan por defecto los últimos 90 días en este equipo. Elige **Set up and index** para confirmar o **Customize setup** para cambiar los historiales, el intervalo o las conexiones.

En la primera pantalla de revisión, **Skip agent connections** te permite indexar los mismos historiales y buscar por tu cuenta en el panel o la terminal. Si tu versión empieza por la selección de historiales, elige **Next** para llegar a la revisión.

Mash aplica los cambios cuando eliges **Set up and index** o **Set up without indexing** en la revisión. Guarda la configuración, conecta cada agente elegido mediante su propia herramienta de terminal y, si lo pediste, empieza a indexar en segundo plano. La página indica qué se guardó, qué archivos modificó cada herramienta y cómo deshacer las conexiones. También muestra el progreso hasta que aparecen tus conversaciones.

- Para indexar necesitas el servicio de búsqueda. Si está detenido, inícialo de la forma que corresponda a tu instalación. Si Mash indica un problema de conexión, consulta el motivo y [repara el servicio de búsqueda](troubleshooting.md#qdrant-connection-failed). Luego elige **Check search service again**. Puedes elegir **Set up without indexing** para guardar la configuración primero. Cuando la comprobación sea correcta, esta pantalla mostrará el comando para indexar tus conversaciones.
- Para buscar por significado, [configura los modelos y prepara el índice](indexing.md#enable-semantic-search). Después puedes [elegir la búsqueda por significado para una consulta](#choose-a-search-mode).
- Si vuelves a configurar un equipo que ya usaba Mash, se guarda la nueva configuración y se conserva una copia de la anterior junto a ella.
- Para compartir con otros equipos, crear recuerdos a partir de conversaciones antiguas o leer historiales de otras ubicaciones, usa `mash init` o edita el archivo de configuración.
- Si otros equipos pueden acceder al panel, la configuración desde el navegador requiere haber iniciado el panel con una configuración de servicios. La página de ajustes tiene el mismo requisito.

<span id="the-rail"></span>

## Usa la barra lateral

En el teléfono se abre con el botón de menú.

- **overview** muestra las conversaciones y los recuerdos más recientes, con el número de conversaciones de cada equipo y agente.
- **chats** lista todas las conversaciones de más reciente a más antigua y ofrece un cuadro de búsqueda. **viewing history** lista las que abriste en esta sesión del navegador y te devuelve al punto donde las dejaste.
- **agents**, **machines** y **projects** tienen su propia página, con las conversaciones recientes y su distribución entre las otras dos categorías.
- **memory** reúne decisiones, preferencias y hechos guardados.
- **services** y **fleet manager** muestran el estado de procesos y bases de datos, la configuración de agentes por equipo y las acciones de gestión. **fleet manager** aparece donde se ha configurado la gestión de varios equipos. [Servicios](../services.md)
- **tool explorer** muestra las herramientas y parámetros disponibles para los agentes. Puedes probar una herramienta de solo lectura y ver su respuesta. Las herramientas que cambian datos aparecen en la lista.
- **settings** muestra la configuración del equipo y el estado del índice. Puedes elegir qué historiales se indexan y desde qué fecha, si también se comparten, si se guardan recuerdos y si se ocultan credenciales. Si has configurado [Mashy](#ask-mashy), puedes activarlo, desactivarlo o elegir su modelo. Al guardar, Mash reescribe la configuración y conserva `config.toml.previous`. Reinicia la vigilancia y vuelve a conectar los agentes para aplicar los ajustes. Mashy aplica los suyos al momento. Los ajustes de búsqueda y almacenamiento son de consulta. Si otros equipos pueden acceder al panel, se requiere `--services-config` para cambiar ajustes desde el navegador.

El botón de arriba a la derecha alterna entre el tema claro y el oscuro. El navegador recuerda tu elección.

<span id="check-if-local-search-is-ready"></span>

### Comprueba si la búsqueda local está lista

Abre **settings** y mira el estado de la búsqueda en la parte superior. Los
recuentos de **storage** describen el historial que Mash ha guardado. Para
buscar en este equipo también necesitas un servicio de búsqueda en marcha y
un índice actualizado.

Si Mash no puede usar el servicio de búsqueda local, [comprueba qué falló](troubleshooting.md#qdrant-connection-failed)
antes de volver a indexar. La búsqueda por palabras funciona sin modelos.
Instala los modelos solo si quieres [buscar por significado](indexing.md#enable-semantic-search).

<span id="find-and-read-a-chat"></span>

## Busca y lee una conversación

Abre **chats**, escribe una palabra que recuerdes en **Search your history** y pulsa Entrar. Abre un resultado para ir al mensaje que coincide dentro de su conversación.

Si el cuadro está vacío, verás la lista de conversaciones. Puedes acotar la lista o los resultados por agente, equipo, proyecto y fecha. Un filtro que has cambiado aparece con un borde discontinuo hasta que eliges **Apply filters**. Los filtros aplicados aparecen como etiquetas en el cuadro de búsqueda. Elige una etiqueta para quitarla.

Si una búsqueda con filtros queda vacía, elige **Clear filters** para repetir la misma consulta sin ellos. Si tu versión vuelve a la lista de conversaciones, introduce la consulta de nuevo. Elige **Browse chats** para abrir la lista con los filtros actuales. Usa el botón Atrás del navegador para volver a la búsqueda. Para más ayuda, consulta [cómo encontrar la conversación que falta](troubleshooting.md#search-returns-no-results).

La conversación muestra tus mensajes a la derecha y las respuestas del agente a la izquierda. Las llamadas a herramientas y el razonamiento se agrupan en una línea. Ábrela para ver cada paso y su resultado. El panel lateral muestra tus peticiones de la página actual, otras conversaciones del proyecto y los recuerdos guardados a partir de esta.

Usa **Copy session ID** para [reabrir la sesión original](pick-up-work.md#reopen-the-original-session),
o [retoma el trabajo en una sesión nueva](pick-up-work.md#ask-an-agent-for-a-recap).

Las vistas de conversaciones y recuerdos son de solo lectura.

<span id="choose-a-search-mode"></span>

### Elige un modo de búsqueda

Después de buscar, localiza **search: default** junto a los filtros. Al llegar los resultados, la etiqueta muestra **default (words)** o **default (meaning)**. Abre el menú, elige **words** o **meaning** y selecciona **Apply filters** para repetir la consulta. **words** busca términos de la conversación. **meaning** usa [modelos opcionales](indexing.md#enable-semantic-search) y recibe consultas en inglés. La elección se mantiene al pasar de página. Una nueva consulta vuelve al [modo predeterminado](../configuration.md#search-defaults).

Si tu panel no muestra el menú, elige el modo con [`mash search --intent`](search.md).

<span id="inspect-conversation-records"></span>

## Consulta los recuerdos guardados

Abre **memory** en la barra lateral o elige una categoría. Cambia de categoría con los botones superiores, busca en **find a statement** y filtra por estado, revisión y origen. El origen permite elegir entre los recuerdos de este equipo y los publicados en tu grupo. Elige **Apply filters** para buscar y **Next page** para continuar.

Abre un recuerdo para ver sus revisiones, los mensajes citados y los recuerdos relacionados. **open the message** debajo de una cita abre el mensaje original. En una conversación, **memory from this chat** lista los recuerdos que la citan en alguna revisión conservada. Los mensajes citados muestran **Saved as** debajo.

Las listas incluyen recuerdos pendientes y retirados. Antes de usar uno, revisa su estado, su fecha de caducidad, las advertencias sobre sus fuentes y los conflictos declarados. Para confirmar o cambiar un recuerdo, pídeselo al agente mediante las [herramientas de memoria](../records.md). Si el origen de un recuerdo compartido no está disponible, el enlace muestra un error.

<span id="keep-your-place"></span>

## Conserva tu lugar al leer

El panel sigue las actualizaciones de este equipo y de los que publican en tu índice compartido. La conversación que estás leyendo se mantiene en su sitio.

- **Las listas de conversaciones y recuerdos** se actualizan en unos segundos. Esto incluye la vista general, la barra lateral, la lista de sesiones junto a la conversación y las páginas de agentes, equipos, proyectos y memoria.
- **Una conversación abierta** añade mensajes al final y te lleva a ellos si ya estás al pie de la página. Si estás en otro punto, aparece **N new messages** para saltar a los nuevos. Si Mash vuelve a leer un mensaje anterior, el texto visible cambia cuando eliges **Show updated messages**.
- **Una búsqueda ya enviada**, de conversaciones o recuerdos, conserva los resultados y filtros hasta que vuelves a buscar, incluso si abres un resultado y regresas con el botón Atrás.
- **Un recuerdo abierto** se actualiza al elegir **check for updates**.

Las conversaciones y los recuerdos de otros equipos llegan cuando se publican. El servidor compartido debe estar en una versión igual o más reciente que el panel. Si es más antiguo o tienes abiertas más de 16 pestañas del panel, usa **reload chat list** al pie de la barra lateral o el botón de actualización de una conversación.

Si se corta la conexión del navegador o la del panel con el servidor compartido, el panel vuelve a conectarse y se pone al día.

Para ver las conversaciones nuevas, [actualiza tu historial](synchronization.md#refresh-now). El panel refleja los cambios en el índice.

Cuando una petición falla, el panel indica qué no pudo cargar y ofrece **Retry**. Si hay un error de conexión, comprueba que `mash dash` sigue activo y que el navegador puede acceder a él. Luego elige **Retry**.

<span id="ask-mashy"></span>

## Pregúntale a Mashy

Si has [configurado un modelo](../configuration.md#the-dashboard-assistant), verás la mascota de Mash en un círculo en la esquina inferior derecha. Ábrela y pregunta por tu historial con tus propias palabras.

- «¿En qué trabajé ayer?»
- «¿Qué decidimos sobre la prueba de reinicio?»
- Desde una conversación puedes pedir «Resume esta conversación».
- «¿Por qué no encuentro la conversación de esta mañana?»

En una pantalla grande, arrastra la barra superior para apartar la ventana de lo que estás leyendo. Haz doble clic en la barra para devolverla a su sitio. En el teléfono ocupa la pantalla.

Mashy busca en tus conversaciones y recuerdos y responde brevemente. Puedes abrir en el panel lo que cita.

- Las etiquetas de conversaciones o recuerdos enlazan a su página. Si la respuesta se basa en un mensaje, el enlace lleva a ese mensaje.
- Las citas aparecen con su autor y su fecha. Tus palabras llevan un borde de color.
- Los proyectos, agentes y equipos enlazan a sus páginas.
- Si la respuesta cubre solo una parte de lo que encontró, **See all** abre la búsqueda completa con las mismas palabras.
- Los comandos como `mash index` tienen un botón **Copy**.

Mashy sugiere otras preguntas al final de la respuesta y te avisa si no encontró nada. Usa el botón de detener para interrumpir la respuesta.

Para preguntar sobre un pasaje, selecciónalo en la conversación y elige **Ask Mashy**. El texto queda fijado encima del chat y se envía con cada pregunta hasta que lo quitas.

Mashy puede decirte si la búsqueda está lista, cuál es la conversación más reciente del índice, qué agentes están conectados o cuándo se sincronizaron los demás equipos. Para resolver un problema te indica el comando que puedes ejecutar.

También responde preguntas sobre Mash con las guías incluidas en el programa. Por ejemplo, «¿Cómo comparto el índice con mi portátil?» o «¿Qué cambia `mash init`?». Las guías citadas se abren en este sitio.

Las fechas y horas siguen el reloj del navegador. Las preguntas sobre un periodo, como la semana pasada, se responden con conversaciones que tuvieron mensajes en ese intervalo. Mashy lee las conversaciones largas por partes e indica cuál cubre la respuesta.

Mashy usa herramientas de solo lectura para consultar tu historial y las guías incluidas. No indexa, elimina, guarda recuerdos, ejecuta comandos ni cambia ajustes. El chat de Mashy dura lo que dura la pestaña y queda fuera de tu historial.

Cada pregunta envía al modelo que configuraste el texto que Mashy lee, incluido el pasaje seleccionado. El círculo aparece y Mashy envía texto solo si has configurado el modelo.

Si Mashy falla o responde mal, inicia el panel con `MASH_TRACE=1` y vuelve a preguntar. El registro indica qué consultas hizo, cuánto devolvieron, cuánto tardó el modelo, cuántos tokens usó y por qué se detuvo. Las preguntas, las respuestas y el texto de tu historial quedan fuera del registro.

<span id="save-a-transcript"></span>

## Guarda una conversación

Usa **Export** en la conversación para descargarla entera en Markdown o HTML, incluido el resultado de las herramientas. **Export to PDF** abre una versión para imprimir. Elige **Save as PDF** en el cuadro de impresión de tu navegador.

Para guardar una copia estable, espera a que termine la indexación. Las descargas pueden contener información sensible.

Cada conversación tiene su dirección, `http://127.0.0.1:8790/#/chat/<chat id>`. Con el [complemento](plugin.md), `/mash:export` encuentra la conversación que indiques y la abre allí.

<span id="export-many-chats-at-once"></span>

## Exporta varias conversaciones a la vez

Abre **chats** y elige **export chats**. Filtra por agente, equipo, proyecto y fecha. Se exportan las conversaciones que coinciden en todas las páginas, salvo las que desmarques. Elige **Markdown** para leer o **JSON** para usar en scripts y pulsa **Export**. El navegador descarga un archivo `.zip` con una carpeta por proyecto y un archivo por conversación.

Deja abierta la página hasta que empiece la descarga. Si una conversación no puede leerse, la exportación se detiene y no guarda archivos. Las descargas incluyen el texto completo, también el contenido sensible.

Para restaurar conversaciones en Mash, utiliza una [copia de seguridad portable](backup.md#make-a-portable-backup). La exportación del panel sirve para leer las conversaciones y excluye los adjuntos. Para ver conversaciones de otros equipos en el panel, consulta [cómo compartir el historial](shared-index.md).

<span id="open-it-from-your-phone-or-another-computer"></span>

## Abre el panel desde el teléfono u otro equipo

De forma predeterminada solo puedes acceder desde este equipo. En Linux o macOS, si estás conectado a [Tailscale](https://tailscale.com), ejecuta este comando para dar acceso a tus otros dispositivos dentro de tu red.

```sh
mash dash remote
```

El panel se queda activo como servicio de usuario con inicio de sesión. En Linux usa systemd y en macOS un LaunchAgent. Mash muestra una dirección HTTPS dentro de tu red de Tailscale, un código de un solo uso y un código QR.

```json
{
  "dashboard": "https://laptop.your-tailnet.ts.net:8790/",
  "service": "mash-dash.service",
  "code": "K7M-2QX",
  "code_minutes": 10
}
```

Con el teléfono conectado a Tailscale, escanea el código QR o abre la dirección y escribe el código de un solo uso. El navegador conserva la sesión durante 30 días.

| Para | Ejecuta |
| --- | --- |
| Iniciar sesión en otro dispositivo | `mash dash remote --code` |
| Volver a ver el código QR | `mash dash remote --qr` |
| Usar otro puerto | `mash dash remote --port 8791` (en todos los comandos) |
| Servir el panel desde un equipo sin historial propio | `mash dash remote --shared-only` |
| Acceder desde un dispositivo sin Tailscale | `mash dash remote --public --port 8443` |
| Detener el servicio | `mash dash remote --remove` |

El código de un solo uso caduca a los diez minutos. El QR incluye el token de larga duración después de `#` en la dirección, en el fragmento que el navegador no envía al servidor al cargar la página. El panel inicia sesión y quita el token de la dirección enseguida. Trata una imagen del QR como una contraseña.

El panel busca en el historial de este equipo y en el historial compartido al que tiene acceso. Si usaste `mash init --join`, el panel puede leer el token del equipo en `~/.mash/node.env`. En v0.1.1-alpha, la configuración puede indicar que solo hay acceso local aunque el token guardado funcione. Abre una conversación de otro equipo para comprobarlo. [Busca en todos tus equipos](shared-index.md#search).

Si configuraste el acceso compartido a mano, define la variable del token en la terminal donde ejecutas `mash dash remote`. Mash la guarda para el servicio en `~/.mash/dashboard.env`, que solo tú puedes leer. En macOS, el LaunchAgent también guarda el entorno del servicio en `~/Library/LaunchAgents/dev.mash.dash.plist` y escribe el registro en `~/.mash/dashboard.log`.

Puedes ejecutar `mash dash remote --shared-only` desde un equipo con acceso al índice compartido. Un servidor sin índice local también puede alojar el panel si tiene configurada la conexión y dispone de un token para su nodo. Las conversaciones, las búsquedas y los recuerdos proceden de ese índice compartido.

`--public` publica la dirección en Internet con [Tailscale Funnel](https://tailscale.com/kb/1223/funnel), para que puedas abrirla desde un dispositivo que esté fuera de tu red. Tailscale debe permitir Funnel para ese equipo y solo se pueden publicar los puertos 443, 8443 y 10000. Cualquiera puede abrir la página de inicio de sesión. El token y los códigos de un solo uso protegen todo tu historial. Diez intentos fallidos en un minuto bloquean temporalmente el inicio de sesión para todos. Usa la red privada predeterminada siempre que puedas.

<span id="set-it-up-by-hand"></span>

### Configura el acceso manualmente

En Windows o si usas otro proxy HTTPS, como Caddy, sigue estos pasos.

1. Elige un token de al menos 32 caracteres y guárdalo en una variable de entorno, por ejemplo `MASH_DASH_TOKEN`.
2. Da al panel una dirección HTTPS. Con Tailscale, ejecuta `tailscale serve --bg 8790`.
3. Inicia el panel con ese nombre y la variable del token.

   ```sh
   mash dash --allow-host laptop.your-tailnet.ts.net --access-token-env MASH_DASH_TOKEN
   ```

`--allow-host` indica el nombre que ven los navegadores, incluido el puerto si figura en la dirección. Puedes repetirlo para añadir otros nombres. Si cambias el token, se cierran las sesiones de todos los navegadores.

<span id="keep-access-private"></span>

## Mantén el acceso privado

Con `--access-token-env` se requiere iniciar sesión para ver el historial. Diez tokens incorrectos en un minuto bloquean el inicio de sesión hasta que pase ese minuto.

La dirección predeterminada solo es accesible desde tu equipo. Si ejecutas `mash dash --listen` con otra dirección sin configurar el inicio de sesión, cualquiera que pueda conectarse podrá ver el historial. Mash lo indica al iniciarse. `--allow-host` requiere un token.

Prefiere una red privada. En Internet, el token protege todo el historial. Un servicio que termina la conexión HTTPS, como Cloudflare Tunnel, puede leer los datos que pasan por él.

<!-- mash-docs-source: 9333cd17a6028dbceb6c1dd85ef0fe4d650fe89da81e2a6c288b9799443a337e -->
