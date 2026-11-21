# Compara las herramientas de historial de agentes de código

Mash te ayuda a encontrar las conversaciones que tus agentes ya guardaron y
abrir el mensaje original que explica una decisión. Puedes buscar en el
navegador, desde la terminal o a través de un agente conectado.

En la demo, [busca conversaciones de ejemplo sobre la exportación de facturas](https://getmash.io/demo/#/search?q=invoice+exports+queue)
y abre el resultado de la estación de trabajo para leer la conversación.
Después, abre la [decisión guardada sobre el inicio de sesión](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
y elige **open the message** para ver de dónde salió.

<span id="when-mash-fits"></span>

## Cuándo elegir Mash

- **Ya tienes conversaciones que vale la pena conservar.** Mash lee las
  conversaciones que los agentes compatibles guardaron en disco. Busca una
  palabra o frase y abre el resultado en su conversación para ver el diálogo
  y las salidas de herramientas. Usa el panel web, la terminal o un agente
  que decidas conectar. [Agentes compatibles](supported-agents.md)
- **Quieres comprobar en qué se basa una decisión guardada.** Con el
  complemento, tu agente puede guardar decisiones y preferencias con una cita
  verificada en la conversación. El panel abre el mensaje citado en contexto.
  [Cómo guarda Mash los registros (en inglés)](../records.md#how-saving-works)
- **Trabajas en varios equipos.** Cada equipo puede buscar en su propio
  índice sin conexión. También puedes publicar tu historial en un servidor
  Mash que tú administras para buscar desde un portátil las conversaciones
  de una estación de trabajo. [Cómo se comparte el historial](shared-index.md)

Mash usa un servicio de búsqueda local llamado Qdrant. El instalador con
configuración automática puede iniciarlo. La búsqueda por palabras funciona
sin una cuenta de Mash, sin descargar modelos y sin conectar agentes.
Puedes añadir modelos locales para buscar por significado y conectar un
agente cuando quieras. [Instalación](installation.md) · [Privacidad](privacy.md) ·
[Búsqueda por significado (en inglés)](../indexing.md#enable-semantic-search)

<span id="other-workflows"></span>

## Otras formas de trabajar

| Si quieres… | Prueba | Qué ofrece |
| --- | --- | --- |
| Buscar y retomar sesiones, o recuperar un cambio en un archivo | Déjà Vu | Un solo ejecutable indexa el historial guardado y permite buscar en él de forma local. Además, permite pasar trabajo a otro agente y recuperar texto de cambios registrados. [README de Déjà Vu](https://github.com/vshulcz/deja-vu#overview) |
| Buscar en los historiales de muchos agentes con una interfaz de teclado | CASS | Ofrece una búsqueda interactiva en la terminal y resultados estructurados para scripts o agentes. [Repositorio de CASS](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| Llevar resúmenes automáticos del trabajo a nuevas sesiones | Claude-Mem | Sus mecanismos de integración capturan la actividad y un observador la resume. La sesión siguiente puede cargar las notas pertinentes. [Cómo funciona Claude-Mem](https://github.com/thedotmack/claude-mem#how-it-works) |
| Guardar conversaciones como Markdown en tu proyecto o retomarlas en un agente | SpecStory | Sus extensiones y su interfaz de terminal escriben archivos Markdown mientras trabajas. La interfaz de terminal puede volver a abrir sesiones guardadas. [Guardar conversaciones](https://docs.specstory.com/specstory/features) · [Buscar y retomar sesiones](https://docs.specstory.com/integrations/terminal-coding-agents/resume) |

[Instala Mash y encuentra tu primera conversación](quick-start.md).

<!-- mash-docs-source: 93cb840d39d49b21496d27385d3b09d3c74fe1f6c462801f5a43717fbbfbc6c9 -->
