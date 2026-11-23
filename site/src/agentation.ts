// The Agentation review toolbar for `make site-review`. In `astro dev` it adds
// the toolbar to every page and proxies the toolbar's requests to the review
// services; in any other command (build, preview, check) it adds nothing, so
// the published site never carries the toolbar or its React runtime.
//
// A review worker edits files while the reviewer may still be typing a note.
// The toolbar saves the open note in localStorage and reopens it after a
// reload; with `hot`, it also tells this dev server a note is open, and
// holdReloadsWhileDrafting holds full reloads until the note is added or
// cancelled (or two quiet minutes pass). CSS and module updates still apply.
import { holdReloadsWhileDrafting, reviewProxy } from "agentation-review/vite";
import type { AstroIntegration } from "astro";

// The service ports, shared with agentation-review.config.mjs. They differ
// from the Agentation defaults (4747 and 4848), which the gohawk site uses on
// Xenia, so both reviews can run at once.
export const reviewPorts = { syncPort: 4757, reviewPort: 4858 };

// Mounted again after each client-router navigation, which replaces the body.
const toolbarScript = `
if (import.meta.env.DEV) {
  import("agentation-review/browser").then(({ mount: mountToolbar, unmount }) => {
    const mount = () => mountToolbar({ hot: import.meta.hot });
    mount();
    document.addEventListener("astro:page-load", mount);
    document.addEventListener("astro:before-swap", unmount);
    import.meta.hot?.dispose(() => {
      document.removeEventListener("astro:page-load", mount);
      document.removeEventListener("astro:before-swap", unmount);
      unmount();
    });
  });
}
`;

export function agentationReview(): AstroIntegration {
  return {
    name: "mash-agentation-review",
    hooks: {
      "astro:config:setup": ({ command, injectScript, updateConfig }) => {
        if (command !== "dev") return;
        injectScript("page", toolbarScript);
        updateConfig({
          vite: {
            plugins: [holdReloadsWhileDrafting()],
            server: {
              proxy: reviewProxy(reviewPorts),
              // The Tailscale name that serves the review to phones and laptops.
              allowedHosts: ["xenia.taild75e12.ts.net"],
            },
          },
        });
      },
    },
  };
}
