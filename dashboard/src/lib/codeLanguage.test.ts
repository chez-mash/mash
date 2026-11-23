import { describe, expect, it } from "vitest";
import {
  argumentHighlight,
  codexPatch,
  commandFile,
  fenceLanguage,
  isDiff,
  isJson,
  outputHighlight,
  pathLanguage,
} from "./codeLanguage";
import type { ToolArgument, ToolCall } from "./toolcalls";

function arg(key: string, text: string): ToolArgument {
  return { key, text, block: text.includes("\n") || text.length > 72 };
}
function call(name: string, args: Record<string, string>, output?: string): ToolCall {
  return {
    id: "m1",
    name,
    summary: "",
    at: "",
    args: Object.entries(args).map(([key, text]) => arg(key, text)),
    raw: "",
    ...(output === undefined ? {} : { output: { text: output } }),
  };
}

describe("languages from names", () => {
  it("knows files by extension, and nothing else", () => {
    expect(pathLanguage("app/core/qdrant.rs")).toBe("rust");
    expect(pathLanguage("C:\\work\\Main.TS")).toBe("typescript");
    expect(pathLanguage("dashboard/src/App.svelte")).toBe("svelte");
    expect(pathLanguage("Makefile")).toBeUndefined();
    expect(pathLanguage("notes.unknownext")).toBeUndefined();
    expect(pathLanguage("src/")).toBeUndefined();
    expect(pathLanguage(".env")).toBeUndefined();
  });
  it("reads fence tags people write, and leaves untagged fences plain", () => {
    expect(fenceLanguage("rust")).toBe("rust");
    expect(fenceLanguage("ts")).toBe("typescript");
    expect(fenceLanguage("sh")).toBe("shellscript");
    expect(fenceLanguage("bash title=install")).toBe("shellscript");
    expect(fenceLanguage("diff")).toBe("diff");
    expect(fenceLanguage("{.python}")).toBe("python");
    expect(fenceLanguage("")).toBeUndefined();
    expect(fenceLanguage(undefined)).toBeUndefined();
    expect(fenceLanguage("text")).toBeUndefined();
    expect(fenceLanguage("mermaid")).toBeUndefined();
  });
});

describe("text that says what it is", () => {
  it("is a diff only from a diff header", () => {
    expect(isDiff("diff --git a/x b/x\n--- a/x")).toBe(true);
    expect(isDiff("--- a/src/x.rs\n+++ b/src/x.rs")).toBe(true);
    expect(isDiff("@@ -1,3 +1,4 @@\n line")).toBe(true);
    expect(isDiff("--- separator in a log ---")).toBe(false);
    expect(isDiff("Compiling mash v0.2.0\n+ added")).toBe(false);
  });
  it("is JSON when it parses, whole or by line", () => {
    expect(isJson('{"a": 1}')).toBe(true);
    expect(isJson('{"a":1}\n{"b":2}')).toBe(true);
    expect(isJson('{"a":1}\nnot json')).toBe(false);
    expect(isJson("[warn] something")).toBe(false);
    expect(isJson("plain words")).toBe(false);
  });
});

describe("a command that prints exactly one file", () => {
  it.each([
    ["cat app/main.rs", "app/main.rs"],
    ["cd /repo && sed -n 120,165p internal/lexer.go", "internal/lexer.go"],
    ["head -n 40 src/lib/x.ts", "src/lib/x.ts"],
    ["tail -20 'docs/a b.md'", "docs/a b.md"],
    ["cat config.toml | head -30", "config.toml"],
  ])("%s", (command, file) => {
    expect(commandFile(command)).toBe(file);
  });
  it.each([
    "cat a.rs b.rs",
    "cat -n app/main.rs",
    "nl app/main.rs",
    "sed 's/a/b/' x.rs",
    "sed -n '/fn main/,/^}/p' x.rs",
    "grep -n fn app/main.rs",
    "cat a.rs; cat b.rs",
    "cat a.rs && cargo test",
    "cat a.rs | grep fn",
    "cat $(git ls-files | head -1)",
    "cat src/*.rs",
    "cat x.rs > copy.rs",
    "python3 - <<'EOF'\nprint(1)\nEOF",
    "cargo test -p mash-dash",
    "git show HEAD:src/main.rs",
  ])("not: %s", (command) => {
    expect(commandFile(command)).toBeUndefined();
  });
});

describe("tool output", () => {
  it("colours a read of one file, keeping its line numbers apart", () => {
    const read = call("Read", { file_path: "/r/app/main.rs" }, "1\tuse std::fs;\n2\tfn main() {}");
    expect(outputHighlight(read)).toEqual({ language: "rust", numbered: true });
    const view = call("view", { path: "a.py", view_range: "[1, 20]" }, "import os\nprint(os)");
    expect(outputHighlight(view)).toEqual({ language: "python" });
  });
  it("leaves a read with notes mixed in plain", () => {
    const read = call(
      "Read",
      { file_path: "a.rs" },
      "1\tfn a() {}\n<system-reminder>note</system-reminder>",
    );
    expect(outputHighlight(read)).toBeUndefined();
  });
  it("colours a command's output only when it printed one known file", () => {
    expect(
      outputHighlight(call("Bash", { command: "sed -n 1,5p a.go" }, "package main\n")),
    ).toEqual({
      language: "go",
    });
    expect(
      outputHighlight(call("Bash", { command: "cargo test" }, "running 3 tests\nok")),
    ).toBeUndefined();
    expect(
      outputHighlight(call("Bash", { command: "cat a.rs; cat b.ts" }, "fn a() {}")),
    ).toBeUndefined();
    expect(outputHighlight(call("Bash", { command: "ls" }, "a.rs\nb.rs"))).toBeUndefined();
  });
  it("reads Codex's command arrays", () => {
    const codex = call(
      "shell",
      { command: '["bash", "-lc", "cat src/x.ts"]' },
      "export const x = 1;",
    );
    expect(outputHighlight(codex)).toEqual({ language: "typescript" });
  });
  it("colours diffs and JSON whatever printed them", () => {
    expect(outputHighlight(call("Bash", { command: "git diff" }, "diff --git a/x b/x\n"))).toEqual({
      language: "diff",
    });
    expect(outputHighlight(call("Bash", { command: "gh api x" }, '{"a":1}\n{"a":2}'))).toEqual({
      language: "json",
    });
  });
  it("does not colour searches, writes or empty output", () => {
    expect(
      outputHighlight(call("Grep", { path: "a.rs", pattern: "fn" }, "3:fn a() {}")),
    ).toBeUndefined();
    expect(
      outputHighlight(call("Write", { file_path: "a.rs", content: "fn a() {}\n" }, "File created")),
    ).toBeUndefined();
    expect(outputHighlight(call("Read", { file_path: "a.rs" }, "  "))).toBeUndefined();
    expect(outputHighlight(call("Read", { file_path: "notes.txt" }, "hello"))).toBeUndefined();
  });
});

describe("tool arguments", () => {
  const edit = call("Edit", {
    file_path: "src/a.ts",
    old_string: "const a = 1;\nconst b = 2;",
    new_string: "const a = 3;\nconst b = 4;",
  });
  it("colours an edit's text in its file's language, marking what went and came", () => {
    const [, removed, added] = edit.args;
    expect(removed && argumentHighlight(edit, removed)).toEqual({
      language: "typescript",
      tone: "removed",
    });
    expect(added && argumentHighlight(edit, added)).toEqual({
      language: "typescript",
      tone: "added",
    });
  });
  it("colours a written file and JSON values, and nothing short or unknown", () => {
    const write = call("Write", { file_path: "a.py", content: "import os\nprint(os.getcwd())" });
    expect(argumentHighlight(write, write.args[1] as ToolArgument)).toEqual({ language: "python" });
    const json = call("mcp", { query: '{\n  "q": "x"\n}' });
    expect(argumentHighlight(json, json.args[0] as ToolArgument)).toEqual({ language: "json" });
    const prose = call("Task", { prompt: "Look through the repo and\nreport what you find." });
    expect(argumentHighlight(prose, prose.args[0] as ToolArgument)).toBeUndefined();
    const short = call("Read", { file_path: "a.rs" });
    expect(argumentHighlight(short, short.args[0] as ToolArgument)).toBeUndefined();
    const unknown = call("Write", { file_path: "notes.txt", content: "hello\nthere" });
    expect(argumentHighlight(unknown, unknown.args[1] as ToolArgument)).toBeUndefined();
  });
});

// The shape Codex's `exec` tool records, minimised from real sessions.
const PATCH =
  "*** Begin Patch\n*** Update File: /work/app/limits.py\n@@\n-LIMIT = 10\n+LIMIT = 20\n*** End Patch";
const quoted = (patch: string) => JSON.stringify(patch);

describe("Codex patches", () => {
  it("reads a raw patch as it is", () => {
    expect(codexPatch(PATCH)).toBe(PATCH);
  });
  it("decodes an exec call that only applies a patch", () => {
    expect(codexPatch(`text(await tools.apply_patch(${quoted(PATCH)}))`)).toBe(PATCH);
    expect(codexPatch(`await tools.apply_patch(${quoted(PATCH)});`)).toBe(PATCH);
    expect(codexPatch(`text(await tools.apply_patch(\`${PATCH}\`));`)).toBe(PATCH);
    expect(codexPatch(`text(await tools.apply_patch('${PATCH.replaceAll("\n", "\\n")}'));`)).toBe(
      PATCH,
    );
  });
  it("keeps every other line of the code exactly as written", () => {
    const code = `text(await tools.apply_patch(${quoted(PATCH)}));\ntext(await tools.exec_command({cmd:"pytest -q"}));`;
    expect(codexPatch(code)).toBe(`${PATCH}\ntext(await tools.exec_command({cmd:"pytest -q"}));`);
  });
  it("leaves code it can't read exactly as written", () => {
    // A template with a substitution isn't one fixed text.
    expect(
      codexPatch("text(await tools.apply_patch(`*** Begin Patch\n${body}\n*** End Patch`));"),
    ).toBeUndefined();
    // A patch applied inside other code isn't a line of its own.
    expect(
      codexPatch(`const r=await Promise.allSettled([tools.apply_patch(${quoted(PATCH)})]);`),
    ).toBeUndefined();
    // A literal that isn't a whole patch, and prose that mentions one.
    expect(codexPatch(`text(await tools.apply_patch("*** Begin Patch"));`)).toBeUndefined();
    expect(codexPatch("Use *** Begin Patch and *** End Patch around the hunks.")).toBeUndefined();
    // An unknown escape could mean anything.
    expect(
      codexPatch(`text(await tools.apply_patch("*** Begin Patch\\q\\n*** End Patch"));`),
    ).toBeUndefined();
  });
  it("colours the decoded patch as a diff in place of the code", () => {
    const code = `text(await tools.apply_patch(${quoted(PATCH)}))`;
    const exec = call("exec", { input: code });
    expect(argumentHighlight(exec, arg("input", code))).toEqual({ language: "diff", text: PATCH });
    expect(argumentHighlight(exec, arg("input", PATCH))).toEqual({ language: "diff" });
    expect(
      argumentHighlight(exec, arg("input", 'text(await tools.exec_command({cmd:"ls"}))\n')),
    ).toBeUndefined();
  });
});
