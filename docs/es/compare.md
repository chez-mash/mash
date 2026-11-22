# ¿Es Mash para ti?

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
  [Cómo guarda Mash los registros](records.md#how-saving-works)
- **Trabajas en varios equipos.** Cada equipo puede buscar en su propio
  índice sin conexión. También puedes publicar tu historial en un servidor
  Mash que tú administras para buscar desde un portátil las conversaciones
  de una estación de trabajo. [Cómo se comparte el historial](shared-index.md)

Mash usa un servicio de búsqueda local llamado Qdrant. El instalador con
configuración automática puede iniciarlo. La búsqueda por palabras funciona
sin una cuenta de Mash, sin descargar modelos y sin conectar agentes.
Puedes añadir modelos locales para buscar por significado y conectar un
agente cuando quieras. [Instalación](installation.md) · [Privacidad](privacy.md) ·
[Búsqueda por significado](indexing.md#enable-semantic-search)

[Instala Mash y encuentra tu primera conversación](quick-start.md).

<!-- mash-docs-source: 0eb4bc99fad7c3d96d03dc79b728b346692efb0c9dfc92e68ee20e9ca4c5d290 -->
