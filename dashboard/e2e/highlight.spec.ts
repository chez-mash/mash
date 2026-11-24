import { expect, type Page, test } from "@playwright/test";

// Code is coloured only where its language is certain, and colouring changes
// nothing about the text: copying, selecting and finding read the same words.
const chat = {
  chat_id: "code",
  provider: "claude-code",
  title: "Colour the code",
  project: "/work/mash",
  updated_at: Date.now(),
  node_id: "laptop",
};
const call = (id: string, ordinal: number, name: string, input: unknown) => ({
  id,
  ordinal,
  role: "assistant",
  content: "",
  tools: { tool_calls: [{ call_id: `call-${id}`, name, input }] },
});
const result = (id: string, ordinal: number, content: string) => ({
  id: `${id}-out`,
  ordinal,
  role: "user",
  content,
  tools: { tool_results: [{ call_id: `call-${id}` }] },
});
const messages = [
  {
    id: "reply",
    ordinal: 0,
    role: "assistant",
    content:
      'The fix:\n\n```rust\nfn main() {\n    let total: u32 = 40 + 2;\n    println!("{total}");\n}\n```\n\nAnd a fence with no language:\n\n```\nplain words here\n```',
  },
  call("read", 1, "Read", { file_path: "/work/mash/src/lib.rs" }),
  result("read", 2, "1\tpub fn answer() -> u32 {\n2\t    42\n3\t}"),
  call("edit", 3, "Edit", {
    file_path: "/work/mash/src/app.ts",
    old_string: "const limit = 10;\nexport { limit };",
    new_string: "const limit = 20;\nexport { limit };",
  }),
  result("edit", 4, "The file was updated."),
  call("json", 5, "Bash", { command: "gh api repos/grep-stack/mash" }),
  result("json", 6, '{"name": "mash"}\n{"name": "mash-dash"}'),
  call("log", 7, "Bash", { command: "cargo test -p mash-dash" }),
  result("log", 8, "running 3 tests\ntest live::a ... ok\ntest result: ok. 3 passed"),
  // Codex's exec tool applies a patch from JavaScript, then runs a command.
  call(
    "patch",
    9,
    "exec",
    `text(await tools.apply_patch(${JSON.stringify(
      "*** Begin Patch\n*** Update File: /work/app/limits.py\n@@\n-LIMIT = 10\n+LIMIT = 20\n*** End Patch",
    )}));\ntext(await tools.exec_command({cmd:"pytest -q"}));`,
  ),
  result("patch", 10, "Success. Updated the following files:\nM /work/app/limits.py"),
];

async function open(page: Page): Promise<void> {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ json: { local_node: "laptop", truncated: false, chats: [chat] } }),
  );
  await page.route("**/api/chat?**", (route) => {
    // Pages end with an empty one, as a whole-chat read expects.
    const offset = Number(new URL(route.request().url()).searchParams.get("offset") ?? 0);
    return route.fulfill({ json: { messages: offset ? [] : messages } });
  });
  await page.goto("/#/chat/code");
  await expect(page.getByRole("heading", { name: chat.title })).toBeVisible();
}

async function openSteps(page: Page): Promise<void> {
  const runs = page.locator("details.fold.steps, details:has(> summary:has-text('tool call'))");
  for (const run of await runs.all())
    if (!(await run.getAttribute("open"))) await run.locator("> summary").click();
  for (const step of await page.locator("details.tc").all()) {
    if (!(await step.getAttribute("open"))) await step.locator("> summary").click();
  }
}

for (const width of [1440, 390]) {
  test(`code is coloured only where its language is certain at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: width > 500 ? 1000 : 844 });
    await open(page);
    // A fence that names its language is coloured and keeps its text.
    const fence = page.locator('main pre[data-highlight="rust"]:has(> code)');
    await expect(fence.locator("code span").first()).toBeVisible();
    await expect(fence).toHaveText(/let total: u32 = 40 \+ 2;/);
    const coloured = await fence
      .locator("code span")
      .evaluateAll((spans) => spans.map((span) => getComputedStyle(span).color));
    expect(new Set(coloured).size).toBeGreaterThan(1);
    // An untagged fence stays plain text.
    await expect(
      page.locator("main pre", { hasText: "plain words here" }).locator("span"),
    ).toHaveCount(0);

    await openSteps(page);
    const read = page.locator('pre[data-highlight="rust"]', { hasText: "pub fn answer" });
    await expect(read.locator(".hl-gutter").first()).toHaveText("1\t");
    await expect(read).toHaveText("1\tpub fn answer() -> u32 {\n2\t    42\n3\t}");
    const edit = page.locator('pre[data-highlight="typescript"]');
    await expect(edit).toHaveCount(2);
    await expect(edit.first()).toHaveClass(/tone-removed/);
    await expect(edit.last()).toHaveClass(/tone-added/);
    await expect(
      page.locator('pre[data-highlight="json"]', { hasText: "mash-dash" }),
    ).toBeVisible();
    // A Codex patch shows as the patch it applies, coloured as a diff; the
    // command that ran after it stays as written.
    const patch = page.locator('pre[data-highlight="diff"]', { hasText: "LIMIT = 20" });
    await expect(patch.locator(".hl-line.head").first()).toHaveText("*** Begin Patch");
    await expect(patch.locator(".hl-line.removed")).toHaveText("-LIMIT = 10");
    await expect(patch.locator(".hl-line.added")).toHaveText("+LIMIT = 20");
    await expect(patch).toContainText('text(await tools.exec_command({cmd:"pytest -q"}));');
    // A build log names no language: it stays exactly as it was.
    const log = page.locator("pre", { hasText: "running 3 tests" });
    await expect(log).toHaveAttribute("data-highlight", "plain");
    await expect(log.locator("*")).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
      true,
    );
    const directory = process.env.MASH_SCREENSHOTS;
    if (directory)
      await page.screenshot({ path: `${directory}/highlight-${width}.png`, fullPage: true });
  });
}

test("find in chat matches text that colouring split into pieces", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page);
  const fence = page.locator('main pre[data-highlight="rust"]:has(> code)');
  await expect(fence.locator("code span").first()).toBeVisible();
  // "u32 = 40" crosses several coloured tokens.
  await page.getByRole("button", { name: /Find in chat/ }).click();
  await page.getByLabel("Find in this chat").fill("u32 = 40");
  await expect(page.locator(".findn")).toHaveText(/^1 (of 1|match)/);
  await page.getByLabel("Find in this chat").press("Enter");
  await expect(page.locator(".findn")).toHaveText("1 of 1");
  // The match is painted inside the coloured block, across its tokens.
  await expect
    .poll(() =>
      page.evaluate(() => {
        if (!("highlights" in CSS)) return "unsupported";
        const [range] = [...(CSS.highlights.get("mash-find-current") ?? [])] as Range[];
        return range ? range.toString() : "";
      }),
    )
    .toMatch(/^(u32 = 40|unsupported)$/);
});
