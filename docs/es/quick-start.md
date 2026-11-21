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

## Instálalo por tu cuenta

Necesitas al menos una conversación guardada con un agente compatible.

### 1. Descarga Mash

En macOS con Apple silicon o en Linux (x86-64 o ARM64, incluido WSL), ejecuta
este comando.

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

En Windows x64, usa PowerShell.

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

El instalador verifica la suma de comprobación de la descarga y guarda los
programas en `.local/bin`, dentro de tu carpeta de usuario. Si `mash` aún no
está en tu `PATH`, usa la ruta que muestra el instalador en lugar de `mash`.
[Otros métodos de instalación (en inglés)](../installation.md#package-manager-alternatives)

### 2. Inicia el servicio de búsqueda

```sh
mash qdrant install
```

Mash descarga y verifica Qdrant (unos 30 MB). El servicio se ejecuta con tu
usuario, solo acepta conexiones desde este equipo y se inicia de nuevo
cuando entras en tu sesión. La búsqueda por palabras funciona sin descargar
modelos.

### 3. Revisa qué va a leer Mash

```sh
mash init
```

Revisa las carpetas de historial y los agentes detectados. Elige
**Set up and index** para crear el índice en este equipo. La configuración
predeterminada incluye archivos actualizados en los últimos 90 días.
Si el asistente te pregunta antes dónde buscar, elige **Just this computer**.
Para elegir carpetas, fechas o agentes, usa **Customize setup** si está
disponible.

También puedes ejecutar `mash dash` en lugar de `mash init` y usar
[el asistente del panel local (en inglés)](../dashboard.md#set-up-mash-from-the-dashboard).

## Encuentra una conversación anterior

Sustituye `migración` por una palabra, un código de error o un nombre de
función que recuerdes de una conversación anterior.

```sh
mash search migración
```

Para abrir el mensaje, pasa el `lookup.view.id` del resultado a
`mash view MESSAGE_ID --context 3`. En el panel, busca el mismo término y
abre el resultado.

Reinicia los agentes que conectaste. Por ejemplo, puedes pedirle a Codex
«Usa Mash para buscar qué decidimos sobre la autenticación en Claude Code y
muéstrame el mensaje original».

Los menús de Mash están en inglés. La búsqueda por significado usa modelos
opcionales y, por ahora, requiere consultas en inglés.
[Cómo funciona la búsqueda (en inglés)](../search.md)
