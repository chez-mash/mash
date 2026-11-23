import type { Page } from "@playwright/test";

/**
 * Answer federated search (`/api/search/stream`) from the search a spec
 * already mocks: the stream's question is asked of the page's own
 * `/api/search` (or `/api/search/live` while typing) and the answer comes back
 * as one finished update, found by words. A spec that tests streaming itself
 * routes the stream directly instead. Call after the spec's own routes, so
 * this one is matched first.
 */
export async function streamFromSearch(page: Page): Promise<void> {
  await page.route("**/api/search/stream?**", async (route) => {
    const asked = new URL(route.request().url()).searchParams;
    const legacy = new URLSearchParams({ q: asked.get("q") ?? "" });
    for (const name of ["provider", "project", "days"]) {
      const value = asked.get(name);
      if (value) legacy.set(name, value);
    }
    const scope = asked.get("scope") ?? "all";
    if (scope.startsWith("node:")) legacy.set("machine", scope.slice(5));
    const live = asked.get("live") === "true";
    if (live) legacy.set("limit", asked.get("limit") ?? "8");
    else legacy.set("offset", "0");
    const path = live ? "/api/search/live" : "/api/search";
    let answer: { status: number; text: string };
    try {
      answer = await page.evaluate(async (url) => {
        const response = await fetch(url);
        return { status: response.status, text: await response.text() };
      }, `${path}?${legacy}`);
    } catch {
      // The test ended, or the page moved on, while this search was asked.
      return route.abort().catch(() => undefined);
    }
    if (answer.status !== 200)
      return route.fulfill({
        status: answer.status,
        contentType: "application/json",
        body: answer.text,
      });
    const body = JSON.parse(answer.text) as { results?: object[]; shared_error?: string | null };
    const results = (body.results ?? []).map((row) => ({ ...row, matched_by: ["words"] }));
    const update = {
      query: asked.get("q") ?? "",
      local_node: "",
      done: true,
      results,
      sources: [
        {
          source: "words",
          label: "this machine",
          state: "done",
          count: results.length,
          elapsed_ms: 1,
        },
        ...(body.shared_error
          ? [
              {
                source: "machines",
                label: "your other machines",
                state: "failed",
                message: body.shared_error,
                elapsed_ms: 1,
                timed_out: false,
              },
            ]
          : []),
      ],
    };
    return route.fulfill({
      status: 200,
      contentType: "application/x-ndjson",
      body: `${JSON.stringify(update)}\n`,
    });
  });
}
