import { describe, expect, it } from "vitest";
import { markdown } from "./markdown.ts";
import {
  prepareTranslation,
  readTranslationSource,
  recordedTranslationRevision,
} from "./translation.ts";
import { readLocaleGuides } from "./translation-links.ts";

const source = {
  file: "docs/privacy.md",
  revision: "3".repeat(64),
  body: `# History

Open [the guide](installation.md#uninstall).

## Search

Choose **Skip agent connections** and run \`mash dash\`.

\`\`\`sh
mash search 'migration' --local
\`\`\`
`,
};

describe("documentation translation drafts", () => {
  it("keeps all sections, real UI labels, commands and link destinations", () => {
    const draft = prepareTranslation(source, "docs/fr/privacy.md");
    expect(draft.passages).not.toContain("Skip agent connections");
    const output = draft.render({
      translations: [
        "Historique",
        "Ouvrez",
        "le guide",
        ".",
        "Rechercher",
        "Choisissez",
        "et lancez",
        ".",
      ],
    });
    expect(output).toContain("# Historique");
    expect(output).toContain("## Rechercher");
    expect(output).toContain("[le guide](../installation.md#uninstall)");
    expect(output).toContain("**Skip agent connections** et lancez `mash dash`");
    expect(output).toContain("mash search 'migration' --local");
    expect(recordedTranslationRevision(output)).toBe(source.revision);
  });

  it("rejects a missing or empty translation before returning Markdown", () => {
    const draft = prepareTranslation(source, "docs/fr/privacy.md");
    expect(() => draft.render({ translations: draft.passages.slice(1) })).toThrow(
      "Expected exactly",
    );
    expect(() => draft.render({ translations: draft.passages.map(() => " ") })).toThrow(
      "passage 1 is missing or empty",
    );
  });

  it("keeps text returned by the translator from creating Markdown links or HTML", () => {
    const draft = prepareTranslation(
      { ...source, body: "# History\n\nOpen the guide." },
      "docs/fr/privacy.md",
    );
    const output = draft.render({ translations: ["Historique", "[Bad](//example.com) <script>"] });
    const body = markdown.parse(output).children[1];
    expect(body?.type).toBe("paragraph");
    if (body?.type !== "paragraph") throw new Error("Expected a paragraph");
    expect(body.children.map((node) => node.type)).toEqual(["text"]);
    expect(() =>
      draft.render({ translations: ["Historique", "Ouvrez https://example.com"] }),
    ).toThrow("changed a heading, link or code example");
  });

  it("takes both commands and the first search from the real English quick start", async () => {
    const quickStart = await readTranslationSource("quick-start");
    expect(quickStart.body).toMatch(/^# Quick start/);
    expect(quickStart.body).toContain("sh -s -- --setup");
    expect(quickStart.body).toContain(" -Setup");
    expect(quickStart.body).toContain("## Find your first chat");
    expect(quickStart.body).not.toContain("[Español]");
    expect(quickStart.body).not.toContain("## Licence");
  });

  it("can draft the installation guide without translating setup choices or shell commands", async () => {
    const installation = await readTranslationSource("installation");
    const draft = prepareTranslation(installation, "docs/es/installation.md");
    expect(draft.passages).not.toContain("Set up and index without connecting agents");
    const output = draft.render({
      translations: draft.passages.map((passage) =>
        passage === "Uninstall" ? "Desinstala Mash" : passage,
      ),
    });
    expect(output).toContain("**Set up and index without connecting agents**");
    expect(output).toContain("mash qdrant install");
    expect(output).toContain("sha256sum mash-x86_64-unknown-linux-gnu.tar.gz");
    expect(output).toContain('<span id="uninstall"></span>');
    expect(recordedTranslationRevision(output)).toBe(installation.revision);
  });

  it("drafts the support guide while keeping the setup wizard label and source revision", async () => {
    const support = await readTranslationSource("supported-agents");
    const draft = prepareTranslation(support, "docs/es/supported-agents.md");
    expect(draft.passages).toContain("Supported agents");
    expect(draft.passages).not.toContain("Customize setup");
    const output = draft.render({ translations: draft.passages });
    expect(output).toContain("**Customize setup**");
    expect(output).toContain("../agents.md#connect-claude-code-codex-or-copilot");
    expect(recordedTranslationRevision(output)).toBe(support.revision);
  });

  it("keeps manual section anchors local and protects the actual connection prompts", async () => {
    const source = await readTranslationSource("agents");
    const draft = prepareTranslation(source, "docs/fr/agents.md");
    const output = draft.render({ translations: draft.passages });

    for (const label of [
      "Use existing settings",
      "Which agents should get access to Mash search?",
      "Set up without indexing",
    ]) {
      expect(draft.passages).not.toContain(label);
      expect(output).toContain(`**${label}**`);
    }
    expect(output).toContain('<span id="try-it"></span>');
    expect(output).toContain("[Try a search](#try-it)");
    expect(output).toContain("../configuration.md#connected-agents");
  });

  it("drafts the dashboard guide with literal menu labels and commands", async () => {
    const dashboard = await readTranslationSource("dashboard");
    const draft = prepareTranslation(dashboard, "docs/es/dashboard.md");
    const output = draft.render({ translations: draft.passages });

    for (const label of [
      "Clear filters",
      "search: default",
      "memory from this chat",
      "overview:",
    ]) {
      expect(draft.passages).not.toContain(label);
      expect(output).toContain(`**${label}**`);
    }
    expect(output).toContain("mash dash --allow-host laptop.your-tailnet.ts.net");
    expect(recordedTranslationRevision(output)).toBe(dashboard.revision);
  });

  it("keeps numbered section links local when duplicate English headings are translated", () => {
    const draft = prepareTranslation(
      {
        ...source,
        body: "# Setup\n\n[Read the next section](#start-1).\n\n## Start\n\n## Start",
      },
      "docs/fr/privacy.md",
    );
    const output = draft.render({
      translations: draft.passages.map((passage) => (passage === "Start" ? "Début" : passage)),
    });
    expect(output).toContain("[Read the next section](#start-1)");
    expect(output).toContain('<span id="start"></span>\n\n## Début');
    expect(output).toContain('<span id="start-1"></span>\n\n## Début');
    expect(draft.render({ translations: draft.passages })).not.toContain("<span");
  });

  it("uses translated guides only when the linked section exists", async () => {
    const [localeGuides, quickStart] = await Promise.all([
      readLocaleGuides("fr"),
      readTranslationSource("quick-start"),
    ]);
    const draft = prepareTranslation(quickStart, "docs/fr/quick-start.md", localeGuides);
    const output = draft.render({ translations: draft.passages });
    expect(output).toContain("(dashboard.md#set-up-mash-from-the-dashboard)");
    expect(output).toContain("(dashboard.md#find-and-read-a-chat)");
    expect(output).toContain("(agents.md#connect-claude-code-codex-or-copilot)");
    expect(output).toContain("(installation.md)");

    const partial = prepareTranslation(
      {
        ...source,
        file: "docs/agents.md",
        body: "# History\n\n[Setup](../README.md#quick-start), then [search](../README.md#try-it) or read [another section](dashboard.md#english-only).",
      },
      "docs/fr/agents.md",
      localeGuides,
    );
    expect(partial.render({ translations: partial.passages })).toContain(
      "[Setup](quick-start.md), then [search](quick-start.md#try-it) or read [another section](../dashboard.md#english-only)",
    );
  });

  it("can draft a guide before its locale directory has been created", async () => {
    expect(await readLocaleGuides("unconfigured-test-locale")).toEqual(new Map());
  });

  it("rejects a translation that would make an old section link point to a different heading", () => {
    const draft = prepareTranslation(
      { ...source, body: "# Guide\n\n## Start\n\n## Finish" },
      "docs/fr/privacy.md",
    );
    expect(() => draft.render({ translations: ["Guide", "Début", "Start"] })).toThrow(
      "A translated heading conflicts with section link #start",
    );
  });
});
