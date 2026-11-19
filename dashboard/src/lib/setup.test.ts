import { expect, test } from "vitest";
import { agents } from "./agents";
import * as wire from "./setup";
import { answersOf, opensOnSetup, problem, shortPath, stepWord, summary } from "./setup";
import { trimmed } from "./trimmed";
import type { Agent } from "./types";

const names = Object.fromEntries(
  Object.entries(agents).map(([agent, info]) => [agent, info.name]),
) as Record<Agent, string>;
const parseFirstRun = (answer: unknown) => wire.parseFirstRun(trimmed(answer));
const body = {
  path: "/h/.mash/config.toml",
  revision: "",
  configured: false,
  editable: true,
  histories: {
    "claude-code": { indexed: true, roots: ["/h/.claude/projects"] },
    codex: { indexed: true, roots: ["/h/.codex/sessions"] },
    "not-an-agent": { indexed: true, roots: [] },
  },
  since_days: 90,
  shared: false,
  integrations: [
    { agent: "claude-code", installed: true, connected: false },
    { agent: "codex", installed: false, connected: false },
  ],
  qdrant: { url: "http://127.0.0.1:6333", reachable: false, message: "start it" },
  models_installed: false,
  index: { state: "idle" },
};

test("a first setup starts from everything found and every installed agent", () => {
  const setup = parseFirstRun(body);
  expect(setup.histories.map((entry) => entry.agent)).toEqual(["claude-code", "codex"]);
  const answers = answersOf(setup);
  expect(answers).toEqual({
    agents: { "claude-code": true, codex: true },
    sinceDays: 90,
    connect: ["claude-code"],
  });
  expect(problem(setup, answers)).toBe("");
});

test("answers that cannot be saved say why", () => {
  const setup = parseFirstRun(body);
  const answers = answersOf(setup);
  answers.agents = { "claude-code": false, codex: false };
  expect(problem(setup, answers)).toMatch(/at least one/);
  answers.agents.codex = true;
  answers.sinceDays = 0;
  expect(problem(setup, answers)).toMatch(/between 1 and 36,500/);
  answers.sinceDays = null;
  expect(problem(setup, answers)).toBe("");
  expect(problem(parseFirstRun({ ...body, histories: {} }), answers)).toMatch(/No agent history/);
});

test("the review names each answer in the page's words", () => {
  const setup = parseFirstRun(body);
  const answers = answersOf(setup);
  answers.agents.codex = false;
  answers.sinceDays = null;
  answers.connect = [];
  expect(summary(setup, answers, names)).toEqual([
    { label: "configuration file", value: "/h/.mash/config.toml" },
    { label: "histories to index", value: "Claude Code" },
    { label: "history range", value: "all history" },
    { label: "connect Mash to", value: "no agents" },
    { label: "search", value: "this machine only" },
  ]);
});

test("the first index is read in each of its states", () => {
  const index = (state: unknown) => parseFirstRun({ ...body, index: state }).index;
  expect(index({ state: "running" })).toEqual({ state: "running" });
  expect(index({ state: "skipped", reason: "no Qdrant" })).toEqual({
    state: "skipped",
    reason: "no Qdrant",
  });
  expect(index({ state: "finished", ok: true, chats: 3, messages: 150 })).toEqual({
    state: "finished",
    ok: true,
    chats: 3,
    messages: 150,
    message: null,
  });
});

test("each step of connecting an agent has a plain word", () => {
  expect(stepWord({ state: "done", command: "c" })).toEqual({ ok: true, word: "added" });
  expect(stepWord({ state: "already-present" }).ok).toBe(true);
  expect(stepWord({ state: "failed", command: "c", message: "m" }).ok).toBe(false);
});

test("only a first visit to the front page of an unconfigured, private dashboard opens on setup", () => {
  const fresh = parseFirstRun(body);
  expect(opensOnSetup(fresh, "overview", false)).toBe(true);
  expect(opensOnSetup(fresh, "overview", true)).toBe(false);
  expect(opensOnSetup(fresh, "search", false)).toBe(false);
  expect(opensOnSetup(parseFirstRun({ ...body, configured: true }), "overview", false)).toBe(false);
  expect(opensOnSetup(parseFirstRun({ ...body, editable: false }), "overview", false)).toBe(false);
});

test("a history location is shown on one line, ending in the folders that identify it", () => {
  expect(shortPath("/home/sam/.claude/projects")).toBe("~/.claude/projects");
  expect(shortPath("/Users/sam/.codex/sessions")).toBe("~/.codex/sessions");
  expect(shortPath("C:\\Users\\sam\\.codex\\sessions")).toBe("~\\.codex\\sessions");
  expect(shortPath("/srv/data/archives/2026/agents/claude/projects")).toBe(
    "…/2026/agents/claude/projects",
  );
  expect(shortPath("/home/sam/Library/Application Support/Cursor/User")).toBe(
    "…/Application Support/Cursor/User",
  );
  expect(shortPath("/short")).toBe("/short");
});
