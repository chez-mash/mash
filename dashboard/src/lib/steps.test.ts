import { expect, test } from "vitest";
import { shownText } from "./message";
import { contextLabel, stepParts, stepsSummary, transcriptItems } from "./steps";
import type { Message } from "./types";

function message(
  id: string,
  role: string,
  presentation: NonNullable<Message["presentation"]>,
  content = id,
  tools?: Message["tools"],
): Message {
  return {
    id,
    role,
    presentation,
    at: "10:22",
    blocks: [{ type: "text", markdown: content }],
    ...(tools ? { tools } : {}),
  };
}
const call = (id: string, callId: string): Message =>
  message(id, "assistant", "tool-call", "", {
    calls: [{ name: "Bash", callId, input: { command: "ls" } }],
    results: [],
  });
const result = (id: string, callId: string, content = "out"): Message =>
  message(id, "user", "tool-result", content, {
    calls: [],
    results: [{ callId, isError: false }],
  });
function run(page: Parameters<typeof transcriptItems>[0], index: number) {
  const item = transcriptItems(page)[index];
  if (item?.kind !== "steps") throw new Error(`item ${index} is not a run of steps`);
  return item;
}

test("context is folded with the steps around it and is never a message bubble", () => {
  const items = transcriptItems({
    messages: [
      message("sys", "system", "context"),
      message("q", "user", "dialogue"),
      message("think", "assistant", "reasoning"),
      message("hook", "user", "context"),
      call("c", "t1"),
      result("r", "t1"),
      message("a", "assistant", "dialogue"),
    ],
  });
  expect(
    items.map((item) =>
      item.kind === "steps" ? item.messages.map((step) => step.id) : item.message.id,
    ),
  ).toEqual([["sys"], "q", ["think", "hook", "c", "r"], "a"]);
});

test("a run shows thinking, context and each call once, in order", () => {
  const steps = run(
    {
      messages: [
        message("think", "assistant", "reasoning"),
        message("hook", "user", "context"),
        call("c", "t1"),
        result("r", "t1"),
      ],
    },
    0,
  );
  const parts = stepParts(steps);
  expect(parts.map((part) => (part.kind === "note" ? part.message.id : part.call.id))).toEqual([
    "think",
    "hook",
    "c",
  ]);
  expect(parts[2]).toMatchObject({ kind: "call", call: { resultId: "r" } });
  expect(stepsSummary(steps)).toBe("1 tool call: Bash");
});

test("a run of context alone says so instead of counting steps", () => {
  const one = run({ messages: [message("sys", "system", "context")] }, 0);
  expect(stepsSummary(one)).toBe("System instructions");
  const three = run(
    {
      messages: [
        message("a", "system", "context"),
        message("b", "developer", "context"),
        message("c", "user", "context"),
      ],
    },
    0,
  );
  expect(stepsSummary(three)).toBe("3 context items");
  expect(stepParts(three).map((part) => part.kind)).toEqual(["note", "note", "note"]);
  const mixed = run(
    { messages: [message("t", "assistant", "reasoning"), message("c", "user", "context")] },
    0,
  );
  expect(stepsSummary(mixed)).toBe("2 steps");
});

test("context is named by who it is attributed to", () => {
  expect(contextLabel(message("a", "user", "context"))).toBe("Context added by the agent");
  expect(contextLabel(message("a", "system", "context"))).toBe("System instructions");
  expect(contextLabel(message("a", "developer", "context"))).toBe("Developer instructions");
  expect(contextLabel(message("a", "assistant", "context", "plain words"))).toBe("Activity");
  expect(
    contextLabel(message("a", "assistant", "context", '{"type":"contextCompaction","n":1}')),
  ).toBe("Activity: context compaction");
  expect(contextLabel(message("a", "assistant", "context", '{"type":"context_compaction"}'))).toBe(
    "Activity: context compaction",
  );
  expect(contextLabel(message("a", "assistant", "context", '{"type":7}'))).toBe("Activity");
  expect(contextLabel(message("a", "assistant", "context", "{broken"))).toBe("Activity");
  expect(contextLabel(message("a", "critic", "context"))).toBe("critic");
});

test("a call that ends the page shows the output read from after it", () => {
  const steps = run(
    {
      messages: [message("q", "user", "dialogue"), call("c", "t1")],
      after: [result("r", "t1", "from the next page")],
    },
    1,
  );
  const parts = stepParts(steps);
  expect(parts).toHaveLength(1);
  expect(parts[0]).toMatchObject({
    kind: "call",
    call: { id: "c", resultId: "r", name: "Bash", output: { text: "from the next page" } },
  });
  expect(stepsSummary(steps)).toBe("1 tool call: Bash");
});

test("output that opens the page is shown under its call from before it", () => {
  const twoCalls = message("c", "assistant", "tool-call", "", {
    calls: [
      { name: "Read", callId: "t0", input: { path: "a" } },
      { name: "Bash", callId: "t1", input: { command: "ls" } },
    ],
    results: [],
  });
  const steps = run(
    {
      before: [twoCalls],
      messages: [result("r", "t1"), message("a", "assistant", "dialogue")],
    },
    0,
  );
  const parts = stepParts(steps);
  // The other call of that message was answered on the page before.
  expect(parts).toHaveLength(1);
  expect(parts[0]).toMatchObject({
    kind: "call",
    call: { id: "c:1", resultId: "r", name: "Bash", output: { text: "out" } },
    messages: [{ id: "r" }],
  });
  expect(stepsSummary(steps)).toBe("1 tool call: Bash");
});

test("neighbours belong only to the runs at the page's edges", () => {
  const items = transcriptItems({
    before: [call("b", "t0")],
    after: [result("z", "t2")],
    messages: [
      result("r0", "t0"),
      message("q", "user", "dialogue"),
      call("c1", "t1"),
      result("r1", "t1"),
      message("a", "assistant", "dialogue"),
      call("c2", "t2"),
    ],
  });
  expect(
    items.map((item) => (item.kind === "steps" ? [!!item.before, !!item.after] : null)),
  ).toEqual([[true, false], null, [false, false], null, [false, true]]);
});

test("find reads a person's words first, then the message they arrived in", () => {
  const typed = {
    ...message("q", "user", "dialogue", "<context>x</context> fix it"),
    dialogue: "fix it",
  };
  expect(shownText(typed)).toBe("fix it\n\n<context>x</context> fix it");
  expect(shownText(message("a", "assistant", "dialogue", "done"))).toBe("done");
});
