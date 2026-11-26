import process from "node:process";
import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";

// A Phosphor icon carries the path data of all six weights. The dashboard
// draws four (see IconWeight in src/lib/icon.svelte.ts), so the other two are
// cut from each icon before it is compiled. A glyph whose source no longer has
// the expected shape stops the build rather than shipping untrimmed.
const unusedWeights = ["duotone", "thin"];
function phosphorWeights(): Plugin {
  return {
    name: "mash-phosphor-weights",
    enforce: "pre",
    transform(code, id) {
      if (!/phosphor-svelte\/lib\/\w+Icon\.svelte$/.test(id)) return null;
      let trimmed = code;
      for (const weight of unusedWeights) {
        const branch = new RegExp(`\\{:else if weight === "${weight}"\\}[\\s\\S]*?(?=\\{:else)`);
        if (!branch.test(trimmed)) this.error(`no ${weight} weight to trim in ${id}`);
        trimmed = trimmed.replace(branch, "");
      }
      return { code: trimmed, map: null };
    },
  };
}

// `mash dash` serves the built files; during development the dev server
// forwards API calls to a running `mash dash`.
//
// `--mode demo` builds the same dashboard for getmash.io/demo, where it is
// answered from made-up history in the browser (see src/demo).
export default defineConfig(({ mode }) => {
  const demo = mode === "demo";
  return {
    base: demo ? "/demo/" : "/",
    plugins: [
      tailwindcss(),
      phosphorWeights(),
      svelte(),
      ...(demo
        ? [
            {
              name: "mash-demo-entry",
              transformIndexHtml: {
                order: "pre" as const,
                handler: (html: string) => html.replace("/src/main.ts", "/src/demo/main.ts"),
              },
            },
          ]
        : []),
    ],
    // `$lib` is where shadcn-svelte's components and their helpers live.
    resolve: { alias: { $lib: fileURLToPath(new URL("./src/lib", import.meta.url)) } },
    // Found before the first page load, so the dev server does not stop to
    // bundle the kit the first time a page that uses it opens.
    optimizeDeps: { include: ["bits-ui", "clsx", "tailwind-merge"] },
    build: { outDir: demo ? "dist-demo" : "dist", emptyOutDir: true, sourcemap: false },
    server: {
      proxy: demo
        ? {}
        : {
            "/api": {
              target: process.env.MASH_DASHBOARD_API_URL || "http://127.0.0.1:8790",
              changeOrigin: true,
            },
          },
    },
    test: { environment: "node", include: ["src/**/*.test.ts"] },
  };
});
