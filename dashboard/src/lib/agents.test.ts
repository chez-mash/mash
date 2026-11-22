import { expect, test } from "vitest";
import { agentNamed } from "./agents";

test("a recorder's name finds its agent by id or by the name shown for it", () => {
  expect(agentNamed("Claude Code")).toBe("claude-code");
  expect(agentNamed("codex")).toBe("codex");
  expect(agentNamed(" github copilot ")).toBe("copilot");
  expect(agentNamed("sam")).toBeUndefined();
  expect(agentNamed("")).toBeUndefined();
});
