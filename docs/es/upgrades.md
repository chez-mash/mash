# Actualiza Mash

Al actualizar Mash se conservan la configuración y las conversaciones
indexadas. Consulta las [notas de la versión (en inglés)](https://github.com/grep-stack/mash/releases),
haz una copia de seguridad y usa tu método habitual de instalación. Si solo
quieres guardar o recuperar una conversación, consulta
[cómo hacer una copia portátil](backup.md#make-a-portable-backup) o
[cómo restaurar una copia por separado](backup.md#restore-without-replacing-your-current-history).

<span id="upgrade-an-existing-installation"></span>

## Actualiza una instalación existente

Si publicas el historial de varios equipos, lee
[los pasos para el índice compartido](#upgrade-a-shared-index) antes de actualizar.

1. **Pausa la indexación y cierra los agentes conectados.** Detén los procesos
   de `mash index --watch` y deja que termine cualquier indexación en curso.
   Si abriste el panel desde una terminal, ciérralo con Ctrl-C.
2. **Haz una copia con tu versión actual.** Conserva su ejecutable o el
   archivo de la versión hasta que funcione la nueva. Exporta a un directorio
   nuevo que puedas mantener en privado.

   ```sh
   mash --version
   mash export "/absolute/path/to/new-backup"
   ```

3. **[Instala la versión nueva](#install-the-new-version)** por el método
   habitual. Ejecuta `mash --version` otra vez para comprobar que tu terminal
   encuentra el ejecutable actualizado.
4. **Actualiza Qdrant si la versión lo requiere.** Vuelve a ejecutar
   `mash qdrant install` para actualizar el servicio que instaló Mash sin
   cambiar sus datos. Si administras Qdrant, actualízalo en el mismo espacio
   de almacenamiento.
5. **Actualiza el índice.** Si el formato de la base está dentro del rango
   admitido pero necesita migrar, este comando lo actualiza antes de indexar.
   Los comandos de solo lectura te indican si hace falta migrar.

   ```sh
   mash index
   ```

6. **Comprueba la búsqueda y vuelve a abrir Mash.** Verifica que
   `mash status` indique `"ready": true` y busca una palabra de una
   conversación indexada. Sustituye el texto de ejemplo por una palabra
   que recuerdes.

   ```sh
   mash status
   mash search 'a word you remember' --local --intent exact-mention
   ```

   Reinicia los agentes conectados y los procesos de actualización que
   detuviste. Abre el panel local con `mash dash`. Si usas el
   [panel remoto](dashboard.md#open-it-from-your-phone-or-another-computer),
   vuelve a ejecutar `mash dash remote` con las mismas opciones para iniciar
   el servicio con el nuevo ejecutable.
   Si a un agente le cuesta encontrar Mash después de la actualización,
   [comprueba el complemento y la conexión](agents.md#confirm-the-connection).

Mash conserva los identificadores de chats y mensajes y reutiliza los
datos de búsqueda compatibles. La indexación también incorpora los
historiales guardados desde la última ejecución.

<span id="install-the-new-version"></span>

### Instala la versión nueva

Usa tu método habitual. Si usaste el script de shell o de PowerShell,
vuelve a ejecutar el [instalador básico](installation.md#recommended-release-installers)
con el mismo directorio de instalación. Si configuraste `MASH_VERSION`,
cámbialo a la versión que quieras.

| Método de instalación | Cómo actualizar |
| --- | --- |
| Homebrew | `brew update && brew upgrade grep-stack/tap/mash` |
| apt | `sudo apt update && sudo apt install --only-upgrade mash-cli` |
| Scoop | Ejecuta `scoop update` y después `scoop update mash`. |
| mise | `mise upgrade github:grep-stack/mash` |
| cargo-binstall | Repite la [instalación del archivo precompilado](install-methods.md#cargo-binstall). |
| Código fuente | [Compila la revisión que quieras](install-methods.md#build-from-source) e incluye los archivos web si usas el panel. |

Si instalaste el primer paquete de Debian con el nombre `mash`, sigue los
[pasos de apt](install-methods.md#apt-debian-or-ubuntu) para cambiar a
`mash-cli`. Si fijaste una versión exacta con mise, cambia esa versión.

Si `mash --version` sigue mostrando la versión anterior, comprueba qué
ejecutable encuentra la terminal con `command -v mash` en macOS o Linux, o
`(Get-Command mash).Source` en PowerShell. Usa la ruta del ejecutable
actualizado para las comprobaciones anteriores.

<span id="if-the-index-format-changes"></span>

### Si cambia el formato del índice

`mash --version-info` muestra el formato de índice que escribe el ejecutable
nuevo (`silver_schema`) y el más antiguo que puede actualizar
(`silver_schema_min`). Si `mash index` indica un formato anterior a ese
rango, consulta [cómo sustituir un índice incompatible](#replace-an-index-that-cannot-be-upgraded).
Si indica un formato más reciente, usa la versión que lo escribió u otra
posterior compatible. Reinicia los agentes, paneles o procesos de
actualización que sigan usando la versión anterior.

**La migración elimina las preguntas abiertas guardadas.** Si el índice
se creó antes de que Mash dejara de guardar preguntas como registros, la
actualización borra esas preguntas, sus revisiones y los enlaces que las
resolvían. Las decisiones, preferencias y hechos siguen disponibles. La
copia exportada conserva las preguntas. Guarda la versión que hizo la
copia si necesitas leerlas más adelante.

<span id="upgrade-a-shared-index"></span>

### Actualiza un índice compartido

Si [compartes el historial entre equipos](shared-index.md), actualiza el
servidor y todos los equipos a la misma versión. Las versiones distintas
rechazan la sincronización. Conserva los datos de búsqueda para cuando
todos los equipos usen la misma versión.

Haz una copia de los datos y la configuración del servidor antes de
actualizarlo. `mash export` en un equipo guarda el índice de ese equipo.
En el servidor, actualiza Qdrant si hace falta e inicia el nuevo
`mash-server`. Actualiza las bases de datos de los equipos al abrirlas.
Reanuda la indexación compartida cuando el servidor y los equipos
que publican usen la misma versión.

<span id="make-a-portable-backup"></span>

## Haz una copia de seguridad portátil

[Exporta las conversaciones indexadas a un directorio privado](backup.md#make-a-portable-backup).

<span id="restore-without-replacing-your-current-history"></span>

## Restaura sin sustituir el historial actual

[Comprueba una copia restaurada en una base de datos independiente](backup.md#restore-without-replacing-your-current-history).

<span id="replace-an-index-that-cannot-be-upgraded"></span>

## Sustituye un índice que no se puede actualizar

Sigue estos pasos si Mash indica que el índice es demasiado antiguo.
La indexación normal no puede actualizarlo. Conserva el ejecutable y el
índice anteriores hasta comprobar que funciona el reemplazo.

1. Exporta con la versión que escribió el índice e importa la copia con
   la nueva versión en una ruta aparte. Añade `--db PATH` al exportar si el
   índice anterior está en una ruta personalizada.

   ```sh
   /path/to/old/mash export "/absolute/path/to/new-backup"
   /path/to/new/mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/staged.sqlite3" --keep-identity
   /path/to/new/mash --db "/absolute/path/to/staged.sqlite3" list-chats --local --limit 5
   ```

   Algunas copias antiguas tampoco se pueden importar. Si falla la
   importación, conserva los datos originales y el ejecutable.
2. `--keep-identity` conserva la identidad del equipo en el índice
   compartido. Detén el panel, el proceso de actualización y las conexiones
   anteriores antes de cambiar de índice. Solo el reemplazo debe publicar
   después del cambio.
3. Elige la ruta definitiva. Puedes dejar el reemplazo donde lo importaste.
   Para usar la ruta original, mueve primero a un sitio seguro la base de
   datos anterior, su carpeta `.attachments` y los archivos `-wal`, `-shm`
   o `-journal` que tenga al lado. Coloca juntos la base importada y sus
   archivos. Cambia el nombre de `staged.sqlite3.attachments` y de los
   archivos auxiliares de SQLite para que coincidan con el de la nueva base.
4. Construye los datos de búsqueda en la ruta definitiva y comprueba
   que puedas buscar en las conversaciones. Usa la ruta que elegiste.

   ```sh
   mash --db "/absolute/path/to/replacement.sqlite3" index --rebuild-local-views --full
   mash --db "/absolute/path/to/replacement.sqlite3" status
   mash --db "/absolute/path/to/replacement.sqlite3" search 'a word you remember' --local --intent exact-mention
   ```

5. Si usas una ruta personalizada, sigue pasando `--db`, también en
   la configuración de MCP de tus agentes. Reinicia los agentes y los
   procesos de actualización automática.

<span id="other-changes-you-may-meet"></span>

## Otros cambios que puedes encontrar

<span id="coming-from-qdrant-119"></span>

### Si venías de Qdrant 1.19

Mash requiere Qdrant 1.20 o posterior. Antes de ejecutar `mash index` con
el Mash nuevo, detén el servicio 1.19 e inicia 1.20 con los mismos datos.
La primera indexación añade un índice de palabras a los datos de búsqueda
existentes sin volver a crearlos ni volver a calcular sus representaciones
semánticas. Hasta entonces, la búsqueda avisará de que los datos de
búsqueda necesitan actualizarse. En un servidor compartido, el nuevo
`mash-server` añade el índice al iniciarse.

**No vuelvas a abrir con Qdrant 1.19 datos que ya abrió 1.20.** Haz una
copia primero si podrías necesitar volver a la versión anterior.

<span id="settings-that-are-no-longer-accepted"></span>

### Ajustes que ya no se admiten

- Quita `views.query.experimental_gpu_reranking`. El reordenamiento se
  hace en la CPU y puedes conservar el índice y los modelos.
- Sustituye los ajustes antiguos de proveedores remotos en `[embeddings]`
  por los descritos en [cálculo de representaciones (en inglés)](../configuration.md#embedding-inference).
- Quita `redaction.remote_embeddings` de la configuración del equipo y del servidor.

<span id="changing-the-embedding-model"></span>

### Cambia el modelo de búsqueda por significado

Las representaciones de tu modelo local siguen sirviendo al actualizar.
Si cambias de modelo, ejecuta `mash models install` y vuelve a construir
los datos de búsqueda locales.

```sh
mash index --rebuild-local-views --full
```

Para un índice compartido, cambia el servidor y todos los equipos a la vez.
Instala los modelos en el servidor e inícialo una vez con este comando.

```sh
mash-server --server-config PATH --models-dir PATH --activate-views --replay-views
```

El servidor reconstruye los datos de búsqueda con el historial que ya
guarda. Haz una copia primero. La búsqueda compartida estará temporalmente
fuera de servicio durante la reconstrucción.

<!-- mash-docs-source: f57dfe0421444d2aa8f7089a5af074b78ac73ec214071488c564847c13fb909c -->
