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
          translations: { es: "Primeros pasos", fr: "Premiers pas", "zh-CN": "开始使用" },
          items: [
            {
              label: "Welcome to Mash",
              translations: {
                es: "Mash en español",
                fr: "Mash en français",
                "zh-CN": "Mash 简体中文指南",
              },
              slug: "index",
            },
            {
              label: "Quick start",
              translations: {
                es: "Guía de inicio rápido",
                fr: "Guide de démarrage rapide",
                "zh-CN": "快速入门",
              },
              slug: "quick-start",
            },
            {
              label: "Installation",
              translations: { es: "Instalación", fr: "Installation", "zh-CN": "安装" },
              slug: "installation",
            },
            {
              label: "Other install methods",
              translations: {
                es: "Otros métodos de instalación",
                fr: "Autres méthodes d’installation",
                "zh-CN": "其他安装方式",
              },
              slug: "install-methods",
            },
            {
              label: "Privacy",
              translations: { es: "Privacidad", fr: "Confidentialité", "zh-CN": "隐私" },
              slug: "privacy",
            },
          ],
        },
        {
          label: "Find past work",
          translations: {
            es: "Encuentra conversaciones",
            fr: "Retrouver ses travaux",
            "zh-CN": "查找历史对话",
          },
          items: [
            {
              label: "Dashboard",
              translations: { es: "Panel", fr: "Tableau de bord", "zh-CN": "网页面板" },
              slug: "dashboard",
            },
            {
              label: "Terminal search",
              translations: {
                es: "Búsqueda desde el terminal",
                fr: "Recherche dans le terminal",
                "zh-CN": "终端搜索",
              },
              slug: "search",
            },
            "records",
          ],
        },
        {
          label: "Use Mash with agents",
          translations: {
            es: "Usa Mash con agentes",
            fr: "Utiliser Mash avec des agents",
            "zh-CN": "在代理中使用 Mash",
          },
          items: [
            {
              label: "Supported agents",
              translations: {
                es: "Agentes compatibles",
                fr: "Agents compatibles",
                "zh-CN": "支持的代理",
              },
              slug: "supported-agents",
            },
            "agents",
            "plugin",
          ],
        },
        {
          label: "Manage Mash",
          translations: { es: "Administra Mash", fr: "Gérer Mash", "zh-CN": "管理 Mash" },
          items: [
            "troubleshooting",
            {
              label: "Local search service",
              translations: {
                es: "Servicio de búsqueda local",
                fr: "Service de recherche local",
                "zh-CN": "本地搜索服务",
              },
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
          translations: { es: "Referencia", fr: "Référence", "zh-CN": "参考" },
          collapsed: true,
          items: ["cli", "configuration", "mcp", "tools"],
        },
      ],
    }),
  ],
});
