---
name: frontend-development-prefs
description: Use when starting a frontend project or choosing its framework, package manager, styling, tests, formatting, linting, components, or icons.
---

# Frontend Development Preferences

Build new frontends with the preferred stack unless explicit requirements or
an established repository require otherwise.

## Preferred stack

- TypeScript 7 native compiler with strict checking
- Svelte 5
- Vite
- Vitest
- Tailwind CSS 4
- pnpm
- Biome for formatting and linting
- shadcn-svelte for reusable UI components
- Phosphor Icons (`phosphor-svelte`) for interface icons

Follow the repository's TypeScript conventions for TypeScript work.

## Icons

Import each Phosphor glyph from its own file (`phosphor-svelte/lib/NAMEIcon`),
never the package root, so only the icons used are bundled. Draw icons through
one wrapper component so weight is decided in one place from theme tokens, not
per icon:

- bold at 15px and under, regular from 16px to 27px, light for decorative
  icons of 28px and over;
- fill for the current navigation item, the selected tab or segment, and
  "on" states;
- bold for warning and failure icons and for icons on primary buttons;
- one step heavier in a dark theme when lines look thin;
- duotone only for large feature icons where it clearly reads well.

Icons inherit `currentColor`. Outside Svelte, copy exact path data per weight
from `@phosphor-icons/core`. Brand and product logos are not interface icons.

For Svelte, run `svelte-check --tsgo`. If current `svelte-check` requires
TypeScript 6 alongside 7, keep 6 only as its compatibility dependency and
install 7 through `@typescript/native@npm:typescript@7`. Do not downgrade the
project checker; verify it executes TypeScript 7.

## Boundaries

Inspect repository instructions, manifests, lockfiles, and existing tooling
before changing anything. Preserve an established stack unless the user asks
for migration. Explicit user and repository requirements override these
defaults.

For new projects, do not substitute npm or Yarn, React or Next.js, ESLint or
Prettier, another component kit, or hand-drawn interface SVGs without a
constraint requiring the substitution. Report any compatibility issue that
prevents the preferred choice.

Install dependencies with pnpm. Use package scripts for development, checking,
testing, and building. Keep generated configuration minimal.

## Verification

Before completion, run Biome checks, TypeScript checking, Vitest, and the
production build. Report unavailable or failing gates.
