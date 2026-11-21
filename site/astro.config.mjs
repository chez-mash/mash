import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { docsLocales } from "./scripts/locales.ts";

export default defineConfig({
  site: "https://getmash.io",
  integrations: [
    starlight({
      title: "mash",
      favicon: "/favicon.svg",
      description: "The Multi-Agent Search History Tool",
      locales: docsLocales,
      // The header is the project logo drawn live: the mark, the name and its
      // cursor, in the dark colouring or the light one to suit the theme.
      logo: { dark: "./public/mash-mark.svg", light: "./public/mash-mark-light.svg" },
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/mashium/mash" }],
      // The fonts are bundled with the site, as they are with the dashboard.
      customCss: [
        "@fontsource/ibm-plex-sans/latin-400.css",
        "@fontsource/ibm-plex-sans/latin-400-italic.css",
        "@fontsource/ibm-plex-sans/latin-500.css",
        "@fontsource/ibm-plex-sans/latin-600.css",
        "@fontsource/ibm-plex-mono/latin-400.css",
        "@fontsource/ibm-plex-mono/latin-500.css",
        "./src/styles/custom.css",
      ],
      components: {
        Footer: "./src/components/Footer.astro",
        Head: "./src/components/Head.astro",
        Hero: "./src/components/Hero.astro",
        Search: "./src/components/Search.astro",
        SocialIcons: "./src/components/SocialIcons.astro",
        ThemeSelect: "./src/components/ThemeSelect.astro",
      },
      sidebar: [
        {
          label: "Start here",
          translations: { es: "Primeros pasos" },
          items: [
            {
              label: "Welcome to Mash",
              translations: { es: "Mash en español" },
              slug: "index",
            },
            {
              label: "Quick start",
              translations: { es: "Guía de inicio rápido" },
              slug: "quick-start",
            },
            { label: "Installation", translations: { es: "Instalación" }, slug: "installation" },
            {
              label: "Other install methods",
              translations: { es: "Otros métodos de instalación" },
              slug: "install-methods",
            },
            {
              label: "Privacy",
              translations: { es: "Privacidad" },
              slug: "privacy",
            },
          ],
        },
        {
          label: "Find past work",
          translations: { es: "Encuentra conversaciones" },
          items: [
            { label: "Dashboard", translations: { es: "Panel" }, slug: "dashboard" },
            {
              label: "Terminal search",
              translations: { es: "Búsqueda desde el terminal" },
              slug: "search",
            },
            "records",
          ],
        },
        {
          label: "Use Mash with agents",
          translations: { es: "Usa Mash con agentes" },
          items: ["agents", "plugin"],
        },
        {
          label: "Manage Mash",
          translations: { es: "Administra Mash" },
          items: [
            "troubleshooting",
            {
              label: "Local search service",
              translations: { es: "Servicio de búsqueda local" },
              slug: "local-search",
            },
            "synchronization",
            "indexing",
            "shared-index",
            "upgrades",
            "services",
          ],
        },
        {
          label: "Reference",
          translations: { es: "Referencia" },
          collapsed: true,
          items: ["cli", "configuration", "mcp", "tools"],
        },
      ],
    }),
  ],
});
