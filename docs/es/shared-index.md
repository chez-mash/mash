# Busca en todos tus equipos

Si tanto tu portátil como tu ordenador de sobremesa guardan chats de agentes, un índice compartido te permite encontrarlos desde cualquiera de los dos. Cada equipo conserva una copia local en la que puedes buscar y publica su índice en un único servidor Mash que tú administras.

| Qué quieres hacer | Por dónde empezar |
| --- | --- |
| Leer los chats de un equipo desde el móvil u otro navegador | [Abrir de forma remota el panel de ese equipo](dashboard.md#open-it-from-your-phone-or-another-computer) |
| Buscar en el historial de chats indexados en varios equipos | [Configurar un índice compartido](#the-quick-way) |
| Buscar solo en el historial de este equipo | [Usar el panel local](dashboard.md#find-and-read-a-chat) |

Para ver cómo se muestra un resultado de otro equipo, haz una búsqueda en la demostración y abre **Choose a queue for invoice exports**, un chat de la estación de trabajo.
[Prueba la búsqueda de ejemplo](https://getmash.io/demo/#/search?q=invoice+exports+queue)

<span id="before-sharing"></span>

## Antes de compartir

**Al publicar, envías al servidor los chats indexados y los archivos adjuntos que se han conservado.**
Cualquier equipo con un token del mismo grupo puede buscar en ese historial y leerlo. Usa un servidor y una red de confianza. Antes de empezar a publicar, revisa los chats que ya has indexado, incluidos los más antiguos que Mash ha conservado.
[Cuándo puede salir el historial de tu equipo](privacy.md#when-history-can-leave-your-machine)

<span id="what-runs-where"></span>

## Qué se ejecuta en cada equipo

| Dónde | Qué se ejecuta | Cuántas veces se configura |
| --- | --- | --- |
| Un equipo que hace de servidor | `mash-server` y una instancia de Qdrant 1.20 o posterior para el servidor, ambos con almacenamiento persistente | una vez |
| Cada equipo que indexa chats | `mash` con su índice local y su instancia local de Qdrant, tal como se explica en la guía de [instalación](installation.md) | una vez por equipo |

El servidor puede ser uno de tus equipos o un equipo independiente que esté siempre encendido.
`mash-server` se distribuye con `mash`. Usa la misma versión en el servidor y en todos los equipos.

Si ya tienes un servidor, pide a quien lo administra una cadena de conexión para cada equipo. También puedes conectarte con la URL del servidor, el ID del grupo, un nombre para el equipo (su ID de nodo) y su propio token. [Conecta cada equipo](#2-connect-each-computer)

<span id="the-quick-way"></span>

## Configuración rápida

Sigue estos pasos si el servidor usa Linux o macOS, tiene Qdrant en ejecución y puede usar Tailscale Serve. Conecta el servidor y los equipos a tu tailnet. Activa MagicDNS y los certificados HTTPS para esa red.
[Inicia Qdrant](installation.md#start-the-local-search-service) · [Activa HTTPS en Tailscale](https://tailscale.com/docs/how-to/set-up-https-certificates)

En el servidor, añade tu portátil a un grupo llamado `personal`.

```sh
mash-server --setup --add-node personal/laptop
```

Esto guarda la configuración del servidor en `~/.mash/server`, inicia un servicio de usuario y muestra su dirección HTTPS privada y la cadena de conexión `join`. Para añadir otro equipo, vuelve a ejecutar el comando en el servidor con otro nombre, como
`personal/desktop`. Usa una cadena de conexión distinta para cada equipo. Como la cadena contiene su credencial, compártela en privado. Si vuelves a ejecutar la configuración con un nombre ya registrado, se repara el servicio y se reutiliza ese registro.

<span id="join-and-review-each-computer"></span>

### Conecta y revisa cada equipo

En el equipo que quieras conectar, instala Mash e inicia su instancia local de Qdrant. Pausa la indexación en segundo plano mientras haces la revisión. Después, ejecuta el comando con la cadena de conexión completa de ese equipo.

```sh
mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents
```

Al conectarte, se guardan los datos del servidor, del grupo y del nodo, y la credencial se almacena en `~/.mash/node.env`, con permisos de lectura solo para ti. Estas opciones omiten la indexación y dejan intactos los ajustes de los agentes. El comando `mash init --join` sin esas opciones también conecta los agentes compatibles que detecta e inicia la indexación y la publicación sin pasar por el asistente.

Ejecuta `mash status --installation` para comprobar las carpetas del historial configuradas y usa el panel local para revisar los chats que ya estén indexados. Cuando quieras enviar ese historial al servidor, [publica y verifica el historial de cada equipo](#3-publish-and-check).

Con esta configuración, el servidor busca por palabras y no necesita descargar ningún modelo. Si el servidor no tiene Tailscale o necesita ofrecer búsqueda semántica, usa la configuración manual que se describe más abajo. Para detener un servidor creado con `--setup`, ejecuta
`mash-server --setup --remove`. Se conservan la configuración y el historial para la próxima vez que inicies el servidor.

<span id="1-set-up-the-server"></span>

## 1. Configura el servidor

En el equipo que hace de servidor:

1. Inicia Qdrant para el servidor.
   [`mash qdrant install`](installation.md#start-the-local-search-service)
   inicia una instancia en `127.0.0.1`, en los puertos 6333 (HTTP) y 6334 (gRPC). También puedes usar la
   [imagen de Docker](local-search.md#optional-qdrant-docker-image) o un servicio Qdrant 1.20 o posterior que ya esté en ejecución. Mantén los puertos de Qdrant accesibles solo desde este equipo.

2. Crea un token para cada equipo. Cada token debe tener entre 32 y 4096 caracteres ASCII imprimibles, sin espacios. Por ejemplo:

   ```sh
   openssl rand -hex 32
   ```

3. Guarda lo siguiente como `server.toml`. El archivo enumera todos los equipos autorizados a conectarse y configura la búsqueda solo por palabras, así que el servidor no necesita descargar ningún modelo:

   ```toml
   version = 1
   listen = "127.0.0.1:8787"
   qdrant_url_env = "MASH_QDRANT_URL"
   qdrant_grpc_url_env = "MASH_QDRANT_GRPC_URL"

   [views.build]
   representations = "lexical-only"

   [[nodes]]
   group_id = "personal"
   node_id = "laptop"
   token_env = "MASH_LAPTOP_TOKEN"

   [[nodes]]
   group_id = "personal"
   node_id = "desktop"
   token_env = "MASH_DESKTOP_TOKEN"
   ```

   El archivo contiene los nombres de las variables de entorno. Los secretos se guardan por separado. Mantén los tokens fuera de Git.

4. Define esas variables e inicia el servidor. Ambas variables de Qdrant deben apuntar a la **misma** instancia de Qdrant:

   ```sh
   export MASH_QDRANT_URL=http://127.0.0.1:6333
   export MASH_QDRANT_GRPC_URL=http://127.0.0.1:6334
   export MASH_LAPTOP_TOKEN='the token you made for the laptop'
   export MASH_DESKTOP_TOKEN='the token you made for the desktop'
   mash-server --server-config /absolute/path/to/server.toml
   ```

   Mantén el servidor en ejecución con tu gestor de servicios y las mismas variables.

5. Permite el acceso al servidor. `listen = "127.0.0.1:8787"` solo acepta conexiones desde el propio servidor. Pon un proxy inverso HTTPS delante del servidor o configura
   `listen` con la IP del equipo en la red privada y usa esa dirección como URL del servidor.

Esta configuración es para un servidor nuevo. Si ya tienes uno, haz una copia de seguridad antes de cambiar su configuración.

<span id="2-connect-each-computer"></span>

## 2. Conecta cada equipo

Haz esto en todos los equipos que deban publicar el historial compartido o buscar en él. Mash ya debe estar [instalado](installation.md) en cada equipo, con su instancia local de Qdrant en ejecución.

<span id="with-a-join-string"></span>

### Con una cadena de conexión

En el servidor, registra cada equipo con un ID de nodo único dentro del grupo. Indica la dirección que usará ese equipo para acceder al servidor.

```sh
mash-server --server-config /absolute/path/to/server.toml \
  --add-node personal/laptop --api-url https://mash.example.net
```

El comando muestra un `join` que empieza por `mash-join.`. Reinicia el servidor para que acepte el nuevo equipo. Luego, [conecta y revisa ese equipo](#join-and-review-each-computer). El servidor guarda un hash de la credencial en un archivo separado, junto al archivo de configuración, por ejemplo `server.toml.nodes`.
[Sustituye o revoca una credencial de unión](#revoke-or-replace-a-computers-credential)

<span id="by-hand"></span>

### Manualmente

También puedes definir el token y ajustar la configuración tú mismo.

Primero, guarda el token de este equipo en una variable de entorno, por ejemplo
`MASH_NODE_TOKEN`. Mash solo guarda el nombre de la variable. Defínela en todos los entornos donde se ejecute Mash, incluido el intérprete de comandos, el
[monitor](../synchronization.md) y los clientes de agentes que inician el servidor MCP de Mash.

A continuación, usa el asistente o edita la configuración.

**Con el asistente.** Ejecuta `mash init --no-index --no-agents` en un terminal. Si el asistente se abre en una pantalla de revisión, elige **Customize setup** (llamado **Review and edit settings**
en la versión v0.1.1-alpha). Cuando te pregunte dónde quieres buscar, elige **Connect to an existing shared server**. Introduce la URL del servidor, el ID del grupo, el ID de nodo de este equipo y el nombre de la variable del token. Revisa las carpetas del historial y elige **Set up without indexing**. En el siguiente paso enviarás el historial al servidor.

**Manualmente.** Edita la configuración del nodo que creó `mash init`. Conserva los proveedores y los ajustes de la instancia local de Qdrant. En la tabla `[index]` que ya existe, define
`destination`y añade una tabla `[shared]` :

```toml
[index]
destination = "both"
since_days = 90

[shared]
group_id = "personal"
node_id = "laptop"
api_url = "https://mash.example.net"
token_env = "MASH_NODE_TOKEN"
```

`group_id` y `node_id` deben coincidir con los de una misma entrada `[[nodes]]` del servidor. La variable debe contener el token de esa entrada. En el ordenador de sobremesa, usa
`node_id = "desktop"` y el token de ese equipo.

Los ajustes de búsqueda deben ser compatibles con el servidor. Para el servidor de búsqueda por palabras del ejemplo anterior, cada equipo usa `[views.build] representations = "lexical-only"` y
`[views.query] intent = "exact-mention"`, que son los ajustes que `mash init` escribe cuando no hay modelos instalados. Para la búsqueda semántica, pide a quien administra el servidor los ajustes de modelo correspondientes.

<span id="3-publish-and-check"></span>

## 3. Publica y verifica el historial

En **cada** equipo:

```sh
mash index
mash status --shared
```

`mash index` actualiza el índice local y después publica el historial que conserva. El intervalo de fechas configurado determina qué archivos del historial lee Mash en esta ejecución. También se pueden publicar chats más antiguos que ya estén en el índice local.
`mash status --shared` muestra qué recibió el servidor en `shared_index`. Si ves `shared_error`, comprueba la dirección del servidor, la conexión y el token.

Los chats de un equipo aparecen en los demás solo después de que ese equipo los haya publicado. Si falla la publicación, se conserva el historial local. Corrige el error indicado y vuelve a ejecutar `mash index`. Para seguir publicando chats nuevos, ejecuta un
[monitor](../synchronization.md) en cada equipo.

<span id="search"></span>

## Busca

Abre el panel con `mash dash`, ve a **chats** y busca algo que recuerdes del otro equipo. El resultado indica de qué equipo procede. Para limitar la búsqueda a un equipo, usa el filtro de equipos y elige **Apply filters**.
[Lee un chat en el panel](dashboard.md#find-and-read-a-chat)

En la terminal, ejecuta estas órdenes.

```sh
mash search 'authentication'            # this computer and the shared index
mash search 'authentication' --shared   # the shared index only, including this computer's published chats
mash search 'authentication' --local    # this computer only; the server is not contacted
```

Añade `--node desktop` para limitar la búsqueda a los chats de ese nodo. Las consultas del historial compartido van al servidor, así que el ordenador de sobremesa puede estar en suspensión. Para acceder desde el móvil,
[abre el panel de forma remota](dashboard.md#open-it-from-your-phone-or-another-computer)
desde un equipo que ya esté configurado para buscar en este grupo.

<span id="remove-a-machine-or-let-a-rebuilt-one-publish-again"></span>

## Retira un equipo o vuelve a publicar tras reconstruir su índice

El servidor vincula cada ID de nodo al índice local que publicó primero con ese ID. Así, otro equipo no puede sobrescribir ese historial reutilizando el nombre. Por el mismo motivo, si se borra y se reconstruye el índice de un equipo, el servidor rechaza la publicación con el error «shared node ownership or revision changed». Si restauras el índice desde una [copia de seguridad](../upgrades.md#replace-an-index-that-cannot-be-upgraded) con
`--keep-identity`, conservas el nombre. Si no puedes restaurarlo, retira el nombre en el servidor. Para retirar un equipo definitivamente, deja de publicar desde él y elimina su historial del servidor antes de revocar su credencial. Si desinstalas Mash de un equipo, la copia del servidor se conserva.

Detén `mash-server` y deja su instancia de Qdrant en ejecución. Ejecuta este comando con la misma configuración del servidor y el mismo entorno que usa el servicio:

```sh
# On the server machine, with mash-server stopped.
mash-server --server-config /absolute/path/to/server.toml --retire-node personal/laptop
```

**Esto borra de forma permanente todo lo que publicó ese equipo:** sus chats y registros en el servidor, los datos de búsqueda asociados y los archivos adjuntos compartidos. El comando muestra cuántos chats y mensajes eliminó y termina sin iniciar el servicio. Los demás equipos y el historial local del equipo retirado quedan intactos.

Si estás reconstruyendo el índice de un equipo, vuelve a iniciar `mash-server` y luego ejecuta
`mash index` en ese equipo para publicar desde su índice actual. Si vas a retirar el equipo, revoca su credencial antes de reiniciar el servidor.

<span id="revoke-or-replace-a-computers-credential"></span>

### Revoca o sustituye la credencial de un equipo

En el servidor, busca el bloque `[[nodes]]` que contiene los valores de `group_id` y
`node_id` de ese equipo. Si añadiste el equipo con `--add-node`, el bloque está en el archivo que se guarda junto al archivo de configuración del servidor, normalmente `server.toml.nodes`. Si lo añadiste manualmente, el bloque está en `server.toml`. Elimina el bloque y reinicia el servidor para revocar el acceso. Si era el último equipo de la configuración, deja el servidor apagado. Para conservar los archivos publicados en el servidor, omite el comando de retirada.

Para sustituir una cadena de conexión que se ha perdido o se ha expuesto, elimina el bloque anterior, vuelve a añadir el mismo grupo y nodo, y reinicia el servidor. Conéctate desde el mismo equipo con la cadena nueva. Si el equipo conserva su índice local, puede seguir publicando con el mismo nombre.

Antes de instalar una nueva versión de Mash, consulta
[las instrucciones de actualización y copia de seguridad](../upgrades.md#upgrade-a-shared-index).

<!-- mash-docs-source: d53981430b20b1e97415d553067b960e398181e3c769b414ed3965a9f7296bd1 -->
