import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// Storybook is the reference for what exists, so what exists must be in it:
// every control of the kit, and every building block listed here, ships with
// a story beside it. Add a new building block to the list with its story.

const src = join(import.meta.dirname, "..");
const kit = join(src, "lib/components/ui");
const components = join(src, "components");

/** The building blocks pages are made of; each has `NAME.stories.svelte` beside it. */
const blocks = [
  "AgentCard",
  "AgentMark",
  "ChatPlace",
  "ChatRow",
  "CommandCopy",
  "CommandLine",
  "ConfirmCard",
  "DecisionCount",
  "EmptyState",
  "EvidenceCard",
  "ExportOutcome",
  "KindIcon",
  "KindPill",
  "LoadError",
  "MachineCard",
  "MachineIcon",
  "MemoryItem",
  "MemoryRow",
  "SavedMemory",
  "SearchRow",
  "ServiceList",
  "Skeleton",
  "StatusBadge",
  "ToneIcon",
  "UpdateBanner",
];

/** Composed pieces, whose stories live under src/stories/composed. */
const composed = ["Mashy", "QuickSearch", "Rail", "TopBar"];

const hasStory = (path: string): boolean =>
  existsSync(path) && /<Story\b/.test(readFileSync(path, "utf8"));

describe("every listed component has a story", () => {
  it("covers each control of the kit", () => {
    const controls = readdirSync(kit, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);
    expect(controls.length).toBeGreaterThan(0);
    const missing = controls.filter(
      (control) => !hasStory(join(kit, control, `${control}.stories.svelte`)),
    );
    expect(missing, "kit controls without CONTROL/CONTROL.stories.svelte").toEqual([]);
  });

  it("covers each building block", () => {
    const absent = blocks.filter((name) => !existsSync(join(components, `${name}.svelte`)));
    expect(absent, "listed building blocks that no longer exist").toEqual([]);
    const missing = blocks.filter((name) => !hasStory(join(components, `${name}.stories.svelte`)));
    expect(missing, "building blocks without NAME.stories.svelte beside them").toEqual([]);
  });

  it("covers each composed piece", () => {
    const missing = composed.filter(
      (name) => !hasStory(join(src, "stories/composed", `${name}.stories.svelte`)),
    );
    expect(missing, "composed pieces without a story in src/stories/composed").toEqual([]);
  });

  it("lists every component that has a story", () => {
    const storied = readdirSync(components)
      .filter((name) => name.endsWith(".stories.svelte"))
      .map((name) => name.replace(".stories.svelte", ""));
    expect(storied.filter((name) => !blocks.includes(name))).toEqual([]);
  });
});
