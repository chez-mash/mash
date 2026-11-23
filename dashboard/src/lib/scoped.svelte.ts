// The catalogue a scoped page lists: the whole catalogue narrowed to the
// viewing scope, or, for a machine read live, that machine's own chats from its
// dashboard. It is derived from the whole catalogue's query rather than
// fetched inside a query of its own, so the sidebar and every page keep
// reading the one catalogue query and follow it as it updates.
import { createQuery } from "@tanstack/svelte-query";
import { peerCatalog } from "./api";
import { catalogOptions, liveOptions, reachOf, type Scoped } from "./queries";
import { effectiveScope, isLive, liveMachinesSeen, scope, scopeCatalog } from "./scope.svelte";

/** What a page needs of the scoped catalogue; it reads like a query. */
export interface ScopedQuery {
  readonly data: Scoped | undefined;
  readonly error: Error | null;
  readonly isError: boolean;
  readonly isPending: boolean;
  readonly isFetching: boolean;
  readonly dataUpdatedAt: number;
  refetch: () => unknown;
}

/** Call while a component starts, like `createQuery`. */
export function createScoped(): ScopedQuery {
  const whole = createQuery(() => catalogOptions);
  const liveList = createQuery(() => liveOptions);
  const reach = $derived(whole.data ? reachOf(whole.data, liveList.data ?? []) : undefined);
  const ready = $derived(reach !== undefined && !liveList.isPending);
  const target = $derived(reach && ready ? effectiveScope(scope.chosen, reach) : undefined);
  const peer = $derived(target && reach && isLive(target, reach) ? target.node : "");
  const remote = createQuery(() => ({
    queryKey: ["catalog", "peer", peer],
    enabled: peer !== "",
    queryFn: async ({ signal }: { signal: AbortSignal }) => {
      try {
        const own = await peerCatalog(peer, signal);
        liveMachinesSeen.add(peer);
        return own;
      } catch (error) {
        liveMachinesSeen.delete(peer);
        throw error;
      }
    },
    staleTime: Infinity,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  }));
  const data = $derived.by((): Scoped | undefined => {
    if (!whole.data || !reach || !target) return undefined;
    if (!peer) return { ...scopeCatalog(whole.data, target), scope: target, reach, live: false };
    if (remote.data) return { ...remote.data, scope: target, reach, live: true };
    if (!remote.isError) return undefined;
    // A machine that does not answer is shown from its published copy, and
    // the page says so.
    return {
      ...scopeCatalog(whole.data, target),
      scope: target,
      reach,
      live: false,
      fallback: `${peer}'s dashboard didn't answer, so this is its published copy.`,
    };
  });
  return {
    get data() {
      return data;
    },
    get error() {
      return whole.error;
    },
    get isError() {
      return whole.isError;
    },
    get isPending() {
      return !whole.isError && data === undefined;
    },
    get isFetching() {
      return whole.isFetching || remote.isFetching;
    },
    get dataUpdatedAt() {
      return Math.max(whole.dataUpdatedAt, remote.dataUpdatedAt);
    },
    refetch: () => (peer ? remote.refetch() : whole.refetch()),
  };
}
