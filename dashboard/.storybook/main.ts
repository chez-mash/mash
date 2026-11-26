import process from "node:process";
import type { StorybookConfig } from "@storybook/svelte-vite";

// Storybook is the dashboard's design reference: every token, kit control and
// building block in its states, drawn with the dashboard's own stylesheets
// and made-up data. It builds with the dashboard's Vite configuration (the
// Tailwind and Svelte plugins, the `$lib` alias, the trimmed icon weights)
// and is no part of `pnpm build`: nothing in src/ imports a story.
const config: StorybookConfig = {
  framework: "@storybook/svelte-vite",
  stories: ["../src/**/*.stories.svelte"],
  addons: ["@storybook/addon-svelte-csf", "@storybook/addon-a11y"],
  // Mashy's picture and the other files the dashboard serves beside its code.
  staticDirs: ["../public"],
  core: {
    // Nothing about this repository or who builds it leaves the machine.
    disableTelemetry: true,
    disableWhatsNewNotifications: true,
    // The dev server answers its own address only. A preview on the tailnet
    // (tools/worktree.sh preview NAME --storybook) names this machine's
    // MagicDNS name for that run, from outside the committed configuration.
    allowedHosts: process.env.MASH_STORYBOOK_HOSTS?.split(",").filter(Boolean) ?? [],
  },
};

export default config;
