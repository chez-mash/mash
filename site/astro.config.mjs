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
          translations: {
            es: "Primeros pasos",
            fr: "Premiers pas",
            "zh-CN": "开始使用",
            ja: "はじめに",
          },
          items: [
            {
              label: "Welcome to Mash",
              translations: {
                es: "Mash en español",
                fr: "Mash en français",
                "zh-CN": "Mash 简体中文指南",
                ja: "Mash 日本語ガイド",
              },
              slug: "index",
            },
            {
              label: "Quick start",
              translations: {
                es: "Guía de inicio rápido",
                fr: "Guide de démarrage rapide",
                "zh-CN": "快速入门",
                ja: "クイックスタート",
              },
              slug: "quick-start",
            },
            {
              label: "Compare tools",
              translations: {
                es: "Comparar herramientas",
                fr: "Comparer les outils",
                "zh-CN": "比较工具",
              },
              slug: "compare",
            },
            {
              label: "Installation",
              translations: { es: "Instalación", fr: "Installation", "zh-CN": "安装" },
              slug: "installation",
            },
            {
              label: "Download and verify",
              translations: {
                es: "Descarga y verificación",
                fr: "Téléchargement et vérification",
                "zh-CN": "下载与校验",
                ja: "ダウンロードと検証",
              },
              slug: "downloads",
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
              translations: {
                es: "Privacidad",
                fr: "Confidentialité",
                "zh-CN": "隐私",
                ja: "プライバシー",
              },
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
            ja: "過去の作業を探す",
          },
          items: [
            {
              label: "Dashboard",
              translations: {
                es: "Panel",
                fr: "Tableau de bord",
                "zh-CN": "网页面板",
                ja: "ダッシュボード",
              },
              slug: "dashboard",
            },
            {
              label: "Pick up past work",
              translations: {
                es: "Retomar un trabajo",
                fr: "Reprendre un travail",
                "zh-CN": "继续之前的工作",
              },
              slug: "pick-up-work",
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
            {
              label: "Search in other languages",
              translations: {
                es: "Buscar en otros idiomas",
                fr: "Rechercher dans d’autres langues",
                "zh-CN": "搜索其他语言的对话",
                ja: "英語以外のチャットを検索する",
              },
              slug: "search-languages",
            },
            {
              label: "Memory",
              translations: { es: "Memoria", fr: "Mémoire", "zh-CN": "记忆" },
              slug: "records",
            },
          ],
        },
        {
          label: "Use Mash with agents",
          translations: {
            es: "Usa Mash con agentes",
            fr: "Utiliser Mash avec des agents",
            "zh-CN": "在代理中使用 Mash",
            ja: "エージェントと使う",
          },
          items: [
            {
              label: "Supported agents",
              translations: {
                es: "Agentes compatibles",
                fr: "Agents compatibles",
                "zh-CN": "支持的代理",
                ja: "対応するエージェント",
              },
              slug: "supported-agents",
            },
            {
              label: "Connect an agent",
              translations: {
                es: "Conectar un agente",
                fr: "Connecter un agent",
                "zh-CN": "连接代理",
                ja: "エージェントを接続する",
              },
              slug: "agents",
            },
            {
              label: "Mash plugin",
              translations: { es: "Complemento de Mash", fr: "Module Mash", "zh-CN": "Mash 插件" },
              slug: "plugin",
            },
          ],
        },
        {
          label: "Manage Mash",
          translations: {
            es: "Administra Mash",
            fr: "Gérer Mash",
            "zh-CN": "管理 Mash",
            ja: "Mash を管理する",
          },
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
            {
              label: "Keep history up to date",
              translations: {
                es: "Mantén el historial al día",
                fr: "Garder l’historique à jour",
                "zh-CN": "保持历史记录最新",
              },
              slug: "synchronization",
            },
            {
              label: "Older history and search models",
              translations: {
                es: "Historial anterior y modelos de búsqueda",
                fr: "Historique ancien et modèles de recherche",
                "zh-CN": "较早的历史记录和搜索模型",
              },
              slug: "indexing",
            },
            {
              label: "Search across your machines",
              translations: {
                es: "Buscar en varios equipos",
                fr: "Rechercher sur plusieurs ordinateurs",
                "zh-CN": "跨设备搜索",
              },
              slug: "shared-index",
            },
            {
              label: "Back up and restore history",
              translations: {
                es: "Copia y restaura tu historial",
                fr: "Sauvegarder et restaurer l’historique",
                "zh-CN": "备份和恢复历史记录",
              },
              slug: "backup",
            },
            {
              label: "Upgrade Mash",
              translations: {
                es: "Actualiza Mash",
                fr: "Mettre Mash à niveau",
                "zh-CN": "升级 Mash",
              },
              slug: "upgrades",
            },
            {
              label: "Uninstall Mash",
              translations: {
                es: "Desinstala Mash",
                fr: "Désinstaller Mash",
                "zh-CN": "卸载 Mash",
              },
              slug: "uninstall",
            },
            "services",
          ],
        },
        {
          label: "Reference",
          translations: { es: "Referencia", fr: "Référence", "zh-CN": "参考", ja: "リファレンス" },
          collapsed: true,
          items: ["cli", "configuration", "mcp", "tools", "record-tools"],
        },
      ],
    }),
  ],
});
