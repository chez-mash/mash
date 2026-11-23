import { queryOptions } from "@tanstack/svelte-query";
import { type Catalog, catalog, liveMachines } from "./api";
import type { Reach, Scope } from "./scope.svelte";

export const catalogOptions = queryOptions({
  queryKey: ["catalog"],
  queryFn: ({ signal }) => catalog(signal),
  staleTime: Infinity,
  retry: false,
  refetchOnMount: false,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
});
export function searchQueryKey(params: string): readonly ["search", string] {
  return ["search", params];
}

/** Submitted searches and reading pages change only at the reader's request. */
export const stableRead = {
  // Explicit reads must settle even when navigator.onLine is false. Otherwise
  // TanStack pauses them before fetch and the screen stays "Loading" forever.
  networkMode: "always",
  staleTime: Infinity,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
  refetchOnMount: false,
  retry: false,
  refetchInterval: false,
} as const;

/** The machines whose own dashboards this one can read live. A dashboard
 * without any, or an older one, answers with none. */
export const liveOptions = queryOptions({
  queryKey: ["catalog", "viewing"],
  queryFn: ({ signal }) => liveMachines(signal).catch((): readonly string[] => []),
  staleTime: Infinity,
  retry: false,
  refetchOnMount: false,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
});

export function reachOf(whole: Catalog, live: readonly string[]): Reach {
  return {
    localNode: whole.localNode,
    joined: whole.joined,
    sharedOnly: whole.sharedOnly,
    live: live.filter((node) => node !== whole.localNode),
  };
}

/** The catalogue a scoped page lists, with the scope it ended up showing. */
export interface Scoped extends Catalog {
  readonly scope: Scope;
  readonly reach: Reach;
  /** Read from the machine's own dashboard rather than its published copy. */
  readonly live: boolean;
  /** Why a live read fell back to the published copy, said on the page. */
  readonly fallback?: string;
}
