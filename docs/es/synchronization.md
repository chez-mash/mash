# Mantén el historial al día

Una conversación nueva aparece en las búsquedas cuando Mash indexa el
historial que guardó tu agente. Usa [Actualizar ahora](#refresh-now) si buscas
una conversación que acabas de tener. Para actualizar mientras trabajas,
[deja el monitor en marcha](#keep-refreshing-while-you-work).

<span id="refresh-now"></span>

## Actualizar ahora

Ejecuta estos comandos en el equipo donde el agente guardó la conversación.
Si todavía no has elegido qué historiales puede leer Mash,
[termina la configuración](installation.md#connect-your-histories) primero.

```bash
mash index
mash status
```

`mash index` lee el historial nuevo o modificado y muestra el progreso.
Comprueba en el resultado final si quedó alguna lectura incompleta o una
publicación pendiente. Corrige el motivo que indica y repite `mash index`.
`mash status` indica si la búsqueda local está lista.

La lista de conversaciones del panel se actualiza tras la indexación. Si
tienes una búsqueda abierta, ejecútala de nuevo para incluir las conversaciones nuevas.

<span id="keep-refreshing-while-you-work"></span>

## Actualiza mientras trabajas

Ejecuta este comando en otra terminal y déjala abierta:

```bash
mash index --watch
```

Hace una primera indexación y luego vuelve a indexar tras cada grupo de cambios
en el historial. Espera al menos 30 segundos entre pasadas, y puede esperar
más después de una pasada lenta. Ctrl-C deja que termine la pasada en curso y detiene el monitor.
Cerrar la terminal también lo detiene.

Se aplican los agentes elegidos, el intervalo del historial y el destino
local o compartido de tu configuración. Para incluir historial anterior:

```bash
mash index --watch --all-history
```

Para cambiar el intervalo de forma permanente, edita la
[configuración (en inglés)](../configuration.md). `--watch` no se puede combinar
con `--full` ni `--reread`.

Al actualizar Mash, el monitor termina la pasada en curso e inicia la nueva
versión de `mash` con las mismas opciones. La terminal o el servicio puede
seguir ejecutándose. Si no puede iniciar la nueva versión, se cierra con un
error y puedes volver a iniciarlo.

<span id="what-the-agent-plugin-does-automatically"></span>

## Qué hace automáticamente el complemento

El [complemento de Mash](plugin.md) inicia una indexación de baja prioridad
al empezar una sesión, como máximo una vez cada diez minutos. El agente
continúa trabajando enseguida. Si ya hay una indexación en curso, el intento
termina sin aplicar cambios.

El complemento pone al día el historial al iniciar una sesión. Para seguir
actualizando durante una sesión larga, usa el monitor. Para desactivar la
actualización al inicio, configura `MASH_AUTO_INDEX=off` en el entorno del agente.

<span id="understand-freshness-warnings"></span>

## Entiende los avisos de actualización

Puedes actualizar de forma manual tras `mash init` o una primera ejecución
de `mash index`. La búsqueda usa el historial de la última indexación. Si
una versión anterior muestra «Live indexing is inactive», el historial se
actualizará la próxima vez que se ejecute la indexación.

Una búsqueda puede seguir devolviendo conversaciones antiguas si la última
lectura quedó incompleta o Mash no pudo consultar su estado. Sigue el aviso
para comprobar o actualizar el índice. Si falta una conversación reciente,
ejecuta `mash index` y repite la búsqueda.

- Deja el monitor en marcha si necesitas actualizaciones continuas y revisa
  sus resultados para saber si pudo leer todos los orígenes.
- Si el aviso aparece tras una actualización o menciona un índice incompatible,
  sigue la [guía de actualizaciones](upgrades.md) antes de reintentar.

<span id="in-the-dashboard"></span>

## En el panel web

La indexación pone el historial nuevo a disposición del panel. Si estás al
final de una conversación, la vista sigue los mensajes nuevos. Si estás en
otro punto, elige **N new messages** para verlos. Un mensaje anterior se
mantiene como se mostró hasta que eliges **Show updated messages**.
[Cómo se actualiza el panel](dashboard.md#keep-your-place)

<span id="across-machines"></span>

## En varios equipos

Si falta una conversación de otro equipo, comprueba primero aquel equipo.
Es donde se indexa la conversación y se publica la copia compartida. Ejecuta
allí `mash status --shared` para revisar la publicación. Si está pendiente,
restaura la conexión y ejecuta `mash index` allí de nuevo.

[Configuración para varios equipos](shared-index.md)

<!-- mash-docs-source: e4a3daadae69cf3d992691e9b02a63c7d28d80d0afc1f9f8c05f204a1d66dabd -->
