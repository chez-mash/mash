import { describe, expect, it } from "vitest";
import { markdown } from "./markdown.ts";
import {
  prepareTranslation,
  readTranslationSource,
  recordedTranslationRevision,
} from "./translation.ts";

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
});
