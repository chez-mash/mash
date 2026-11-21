# Privacidad del historial

[Privacy in English](../privacy.md) · [Guía de inicio rápido](quick-start.md)

## Qué se queda en tu equipo

Con la configuración predeterminada, Mash lee e indexa el historial y ejecuta
las búsquedas en tu equipo. No modifica el contenido de los archivos de
historial de tus agentes. No necesitas una cuenta de Mash y el sitio público
de documentación no recibe tus conversaciones indexadas. La
[demo pública](https://getmash.io/demo/) usa conversaciones de ejemplo y
no accede a tu historial local.

Si usas el panel local, conserva su dirección predeterminada o limita el
acceso a una red privada de confianza. El panel local no tiene inicio de
sesión.

## Cuándo puede salir parte del historial

- **Agentes conectados.** Cuando un agente usa Mash, recibe resultados y
  fragmentos de las conversaciones. Esos fragmentos pasan al contexto del
  agente y pueden enviarse a su proveedor de modelos. Para buscar por tu
  cuenta, puedes usar el panel local o la terminal.
- **Historial compartido.** Si habilitas la búsqueda entre equipos, Mash
  envía el historial al servidor que configures y alojes.
- **Servicios remotos opcionales.** Si configuras un servicio remoto para
  preparar el índice o reordenar resultados, ese servicio puede recibir
  texto de las conversaciones. Si habilitas el asistente del panel, enviará
  tu pregunta y los fragmentos que consulte al servicio de modelos que
  configures.
- **Recuperar recuerdos de chats anteriores.** Si eliges esta opción en la
  configuración inicial, los chats seleccionados se envían al proveedor del
  agente que elijas.

## Prueba con menos historial

Durante la configuración, elige el historial de un solo agente y un intervalo
corto. Puedes dejar desactivadas las conexiones con los agentes y buscar por
tu cuenta en el panel local o en la terminal. Antes de instalar, puedes revisar
[los instaladores y las sumas de comprobación (en inglés)](../installation.md#check-the-release-download).

`mash uninstall` muestra qué se eliminaría antes de confirmar y conserva las
conversaciones originales de tus agentes.
[Cómo desinstalar Mash (en inglés)](../installation.md#uninstall)

## Credenciales y copias

Mash oculta las credenciales que reconoce en los resultados y al compartir
el historial. La detección puede pasar por alto datos sensibles. Revisa lo
que vayas a compartir y guarda las copias de seguridad con el mismo cuidado
que las conversaciones originales.

Para quitar un chat del índice, consulta la
[guía completa de privacidad (en inglés)](../privacy.md#remove-a-chat).
