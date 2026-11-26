import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { docsLocales } from "./scripts/locales.ts";
import { systemMarks } from "./scripts/systems.ts";
import { agentationReview } from "./src/agentation.ts";

export default defineConfig({
  site: "https://getmash.io",
  // GitHub Pages serves files only, so a retired page stays as a small HTML
  // page that sends the reader on: "Why Mash" (/compare/) is now the landing
  // page, so each language's copy goes to that language's home.
  redirects: Object.fromEntries(
    Object.keys(docsLocales).map((locale) => {
      const home = locale === "root" ? "/" : `/${locale}/`;
      return [`${home}compare`, home];
    }),
  ),
  integrations: [
    // Adds the review toolbar in `astro dev` only (`just site-review`).
    agentationReview(),
    starlight({
      title: "mash",
      favicon: "/favicon.svg",
      // Safari, iOS home screens and older browsers ignore SVG icons, so the
      // same logo also ships as PNG and ICO (rendered from favicon.svg).
      head: [
        {
          tag: "link",
          attrs: { rel: "icon", type: "image/png", sizes: "32x32", href: "/favicon-32.png" },
        },
        { tag: "link", attrs: { rel: "apple-touch-icon", href: "/apple-touch-icon.png" } },
      ],
      // The tagline, for pages that give no description of their own.
      description: "Supercharged context for your agents.",
      locales: docsLocales,
      // src/pages/404.astro draws the missing-page screen without the hero.
      disable404Route: true,
      // src/routeData.ts keeps untranslated pages out of each language's search.
      routeMiddleware: "./src/routeData.ts",
      // The header draws the logo itself (components/Header.astro): the blue
      // tile, then the lowercase name whose full stop is a pat of butter.
      logo: { src: "./public/mash-mark.svg" },
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/mashhq/mash" }],
      // The fonts are bundled with the site, as they are with the dashboard.
      customCss: [
        "@fontsource/geist/latin-400.css",
        "@fontsource/geist/latin-500.css",
        "@fontsource/geist/latin-600.css",
        "@fontsource/geist-mono/latin-400.css",
        "@fontsource/geist-mono/latin-500.css",
        "./src/styles/theme.css",
        "./src/styles/custom.css",
        "./src/styles/docs.css",
      ],
      // Code blocks are raised windows: a hairline ring and a soft shadow, a
      // bar with the systems' marks beside the title (scripts/systems.ts), and
      // a quiet copy button. docs.css draws the rest.
      expressiveCode: {
        plugins: [systemMarks()],
        // Long commands wrap inside the window instead of scrolling sideways.
        defaultProps: { wrap: true, preserveIndent: true },
        styleOverrides: {
          borderRadius: "var(--radius-7)",
          borderWidth: "0",
          borderColor: "transparent",
          codeBackground: "var(--surface-code)",
          codeFontFamily: "var(--type-mono)",
          codeFontSize: "0.8125rem",
          codeLineHeight: "1.8",
          codePaddingBlock: "1rem",
          codePaddingInline: "1.125rem",
          uiFontFamily: "var(--type-sans)",
          uiFontWeight: "500",
          uiFontSize: "0.8125rem",
          frames: {
            shadowColor: "transparent",
            frameBoxShadowCssValue: "var(--shadow-float)",
            editorBackground: "var(--surface-code)",
            editorTabBarBackground: "var(--surface-code)",
            editorTabBarBorderColor: "transparent",
            editorTabBarBorderBottomColor: "var(--line-divider)",
            editorActiveTabBackground: "var(--surface-code)",
            editorActiveTabForeground: "var(--text-strong)",
            editorActiveTabBorderColor: "transparent",
            editorActiveTabIndicatorTopColor: "transparent",
            editorActiveTabIndicatorBottomColor: "transparent",
            terminalBackground: "var(--surface-code)",
            terminalTitlebarBackground: "var(--surface-code)",
            terminalTitlebarForeground: "var(--text-strong)",
            terminalTitlebarBorderBottomColor: "var(--line-divider)",
            terminalTitlebarDotsOpacity: "0",
            inlineButtonBackground: "var(--surface-code)",
            inlineButtonBackgroundIdleOpacity: "1",
            inlineButtonBackgroundHoverOrFocusOpacity: "1",
            inlineButtonBorder: "var(--line-outline)",
            inlineButtonBorderOpacity: "1",
            inlineButtonForeground: "var(--text-secondary)",
            tooltipSuccessBackground: "var(--action-bg)",
            tooltipSuccessForeground: "var(--action-text)",
          },
        },
      },
      components: {
        Footer: "./src/components/Footer.astro",
        Head: "./src/components/Head.astro",
        Header: "./src/components/Header.astro",
        Hero: "./src/components/Hero.astro",
        LanguageSelect: "./src/components/LanguageSelect.astro",
        PageTitle: "./src/components/PageTitle.astro",
        Search: "./src/components/Search.astro",
        Sidebar: "./src/components/Sidebar.astro",
        SocialIcons: "./src/components/SocialIcons.astro",
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
              // The directory of every guide, named as its page is. The English
              // index is the landing page, which leaves the docs, so the
              // sidebar points here instead.
              label: "Mash documentation",
              translations: {
                es: "Documentación de Mash",
                fr: "Documentation de Mash",
                "zh-CN": "Mash 文档",
                ja: "Mash ドキュメント",
              },
              slug: "guides",
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
              label: "Installation",
              translations: {
                es: "Instalación",
                fr: "Installation",
                "zh-CN": "安装",
                ja: "インストール",
              },
              collapsed: true,
              items: [
                {
                  label: "Install Mash",
                  translations: {
                    es: "Configurar Mash",
                    fr: "Configurer Mash",
                    "zh-CN": "设置 Mash",
                    ja: "Mash のセットアップ",
                  },
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
                    ja: "その他のインストール方法",
                  },
                  slug: "install-methods",
                },
                {
                  label: "Local search service",
                  translations: {
                    es: "Servicio de búsqueda local",
                    fr: "Service de recherche local",
                    "zh-CN": "本地搜索服务",
                    ja: "ローカル検索サービス",
                  },
                  slug: "local-search",
                },
              ],
            },
            {
              label: "Use the dashboard",
              translations: {
                es: "Usar el panel",
                fr: "Utiliser le tableau de bord",
                "zh-CN": "使用网页面板",
                ja: "ダッシュボードを使う",
              },
              slug: "dashboard",
            },
            {
              label: "Search from the terminal",
              translations: {
                es: "Búsqueda desde el terminal",
                fr: "Recherche dans le terminal",
                "zh-CN": "终端搜索",
                ja: "ターミナルから検索する",
              },
              slug: "search",
            },
            {
              label: "Use Mash with your agent",
              translations: {
                es: "Conectar un agente",
                fr: "Connecter un agent",
                "zh-CN": "连接代理",
                ja: "エージェントを接続する",
              },
              slug: "agents",
            },
            {
              label: "Pick up past work",
              translations: {
                es: "Retomar un trabajo",
                fr: "Reprendre un travail",
                "zh-CN": "继续之前的工作",
                ja: "以前の作業を再開する",
              },
              slug: "pick-up-work",
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
          label: "Dashboard",
          translations: {
            es: "Panel web",
            fr: "Tableau de bord",
            "zh-CN": "网页面板",
            ja: "ダッシュボード",
          },
          items: ["dashboard-settings", "mashy", "dashboard-access", "services"],
        },
        {
          label: "Search",
          translations: {
            es: "Búsqueda",
            fr: "Recherche",
            "zh-CN": "搜索",
            ja: "検索",
          },
          items: [
            "find-missing-chat",
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
              label: "Keep history up to date",
              translations: {
                es: "Mantén el historial al día",
                fr: "Garder l’historique à jour",
                "zh-CN": "保持历史记录最新",
                ja: "履歴を最新に保つ",
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
              slug: "search-across-machines",
            },
            {
              label: "Set up a shared index",
              translations: {
                es: "Configurar un índice compartido",
                fr: "Configurer un index partagé",
                "zh-CN": "设置共享索引",
                ja: "共有インデックスを設定する",
              },
              slug: "shared-index",
            },
          ],
        },
        {
          label: "Agents and memory",
          translations: {
            es: "Agentes y memoria",
            fr: "Agents et mémoire",
            "zh-CN": "代理与记忆",
            ja: "エージェントと記憶",
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
              label: "Mash plugin",
              translations: {
                es: "Complemento de Mash",
                fr: "Module Mash",
                "zh-CN": "Mash 插件",
              },
              slug: "plugin",
            },
            {
              label: "Use Mash memory",
              translations: { es: "Memoria", fr: "Mémoire", "zh-CN": "记忆" },
              slug: "records",
            },
          ],
        },
        {
          label: "Maintenance",
          translations: {
            es: "Mantenimiento",
            fr: "Maintenance",
            "zh-CN": "维护",
            ja: "メンテナンス",
          },
          items: [
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
                ja: "Mash をアンインストールする",
              },
              slug: "uninstall",
            },
          ],
        },
        "troubleshooting",
        {
          label: "Reference",
          translations: {
            es: "Referencia",
            fr: "Référence",
            "zh-CN": "参考",
            ja: "リファレンス",
          },
          items: ["architecture", "cli", "configuration", "mcp", "tools", "record-tools"],
        },
      ],
    }),
  ],
});
