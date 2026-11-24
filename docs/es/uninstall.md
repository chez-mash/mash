# Desinstala Mash

Mash conserva las conversaciones originales de tus agentes. La
desinstalación puede borrar la copia que Mash creó para buscar, incluidas
las memorias guardadas.

| Qué quieres hacer | Por dónde empezar |
| --- | --- |
| Quitar Mash y su copia local de tus conversaciones | [Ver la vista previa](#remove-mash-and-its-local-copy) |
| Reinstalar en el mismo equipo y conservar las conversaciones indexadas y las memorias | [Conservar la copia de Mash](#keep-history-for-a-reinstall) |
| Guardar una copia aparte para restaurarla en este equipo o en otro | [Hacer antes una copia de seguridad portátil](backup.md#make-a-portable-backup) |

<span id="remove-mash-and-its-local-copy"></span>

## Quita Mash y su copia local

Empieza por la vista previa. Este comando enumera lo que se quitaría sin
hacer cambios.

```sh
mash uninstall
```

Revisa qué se quitaría y qué se conservaría. La vista previa incluye el
tamaño de la copia de tus conversaciones. Si el informe aparece en JSON,
consulta `paths`, `kept` y `next`.

Cuando quieras continuar, cierra los paneles o procesos de actualización
del índice que hayas iniciado en una terminal y confirma.

```sh
mash uninstall --yes
```

Esto borra el índice, la configuración, los tokens guardados y los modelos
descargados. Mash detiene y retira los servicios que configuró, incluido
el servicio de búsqueda con sus datos. También retira sus herramientas de
búsqueda y su complemento de los [agentes conectados](agents.md#disconnect-an-agent)
con los comandos de cada agente.

Reinicia los agentes conectados que sigan abiertos. Si el informe indica
que falló un paso, corrige el problema y vuelve a ejecutar el comando con
las mismas opciones. Cuando termine, sigue la última instrucción para quitar
el programa. En JSON, consulta `binary.remove`. Según cómo instalaste Mash,
puede indicar `brew uninstall mash`, `scoop uninstall mash`,
`cargo uninstall mash` o qué archivos borrar.

Si instalaste el paquete `mash-cli` con apt, ejecuta `sudo apt remove mash-cli`.
El informe de desinstalación de Mash `v0.1.1-alpha` puede mostrar un nombre de paquete incorrecto.

## Copias que se conservan

- Las conversaciones originales de tus agentes y los archivos ajenos a Mash
  en `~/.mash` se quedan en tu equipo.
- Si publicaste conversaciones en un servidor compartido, su copia sigue allí.
  Para borrarla y revocar el acceso de este equipo, consulta
  [cómo retirar un equipo del índice compartido](shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again).
- Si gestionas tu propio Qdrant, administra el servicio y sus datos de
  búsqueda de Mash por separado.

Para quitar solo el servicio de búsqueda instalado por Mash, usa
`mash qdrant uninstall`. Conserva los datos de búsqueda salvo si pasas
`--delete-storage`. Puedes reconstruirlos desde el índice local con
`mash index --rebuild-local-views --full`.

<span id="keep-history-for-a-reinstall"></span>

## Conserva el historial para reinstalar

Usa `--keep-history` si piensas volver al mismo equipo y quieres conservar
las conversaciones indexadas y las memorias. Mash deja en su sitio el
índice, la configuración, los tokens guardados y los datos del servicio de
búsqueda que gestiona. Retira los servicios, las conexiones a agentes y los
modelos descargados.

Usa la opción tanto en la vista previa como al confirmar.

```sh
mash uninstall --keep-history
```

En `v0.1.1-alpha`, la vista previa puede omitir `--keep-history` en el comando
de confirmación sugerido. Usa el comando completo de abajo. Si confirmas solo
con `--yes`, Mash elimina las conversaciones indexadas y los recuerdos guardados.

```sh
mash uninstall --yes --keep-history
```

Antes de quitar el programa, ejecuta `mash --version` y anota el número de
versión para la reinstalación. Sigue los pasos de
[la desinstalación](#remove-mash-and-its-local-copy) para reintentar los pasos
que fallen, reiniciar los agentes y quitar el programa.

<span id="reopen-the-retained-history"></span>

### Vuelve a abrir el historial conservado

1. En el mismo equipo y con la misma cuenta de usuario, instala la versión
   de Mash que anotaste. Asigna su etiqueta, como `v0.1.1-alpha`, a
   `MASH_VERSION` antes de ejecutar el
   [instalador básico](installation.md#recommended-release-installers).
   Si prefieres instalar desde un archivo, abre esa etiqueta en
   [GitHub Releases](https://github.com/grep-stack/mash/releases) y descarga
   el archivo para tu sistema y el fichero `SHA256SUMS` de esa misma versión.
   [Verifica el archivo](downloads.md#verify-the-checksum) antes de extraerlo.
   Para instalar una versión más reciente, consulta la
   [guía de actualización](upgrades.md#upgrade-an-existing-installation).
2. Si Mash instaló el servicio de búsqueda, ejecuta `mash qdrant install`
   para reinstalarlo y reutilizar sus datos. Si [gestionabas Qdrant por tu cuenta](local-search.md#run-qdrant-yourself),
   inícialo del mismo modo que antes.
3. Ejecuta `mash dash` y abre la dirección que aparece, o usa
   `mash list-chats --local --limit 5` para comprobar en la terminal las
   conversaciones guardadas. Mash usa la configuración y el índice
   conservados. Si usabas rutas personalizadas, vuelve a indicarlas.

Si la búsqueda necesita actualizarse, consulta [cómo mantener el historial al día](synchronization.md).
Si usabas la búsqueda por significado, [vuelve a instalar sus modelos](indexing.md#install-the-models).
También puedes [volver a conectar un agente](agents.md#connect-claude-code-codex-or-copilot).

<!-- mash-docs-source: 78c676e9d7c8bc968b81510a879cb7d593ba3e7d150d2620a927882b98f7d57e -->
