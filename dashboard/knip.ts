import type { KnipConfig } from "knip";

// Knip reads only a component's <script>. The kit's parts are used from the
// markup as members of a namespace (<Dialog.Root>), so each such member is
// added to the script as an expression; otherwise every part would be
// reported as an unused export.
function svelte(source: string): string {
  const scripts = [...source.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)].map(
    (found) => found[1] ?? "",
  );
  const members = new Set([...source.matchAll(/<([A-Z]\w*(?:\.\w+)+)/g)].map((found) => found[1]));
  return [...scripts, ...members].join(";\n");
}

const config: KnipConfig = {
  // index.html names src/main.ts and public/load-recovery.js; the demo build
  // swaps in its own entry (vite.config.ts), and the live browser checks run
  // only against a real dashboard (playwright.config.ts).
  // tools/visual.sh runs the visual baselines with their own configuration.
  // Storybook reads its configuration and finds the stories by their names.
  entry: [
    "src/demo/main.ts",
    ".storybook/*.ts",
    "src/**/*.stories.svelte",
    "e2e/**/*.live.ts",
    "playwright.visual.config.ts",
    "visual/**/*.spec.ts",
  ],
  project: [
    "src/**/*.{ts,svelte,css}",
    ".storybook/*.ts",
    "e2e/**/*.ts",
    "visual/**/*.ts",
    "*.ts",
    "public/*.js",
  ],
  compilers: { svelte },
  ignoreDependencies: [
    // The compiler `svelte-check --tsgo` runs.
    "@typescript/native",
  ],
};

export default config;
