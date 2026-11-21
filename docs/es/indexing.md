# Incluye conversaciones antiguas y busca por significado

<span id="add-older-history"></span>

## Incluye más historial

En una instalación nueva, Mash lee el historial que tus agentes actualizaron
en los últimos 90 días. Una conversación actualizada hace poco puede contener
mensajes más antiguos. Para incluir conversaciones anteriores que siguen en disco:

```sh
mash index --all-history
```

Si prefieres retroceder un número fijo de días:

```sh
mash index --since-days 365
```

Para actualizar el historial de un solo agente:

```sh
mash index --provider codex
```

Los nombres de proveedor son `claude-code`, `codex`, `opencode`, `copilot`,
`cursor` y `antigravity`.

El servicio de búsqueda local (Qdrant) debe estar en marcha para indexar.
Mash lee los archivos originales del historial sin modificarlos. Comprueba
si el resultado final del comando indica algún error.

Estas opciones se aplican a una sola ejecución. Para cambiar el límite de
forma permanente, consulta la [configuración (en inglés)](../configuration.md).
Para actualizar de forma continua, consulta [cómo mantener el historial al día (en inglés)](../synchronization.md).

<span id="copilot-in-vs-code-remote-windows"></span>

### Copilot en ventanas remotas de VS Code

Indexa cada tipo de sesión en el equipo donde se guarda:

- Las sesiones de **Copilot Chat** de una ventana de Remote-SSH, un contenedor
  o Codespaces se guardan en el equipo que ejecuta VS Code. Su proyecto es la
  carpeta del equipo remoto.
- Las sesiones de **Copilot CLI** abiertas desde la vista de chat de una ventana
  remota se ejecutan en el equipo remoto y se guardan en su `~/.copilot/session-state`.

Si Copilot sigue escribiendo una sesión mientras Mash la lee, Mash conserva
los mensajes que ya había indexado y vuelve a leer la sesión en la próxima actualización.

<span id="enable-semantic-search"></span>

## Activa la búsqueda por significado

La búsqueda por significado, también llamada búsqueda semántica, te permite
hacer una pregunta sin recordar las palabras exactas. Escribe estas consultas
en inglés. La búsqueda por palabras o frases funciona sin modelos y sigue
siendo la predeterminada hasta que cambies la configuración. Para conversaciones
en otros idiomas, consulta los [ejemplos de búsqueda por palabras](search-languages.md).

<span id="install-the-models"></span>

### Instala los modelos

```sh
mash models install
```

Descarga unos 177 MB de `https://huggingface.co` a `~/.mash/models`. En Apple
silicon y Windows, también descarga el modelo de embeddings sin cuantizar para
la GPU, unos 90 MB más. Configura `HF_ENDPOINT` para usar un servidor espejo de
Hugging Face o pasa `--models-dir PATH` para elegir otro directorio.

<span id="build-search-data"></span>

### Prepara el índice para buscar por significado

Si estás configurando Mash por primera vez, ejecuta `mash init` después de
instalar los modelos. Cuando crea la configuración, Mash activa la indexación
por significado y prepara los datos en la primera pasada. [Configuración inicial](installation.md#connect-your-histories)

Si ya usas Mash, modifica la tabla `[views.build]` de tu
[configuración (en inglés)](../configuration.md). Abre **settings** en el
panel o busca `configuration.path` en el resultado de `mash status` para
encontrar el archivo. Conserva tus otros ajustes y añade la tabla solo si falta.

```toml
[views.build]
representations = "lexical-and-dense"
```

Ejecuta `mash index`. La primera pasada tarda más porque Mash prepara tu
historial para buscar por significado. En las siguientes reutiliza el trabajo
de las conversaciones que siguen iguales.

Ejecuta `mash status` después. El campo `realized_mode` muestra `hybrid`
cuando puedes buscar por palabras y por significado en ese equipo.

<span id="try-a-question"></span>

### Prueba una pregunta

```sh
mash search 'why did we choose a separate database' --intent related-discussion
```

<span id="use-search-by-meaning-by-default"></span>

### Usa la búsqueda por significado de forma predeterminada

Para usarla en el panel y en las búsquedas sin un modo explícito, configura
`intent` en la tabla `[views.query]`. Añade la tabla si falta. Una vez
preparado el índice, el nuevo modo se aplica sin volver a indexar.

```toml
[views.query]
intent = "related-discussion"
```

Si quieres hacer una búsqueda por palabras con ese modo predeterminado,
pasa `--intent exact-mention`.

```sh
mash search 'auth_token' --intent exact-mention
```

Si algo falla, consulta la [guía de solución de problemas](troubleshooting.md).

<!-- mash-docs-source: 6724bc90be8c2bf0b6959aaeb8d9bf61cc977de3fcd373c007efa07572341d58 -->
