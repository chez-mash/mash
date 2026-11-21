# Agentes compatibles

Busca tus conversaciones guardadas en el [panel web](dashboard.md) con
`mash dash` o desde el [terminal](../search.md). Ambas opciones funcionan
con todos los historiales de la tabla.

También puedes conectar un agente para pedirle que busque trabajos anteriores.
La conexión es opcional. Un agente conectado puede buscar en todos los
historiales que hayas indexado, incluidas las conversaciones con otros agentes.
[Instrucciones de conexión](agents.md#connect-claude-code-codex-or-copilot)

[Configura Mash y encuentra tu primera conversación](quick-start.md).

| Agente | Conversaciones guardadas que Mash puede leer | Conexión con `mash init` |
| --- | --- | --- |
| Claude Code | Conversaciones locales de Claude Code | Sí |
| Codex | Conversaciones locales de Codex | Sí, mediante la CLI de Codex |
| GitHub Copilot CLI | Sesiones locales de la línea de comandos | Sí |
| GitHub Copilot Chat en VS Code | Conversaciones que VS Code guarda en tu equipo | — |
| OpenCode | Conversaciones de la base de datos local de OpenCode | — |
| Cursor | Conversaciones de la línea de comandos y formatos compatibles del editor | — |
| Antigravity | Conversaciones guardadas en las bases de datos locales de la aplicación y del IDE | — |

Si tu cliente admite servidores MCP locales por entrada y salida estándar,
consulta [cómo conectarlo manualmente (en inglés)](../mcp.md#connect-another-mcp-client).

## Si falta una conversación

Ejecuta `mash init` para revisar qué agentes encontró Mash y qué historiales
leerá. Si ya configuraste Mash, elige **Customize setup** para cambiar la
selección. Si guardas un historial en otra ubicación, añádela con la
[guía de configuración (en inglés)](../configuration.md#choose-histories-and-an-age-limit).

Ejecuta `mash index` para leer las conversaciones nuevas. De forma
predeterminada, Mash empieza por los últimos 90 días. Consulta
[cómo incluir conversaciones más antiguas](../indexing.md#add-older-history)
y [dónde se guardan las sesiones remotas de VS Code](../indexing.md#copilot-in-vs-code-remote-windows).
Ambas guías están en inglés.

<span id="tested-with"></span>

## Pruebas por sistema operativo

Así se han probado hasta ahora la indexación y la búsqueda de estos historiales.

| Agente | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Claude Code | Uso diario | Uso diario | Uso limitado |
| Codex | Uso diario | Uso diario | Uso limitado |
| GitHub Copilot | Uso limitado | Uso limitado | Solo registros de prueba |
| OpenCode | Solo registros de prueba | Uso limitado | Simulación en CI |
| Antigravity | Solo registros de prueba | Uso limitado | Solo registros de prueba |
| Cursor | Solo registros de prueba | Parcial | Solo registros de prueba |

- **Uso diario** abarca cientos de conversaciones reales de un agente instalado en ese sistema.
- **Uso limitado** abarca menos de veinte conversaciones reales de un agente instalado.
- **Simulación en CI** indica que una prueba automatizada escribe el historial
  del mismo modo que el agente mientras Mash lo indexa.
- **Parcial** indica que Mash leyó conversaciones de una instalación real de
  Cursor CLI. Algunos formatos antiguos del editor son incompatibles.
- **Solo registros de prueba** indica que las pruebas usan registros con el formato
  del agente. Sigue pendiente verificar una instalación real en ese sistema.

Las pruebas de [Linux](https://github.com/mashium/mash/actions/workflows/ci.yml)
y [Windows](https://github.com/mashium/mash/actions/workflows/windows.yml) se
ejecutan con cada envío de cambios. La
[compilación y las comprobaciones básicas en Apple Silicon](https://github.com/mashium/mash/actions/workflows/macos-build.yml)
se ejecutan bajo demanda. Si faltan conversaciones o se muestran mal, puedes
[abrir una incidencia](https://github.com/mashium/mash/issues) con el nombre
del agente, la versión y el sistema operativo.

<!-- mash-docs-source: cae257f3a057c87cef874197ea58c49a155db99c696e9c52437110b8f9765c1c -->
