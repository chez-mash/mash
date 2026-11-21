# Guía de inicio rápido

Mash busca en las conversaciones que tus agentes de programación ya guardaron
en este equipo. Es de [código abierto](https://github.com/mashium/mash) y se
ejecuta en tu propio equipo. Puedes
[probar la demo con conversaciones de ejemplo](https://getmash.io/demo/)
antes de instalarlo.

[Privacidad del historial](privacy.md) · [Quick start in English](../../README.md#quick-start)

## Pide ayuda a tu agente

Pega esta instrucción en Claude Code, Codex u otro agente de programación.

> Lee https://getmash.io/setup.md y configura Mash en este equipo. Explícame
> en español qué va a cambiar antes de empezar y muéstrame un resultado de
> mi historial cuando termines.

Si ya instalaste el [plugin de Mash](../plugin.md#start-with-the-plugin),
pídele a tu agente que use la skill `setup`.

## Instálalo por tu cuenta

Necesitas al menos una conversación guardada con un agente compatible.

En un terminal de macOS con Apple silicon o de Linux (x86-64 o ARM64), ejecuta
este comando para una instalación local nueva.

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

En Windows x64, usa PowerShell.

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

El instalador verifica las descargas, guarda Mash en `.local/bin`, inicia el
servicio de búsqueda local (Qdrant, unos 30 MB) y abre el asistente de Mash.
El servicio funciona con tu usuario y se inicia cuando entras en tu sesión.
En Linux y WSL, esta opción requiere una sesión de usuario de systemd.
La búsqueda por palabras funciona sin descargar modelos.

Revisa las carpetas de historial y los agentes detectados. Elige
**Set up and index** para crear el índice en este equipo. La configuración
predeterminada incluye archivos actualizados en los últimos 90 días.
Si el asistente te pregunta antes dónde buscar, elige **Just this computer**.
Para elegir carpetas, fechas o agentes, usa **Customize setup** si está
disponible.

Para instalar por separado, usar tu propio Qdrant o configurar WSL sin systemd,
consulta la [guía de instalación (en inglés)](../installation.md).
También puedes [usar el asistente del panel local (en inglés)](../dashboard.md#set-up-mash-from-the-dashboard).

## Encuentra una conversación anterior

Sustituye `migración` por una palabra, un código de error o un nombre de
función que recuerdes de una conversación anterior.

```sh
mash search migración
```

Si `mash` aún no está en tu `PATH`, usa la ruta que muestra el instalador.
Para abrir el mensaje, pasa el `lookup.view.id` del resultado a
`mash view MESSAGE_ID --context 3`. En el panel, busca el mismo término y
abre el resultado.

Reinicia los agentes que conectaste. Por ejemplo, puedes pedirle a Codex
«Usa Mash para buscar qué decidimos sobre la autenticación en Claude Code y
muéstrame el mensaje original».

Los menús de Mash están en inglés. La búsqueda por significado usa modelos
opcionales y, por ahora, requiere consultas en inglés.
[Cómo funciona la búsqueda (en inglés)](../search.md)
