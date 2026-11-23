// The demo's federated search (/api/search/stream): its one instant search,
// answered as a single finished update with the other sources skipped.
import type { Answer } from "../lib/routes";

export function streamed(
  found: Answer<"/api/search">,
  params: URLSearchParams,
  localMachine: string,
) {
  const limit = Number(params.get("limit") ?? 10);
  return {
    query: params.get("q") ?? "",
    local_node: localMachine,
    done: true,
    results: found.results.slice(0, limit).map((row) => ({ ...row, matched_by: ["words"] })),
    sources: [
      {
        source: "words",
        label: localMachine,
        state: "done",
        count: found.results.length,
        elapsed_ms: 1,
      },
      {
        source: "meaning",
        label: "by meaning",
        state: "skipped",
        reason: "search by meaning isn't in the demo",
      },
      {
        source: "machines",
        label: "your other machines",
        state: "skipped",
        reason: "the demo's machines are searched with this one",
      },
    ],
  };
}
