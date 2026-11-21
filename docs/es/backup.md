# Haz una copia y restaura tu historial

- **Guarda una copia separada que puedas restaurar.** Usa [`mash export`](#make-a-portable-backup)
  antes de actualizar o desinstalar Mash, o para pasar el historial a otro equipo.
- **Guarda una transcripción para leerla o compartirla.** Usa la
  [exportación del panel web](dashboard.md#export-many-chats-at-once) para obtener
  archivos que puedes abrir fuera de Mash.
- **Reinstala más adelante en el mismo equipo.** Usa
  [`mash uninstall --keep-history`](uninstall.md#keep-history-for-a-reinstall) para dejar en su
  sitio la copia consultable y la configuración. Si necesitas una copia separada,
  haz una copia de seguridad.

<span id="make-a-portable-backup"></span>

## Haz una copia de seguridad portable

Elige un directorio nuevo en un lugar privado y separado de
la [carpeta de datos de Mash](privacy.md#where-mash-stores-data). Sustituye las
rutas de ejemplo por rutas absolutas de tu equipo. Conserva las comillas si
las rutas contienen espacios.

```sh
mash export "/absolute/path/to/new-backup"
```

El resultado indica cuántas conversaciones, mensajes y archivos adjuntos
conservados se copiaron. Guarda toda la carpeta de la copia de seguridad junta.
Contiene el historial y los recuerdos que figuran en el índice de este equipo.
Si compartes un índice, exporta desde cada equipo cuyo historial quieras guardar.

**Guarda las copias en privado.** Contienen el texto guardado de las
conversaciones, incluidas credenciales que pueden estar ocultas en los resultados
de búsqueda. Guarda por separado la configuración, las credenciales de los
servicios y los archivos de historial originales de tus agentes. Conserva la
versión de Mash que creó la copia si puedes necesitarla para
[actualizar o volver a una versión anterior (en inglés)](../upgrades.md#upgrade-an-existing-installation).

<span id="restore-without-replacing-your-current-history"></span>

## Restaura una copia sin sustituir tu historial actual

Elige una ruta nueva para la base de datos. Mash rechaza la importación a una
base de datos existente. Importa la copia y lista unas cuantas conversaciones
para comprobar qué se restauró.

```sh
mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/restored.sqlite3"
mash --db "/absolute/path/to/restored.sqlite3" list-chats --local --limit 5
```

Puedes exportar, importar y listar conversaciones locales sin el servicio de
búsqueda. Si la importación indica que el archivo es incompatible, conserva la
copia y usa la versión de Mash que la creó. Para sustituir un índice y seguir
publicando con la identidad del mismo equipo, sigue el
[procedimiento de actualización (en inglés)](../upgrades.md#replace-an-index-that-cannot-be-upgraded).

<span id="make-the-restored-chats-searchable"></span>

## Busca en las conversaciones restauradas

Inicia el [servicio de búsqueda local](installation.md#start-the-local-search-service)
y prepara los datos de búsqueda de la copia restaurada. Busca una palabra que
recuerdes de una de las conversaciones de la lista.

```sh
mash --db "/absolute/path/to/restored.sqlite3" index --rebuild-local-views --full
mash --db "/absolute/path/to/restored.sqlite3" search 'a word you remember' --local --intent exact-mention
```

Pasa `--db` cada vez que quieras usar la copia restaurada. Si tu configuración
o tus modelos están en otro lugar, pasa también `--config PATH` o `--models-dir PATH`.

<!-- mash-docs-source: 6e60fff6c2bbe05cb0dcefc8bc1c14ffc6d70d8d220650fe25fc9d0cc762e945 -->
