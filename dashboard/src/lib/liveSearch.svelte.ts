// Searching while the reader types, for the top bar and the quick panel: every
// source the viewing scope covers, merged into one list (see federated.ts).
// This machine's words answer at once; search by meaning starts once typing
// pauses on three or more characters; other machines add their results as
// they answer. A machine read live is asked itself, on its own dashboard.
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { liveSearch } from "./api";
import { type FederatedHit, statusLine } from "./federated";
import { FederatedSearch } from "./federatedSearch.svelte";
import type { Scoped } from "./queries";
import { inScope, scopeText } from "./scope.svelte";

export interface LiveSearchOptions {
  /** The text, once typing has paused briefly. */
  readonly typed: string;
  readonly enabled: boolean;
  readonly catalogue: Scoped | undefined;
  /** Search anyway without the chat list (it failed), so the failure shows. */
  readonly withoutCatalogue?: boolean;
  readonly limit: number;
}

export interface LiveSearch {
  readonly hits: readonly FederatedHit[];
  readonly status: { readonly text: string; readonly retry: boolean };
  readonly error: Error | null;
  /** Some answer has arrived for the current text. */
  readonly answered: boolean;
  /** The reader moved to a result: keep the list where it is. */
  interact(): void;
  retry(): void;
}

/** Call while a component starts, like `createQuery`. */
export function createLiveSearch(options: () => LiveSearchOptions): LiveSearch {
  let paused = $state("");
  $effect(() => {
    const value = options().typed;
    const timer = setTimeout(() => (paused = value), 300);
    return () => clearTimeout(timer);
  });
  const peer = $derived.by(() => {
    const data = options().catalogue;
    return data?.live && data.scope.kind === "node" ? data.scope.node : undefined;
  });
  const viaPeer = createQuery(() => ({
    queryKey: ["live-search", options().typed, peer ?? ""],
    queryFn: ({ signal }: { signal: AbortSignal }) =>
      liveSearch(options().typed, options().catalogue?.reach.localNode ?? "", signal, peer),
    enabled: options().enabled && peer !== undefined,
    networkMode: "always" as const,
    staleTime: 30_000,
    retry: false,
    placeholderData: (previous: Awaited<ReturnType<typeof liveSearch>> | undefined) => previous,
  }));
  const federated = new FederatedSearch();
  const question = $derived.by(() => {
    const { typed, enabled, catalogue, withoutCatalogue } = options();
    if (!enabled || peer !== undefined || (!catalogue && !withoutCatalogue)) return null;
    return {
      query: typed,
      scope: catalogue?.scope ?? ({ kind: "all" } as const),
      localNode: catalogue?.reach.localNode ?? "",
      // Once an answer says this machine cannot search by meaning, a pause
      // asks nothing more.
      wordsOnly: paused !== typed || typed.length < 3 || !untrack(() => federated.meaningPossible),
    };
  });
  const asked = $derived(
    question ? JSON.stringify([question.query, scopeText(question.scope), question.wordsOnly]) : "",
  );
  $effect(() => {
    void asked;
    const current = untrack(() => question);
    if (!current) {
      untrack(() => federated.stop());
      return;
    }
    untrack(() =>
      federated.start(
        {
          query: current.query,
          scope: current.scope,
          live: true,
          limit: options().limit,
          wordsOnly: current.wordsOnly,
        },
        current.localNode,
      ),
    );
  });
  return {
    get hits() {
      if (peer !== undefined)
        return (viaPeer.data ?? []).map((hit) => ({ ...hit, byMeaning: false }));
      // The server keeps to the scope; rows outside it are never shown anyway.
      const data = options().catalogue;
      return data
        ? federated.hits.filter((hit) => inScope(hit.chat, data.scope, data.reach.localNode))
        : federated.hits;
    },
    get status() {
      return peer !== undefined
        ? { text: "", retry: false }
        : statusLine(federated.sources, federated.hits.length, federated.done);
    },
    get error() {
      if (peer !== undefined) return viaPeer.error;
      return federated.error ? new Error(federated.error) : null;
    },
    get answered() {
      return peer !== undefined
        ? viaPeer.data !== undefined
        : federated.sources.length > 0 || federated.error !== null;
    },
    interact: () => federated.interact(),
    retry: () => (peer !== undefined ? void viaPeer.refetch() : federated.retry()),
  };
}
