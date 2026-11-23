import { describe, expect, it } from "vitest";
import { mayOpenInApp, onThisMachine, resumeCommand } from "./openIn";

const session = "586f44fa-8aaa-4587-b69f-b763d3279ef1";

describe("opening a chat in its agent", () => {
  it("asks only for agents whose apps open sessions, by their session IDs", () => {
    expect(mayOpenInApp("codex", session)).toBe(true);
    expect(mayOpenInApp("claude-code", session)).toBe(true);
    // Mash's IDs for subagent chats are not sessions an app knows.
    expect(mayOpenInApp("claude-code", "50cf5623ea898883ead39971")).toBe(false);
    expect(mayOpenInApp("cursor", session)).toBe(false);
    expect(mayOpenInApp("opencode", session)).toBe(false);
  });

  it("offers it only for a chat of the machine showing the page", () => {
    expect(onThisMachine({ machine: "laptop" }, "laptop", false)).toBe(true);
    expect(onThisMachine({ machine: "desk" }, "laptop", false)).toBe(false);
    expect(onThisMachine({ machine: "laptop" }, "laptop", true)).toBe(false);
    expect(onThisMachine({ machine: "laptop" }, undefined, false)).toBe(false);
  });

  it("resumes sessions with the commands the agents document", () => {
    expect(resumeCommand("claude-code", session)).toBe(`claude --resume ${session}`);
    expect(resumeCommand("codex", session)).toBe(`codex resume ${session}`);
    expect(resumeCommand("copilot", session)).toBeUndefined();
    expect(resumeCommand("claude-code", "50cf5623ea898883ead39971")).toBeUndefined();
  });
});
