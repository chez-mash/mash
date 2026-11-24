# Solución de problemas

| Qué ocurre | Por dónde empezar |
| --- | --- |
| La terminal no encuentra `mash` o PowerShell dice que no reconoce el comando | [Encuentra el comando instalado](#mash-command-not-found) |
| El asistente no logra conectar con el servicio de búsqueda | [Inicia Qdrant](#qdrant-connection-failed) |
| La dirección del panel no se abre | [Comprueba el arranque del panel](#the-dashboard-doesnt-open) |
| Mash no encuentra el historial o aún no lo ha indexado | [Revisa la configuración del historial](#nothing-is-indexed-yet) |
| Falta una conversación reciente | [Comprueba si hay historial nuevo](#new-chats-are-missing-or-search-is-behind) |
| Puedes leer conversaciones, pero la búsqueda no da resultados | [Revisa la consulta, los filtros y las fechas](#search-returns-no-results) |
| Un agente de programación no encuentra Mash | [Comprueba la conexión del agente](#my-agent-cannot-find-mash) |
| Tras actualizar, Mash dice que el formato del índice es incompatible | [Recupera o actualiza el índice](#the-index-schema-is-older-or-newer-than-mash) |

¿Sigues sin resolverlo o te preocupa un problema de seguridad? [Pide ayuda](#get-help).

<span id="check-what-mash-sees"></span>

## Comprueba qué detecta Mash

Si no está claro qué ocurre, ejecuta el diagnóstico de solo lectura.

```sh
mash doctor
```

En `findings`, busca las entradas con `"state": "problem"`. El campo
`summary` describe el problema y `repair` indica cómo solucionarlo cuando
hay un paso concreto. Si encuentra un problema, el comando termina con
código 1. Las entradas `warning` señalan qué conviene revisar. `mash doctor`
conserva tu configuración y tu historial.

Para revisar una parte concreta de tu instalación, usa el comando correspondiente.

```sh
mash status                  # index and search readiness
mash status --installation   # history locations and agent connections
mash status --shared         # the index shared with your other machines
```

`mash status --installation` funciona aunque el índice aún no esté listo.
Si ves un error, busca más abajo su `error.kind` o una frase de `error.message`.

<span id="mash-command-not-found"></span>

## La terminal no encuentra el comando mash

Usa la ruta completa que mostró el instalador o añade su carpeta a `PATH`
en la terminal abierta. Estos comandos usan la ruta predeterminada del
instalador de la versión publicada.

```sh
export PATH="$HOME/.local/bin:$PATH"
mash --version
```

En Windows, usa estos comandos en tu ventana de PowerShell.

```powershell
$env:Path = "$HOME\.local\bin;$env:Path"
mash --version
```

Si elegiste otro directorio, úsalo en lugar de `.local/bin`. Con
`cargo install`, la ruta habitual es `.cargo/bin` dentro de tu carpeta
personal. Si `mash --version` muestra un número de versión, continúa en esa
terminal. Para conservar el cambio al abrir nuevas terminales, añade la
carpeta al perfil de tu shell o a la variable de usuario **Path** en las
variables de entorno de Windows.

<span id="the-dashboard-doesnt-open"></span>

## El panel no se abre

Deja `mash dash` en marcha y abre la dirección que muestra en un navegador
del mismo equipo. Si ya iniciaste Mash en otra terminal, abre la dirección
que aparece allí. Si otro programa está usando la dirección, deja que Mash
elija un puerto local libre.

```sh
mash dash --listen 127.0.0.1:0
```

El `0` pide un puerto libre. Abre la dirección que muestra Mash. Si necesitas
un puerto fijo, usa uno disponible, como `127.0.0.1:8791`.

Si la página indica que el panel no se ha compilado, instala una versión precompilada o
[incluye el panel al compilar desde el código fuente](install-methods.md#include-the-web-dashboard).

<span id="qdrant-connection-failed"></span>

## Falló la conexión con Qdrant

Ejecuta `mash doctor` y busca `search-service` para saber qué impide la búsqueda.

Si `mash status` muestra `rejected` en `local_qdrant`, comprueba el motivo
antes de repetir la configuración o la indexación. Mash puede estar usando
el puerto o la clave de API incorrectos, o conectándose a una versión antigua
de Qdrant. Las direcciones HTTP y gRPC deben apuntar a la misma instancia de
Qdrant de la serie 1.x, a partir de la versión 1.20. Consulta [cómo configurar
las direcciones y la clave de API (en inglés)](../configuration.md#local-qdrant).

Si ves `qdrant-unavailable`, `qdrant-install` o «Could not reach Qdrant at …»,
o si `local_qdrant` muestra `unreachable`, comprueba si el servicio está activo.

Si el asistente falló en Linux o WSL con un error de `systemctl`,
[comprueba tu sesión y elige cómo iniciar Qdrant](local-search.md#linux-without-a-systemd-user-session).

Inicia Qdrant según cómo lo instalaste.

- **`mash qdrant install`**. Vuelve a ejecutarlo. Reinicia el servicio si
  está detenido y restaura archivos que falten. En `mash status`, consulta
  `local_qdrant.installed` para ver su estado y dónde encontrar el registro.
  En Linux puedes usar `journalctl --user --unit mash-qdrant.service`.
- **Docker**. Inicia el contenedor que ya existe y lee su registro.

  ```sh
  docker start mash-local-qdrant
  docker logs mash-local-qdrant
  ```

- **Tu propio Qdrant**. Reinícialo desde su carpeta original y lee la
  salida en la terminal.

Si todavía no lo has configurado, sigue los
[pasos de instalación](installation.md#start-the-local-search-service).
El servicio está listo cuando la [comprobación de disponibilidad](http://127.0.0.1:6333/readyz)
muestra «all shards are ready».

`qdrant-failure` significa que Qdrant devolvió un error. Lee su registro
para detectar problemas con el almacenamiento, las credenciales o el
formato de sus datos.

Mientras Qdrant esté detenido puedes listar conversaciones, leer mensajes
y exportar una copia. Si necesitas configurar Mash, elige
**Save configuration only** o ejecuta `mash init --no-index`. Después
ejecuta `mash index` para guardar las conversaciones en este equipo. Se
detendrá con `qdrant-unavailable` e indicará cuántas guardó. Puedes leerlas
con `mash list-chats` y `mash view`. Cuando Qdrant esté en marcha, ejecuta
`mash index` otra vez para actualizar la búsqueda.

En Windows, si el registro de Qdrant menciona `gridstore`, «path specified»
y `os error 3`, la ruta de los datos puede ser demasiado larga. Usa las
rutas absolutas cortas de la [guía para Windows](local-search.md#native-windows-without-docker).
Si ya tienes datos, detén Qdrant y guarda una copia antes de moverlos.
Conserva los datos originales. Si solo cambias la ruta en la configuración,
Qdrant abrirá un directorio vacío y los datos se quedarán en el anterior.

<span id="nothing-is-indexed-yet"></span>

## Todavía no hay nada indexado

Busca el error `database-missing`.

| Mensaje | Comando |
| --- | --- |
| “Mash is not set up on this machine” | `mash init` |
| “Mash is configured, but nothing is indexed yet” | `mash index` |

Si ves «database does not exist», inicia la configuración con `mash init`
o ejecuta `mash index` si ya la guardaste. Ambos necesitan que Qdrant esté
en marcha para preparar la búsqueda.

Si Mash dice que no encontró conversaciones, guarda una conversación en
un agente compatible en este equipo y vuelve a ejecutar `mash init`. Si
ya tienes conversaciones, usa `mash status --installation` para ver las
carpetas de historial que detectó. Revisa los agentes y las carpetas
elegidos en [la configuración](installation.md#connect-your-histories).
Para encontrar conversaciones antiguas, [revisa el periodo de historial](indexing.md#add-older-history).

<span id="my-agent-cannot-find-mash"></span>

## Mi agente no encuentra Mash

Comprueba el agente que quieras usar. Para Claude Code, ejecuta esto.

```sh
mash status --installation --agent claude-code
```

Usa `--agent codex` para Codex. Para Copilot CLI, consulta [cómo comprobar la conexión en tu versión de Mash](agents.md#confirm-the-connection). Si el informe
propone una solución en `next_action`, sigue ese paso. Mientras
[configuras la conexión](agents.md#connect-claude-code-codex-or-copilot),
puedes buscar desde el panel.

Reinicia el agente tras configurarlo, repararlo o actualizar Mash o el
plugin. Una conversación abierta conserva su conexión anterior. Para
probar una conexión nueva, ejecuta lo siguiente.

```sh
mash status --installation --agent claude-code --probe
```

Si la comprobación tiene éxito, significa que un proceso MCP nuevo pudo
conectarse. [Cómo conectar un agente](agents.md)

<span id="search-returns-no-results"></span>

## La búsqueda no da resultados

1. Comprueba el [modo de búsqueda](dashboard.md#choose-a-search-mode).
   En la búsqueda por palabras, todos los términos deben aparecer en el
   mismo mensaje. Prueba con una sola palabra distintiva. Para buscar por
   significado, prueba una pregunta breve en inglés. Consulta
   [cómo buscar en otros idiomas](search-languages.md) si las palabras no
   se separan con espacios.
2. Si el panel muestra etiquetas de filtros, elige **Clear filters** para
   repetir la consulta. Si vuelves a la lista de conversaciones, escribe
   de nuevo la consulta. Desde la terminal, repítela sin las opciones de filtrado.
3. Comprueba si la conversación está en la lista. Elige **Browse chats**
   para abrir la lista con los filtros actuales. Si recuerdas parte del
   título, escríbela en la barra de búsqueda y mira las sugerencias bajo
   **chats**. En la terminal, usa el comando de abajo.
4. Si falta una conversación reciente o termina antes de lo esperado,
   [comprueba si hay historial nuevo](#new-chats-are-missing-or-search-is-behind). Para una conversación
   antigua, [incluye más historial](indexing.md#add-older-history).
   Si faltan las conversaciones de un agente,
   [comprueba las carpetas de historial](#nothing-is-indexed-yet).

La búsqueda abarca el diálogo entre tú y el agente. Para ver el razonamiento
o la salida de herramientas, busca un mensaje cercano y abre la conversación.

Para buscar un título en la terminal, usa este comando con la parte que recuerdes.

```sh
mash list-chats --title-contains 'part of the title'
```

Lee desde el primer mensaje con `mash view` y el valor `lookup.view.id` de
la fila, o busca dentro de la conversación con su identificador.

<span id="models-are-missing"></span>

## Faltan los modelos

Busca «semantic models are not installed» (`model`) o «related-discussion
needs semantic search, which this configuration turns off» (`usage`).

La búsqueda por significado es opcional y requiere descargar modelos.
Puedes buscar palabras y frases así.

```sh
mash search 'auth_token' --intent exact-mention
```

Para buscar por significado, [instala los modelos](indexing.md#install-the-models)
y [prepara los datos de búsqueda](indexing.md#build-search-data).
La instalación conserva tus ajustes. Si ya usas Mash, la guía explica cómo
habilitar la indexación por significado en tus ajustes existentes.

<span id="embedding-failed-during-indexing"></span>

## Falló el cálculo de vectores durante la indexación

Si aparece `embedding_error`, la pasada de `mash index` terminó pero debe
repetirse. Mash pudo guardar los mensajes nuevos, pero no pudo calcular sus
vectores, por ejemplo porque faltan los modelos o un servicio remoto de
cálculo no respondió. Los mensajes nuevos ya se pueden encontrar por
palabras (`exact-mention`). Para buscarlos por significado
(`related-discussion`), corrige el error que aparece y vuelve a ejecutar
`mash index`.

<span id="new-chats-are-missing-or-search-is-behind"></span>

## Faltan conversaciones recientes o la búsqueda va con retraso

Si falta una conversación reciente o aparece `search-not-ready`, ejecuta
este comando en el equipo donde el agente guardó la conversación.

```sh
mash index
```

Cuando termine la indexación, elige **Search again** en el panel. Si tu panel
no muestra ese botón, vuelve a cargar la página de búsqueda. Si la conversación
es de otro equipo, [comprueba que se haya publicado](synchronization.md#across-machines).

- «search is catching up with new history» indica que ya se está indexando.
  Vuelve a buscar en un momento.
- «search is behind the stored history and nothing is refreshing it» indica
  que debes ejecutar `mash index`.

El plugin de agente inicia una actualización al abrir una sesión, como
máximo una vez cada diez minutos. Para actualizar continuamente, consulta
[cómo mantener el historial al día](synchronization.md).
Si nunca aparecen conversaciones de un agente, revisa
[qué historiales locales admite Mash y cómo elegirlos](supported-agents.md#if-a-chat-is-missing).

<span id="a-chat-still-shows-what-an-older-release-read"></span>

## Una conversación aún muestra los datos que leyó una versión anterior

Mash omite una conversación si sus archivos siguen igual que la última vez.
Si una actualización cambia cómo se lee el historial, por ejemplo cómo se
asigna una carpeta, las conversaciones ya indexadas conservan el resultado
anterior hasta que cambien. Para volver a leer todo ahora, ejecuta lo siguiente.

```sh
mash index --reread
```

Mash conserva los datos de búsqueda de las conversaciones que siguen
iguales, así que este paso es más rápido que una primera indexación. Para
limitarlo a un agente, añade `--provider codex`, por ejemplo.

<span id="search-views-are-missing-or-stale"></span>

## Faltan datos de búsqueda o están desactualizados

Si la base de datos es compatible y el historial guardado está listo,
reconstruye los datos de búsqueda local a partir de él.

```sh
mash index --rebuild-local-views --full
```

Este comando trabaja con el historial que Mash ya guardó. Para leer nuevos
archivos o volver a intentar leer un origen que falló, ejecuta `mash index`
y comprueba el resultado. Si el formato de la base de datos es incompatible,
sigue los pasos de actualización de abajo.

<span id="the-index-schema-is-older-or-newer-than-mash"></span>

## El formato del índice es más antiguo o más reciente que Mash

Si aparece `schema-incompatible`, detente y consulta
[actualizaciones y copias de seguridad](upgrades.md).
Conserva el ejecutable anterior y los datos. Exporta con la versión que
escribió el índice y valida la importación con la versión nueva en otra
ruta.

Mantén la base de datos y su número de versión. `index --full` sirve para
indexar. Para migrar el formato, sigue la guía de actualización. Si hay una
marca de actualización pendiente, termínala o recupérala antes de crear un
índice.

<span id="indexing-is-busy-incomplete-or-shared-pending"></span>

## La indexación está ocupada, incompleta o pendiente de compartir

- **Ocupada** (`index-busy`, «another Mash operation is updating this local
  index»). Otra operación está escribiendo en el índice. Espera a que
  termine o detén tu proceso `mash index --watch` y vuelve a intentarlo.
  Conserva los archivos de bloqueo.
- **Incompleta**. Lee el error del origen, corrígelo y repite `mash index`.
  Mash conserva el historial ya guardado.
- **Pendiente de compartir**. Es posible que la indexación local haya
  terminado bien. Comprueba la conexión, la configuración del token y
  `mash status --shared` y ejecuta `mash index` de nuevo.

Si una búsqueda entre varios equipos devuelve resultados locales con
`shared_error`, usa `--local` para limitar la búsqueda a este equipo o
repara la conexión compartida antes de basarte en los resultados de toda la red.

<span id="get-help"></span>

## Pide ayuda

[Abre un informe de error](https://github.com/grep-stack/mash/issues/new?template=bug_report.yml)
si tienes problemas al instalar, configurar o usar Mash. Indica tu sistema
operativo, cómo intentaste instalar Mash, qué paso falló y qué esperabas que
ocurriera. Si `mash --version` funciona, incluye el resultado. Si Mash no
llegó a instalarse, basta con el error del instalador para abrir un informe.

Los informes de GitHub son públicos. Comparte solo el error pertinente o un
fragmento breve de [`mash doctor`](#check-what-mash-sees). Elimina rutas y
direcciones privadas, texto de conversaciones y credenciales. Si necesitas
un ejemplo, usa una conversación inventada.

Si crees que Mash expuso un historial privado, permitió saltarse el inicio de
sesión o publicó un paquete con una suma de comprobación incorrecta, usa el
[formulario privado de GitHub](https://github.com/grep-stack/mash/security/advisories/new).
Consulta [qué incluir al informar de un problema de seguridad](../../SECURITY.md#report-a-vulnerability) (en inglés).

<!-- mash-docs-source: a73203b99483a5a34c28c3271033920105964cfdc7a39ddc01fb5d725de247fb -->
