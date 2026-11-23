// Whose history the dashboard shows: this machine's own index, everything in
// the shared index with this machine's own chats, or one other machine.
//
// One choice covers the overview, chats, search, memory, projects and agents,
// so a page never quietly mixes sources. The choice lives in the address as
// `scope=` when it is set, and is remembered per browser; with neither, it
// follows what this machine can show (see `effectiveScope`).
import type { Catalog } from "./api";
import type { ChatSummary } from "./types";

export type Scope =
  | { readonly kind: "local" }
  | { readonly kind: "all" }
  | { readonly kind: "node"; readonly node: string };

/** What this dashboard can show, which decides the choices and the default. */
export interface Reach {
  /** This machine's machine ID. */
  readonly localNode: string;
  /** Configured to publish to and read a shared index. */
  readonly joined: boolean;
  /** No index of its own: everything it reads is a shared copy. */
  readonly sharedOnly: boolean;
  /** Machines whose own dashboards this one can read live. */
  readonly live: readonly string[];
}

const KEY = "mash.scope";

export function parseScope(text: string | null | undefined): Scope | null {
  if (text === "local") return { kind: "local" };
  if (text === "all") return { kind: "all" };
  if (text?.startsWith("node:") && text.length > 5) return { kind: "node", node: text.slice(5) };
  return null;
}
export function scopeText(scope: Scope): string {
  return scope.kind === "node" ? `node:${scope.node}` : scope.kind;
}
export function sameScope(a: Scope, b: Scope): boolean {
  return scopeText(a) === scopeText(b);
}

/** The scope a page uses: the chosen one when this dashboard can show it,
 * otherwise everything it can show. */
export function effectiveScope(chosen: Scope | null, reach: Reach): Scope {
  const all: Scope = reach.joined || reach.sharedOnly ? { kind: "all" } : { kind: "local" };
  if (!chosen) return all;
  if (chosen.kind === "local") return reach.sharedOnly ? all : chosen;
  if (chosen.kind === "all") return all;
  // Another machine needs the shared index, or its own dashboard, to be seen.
  if (chosen.node === reach.localNode && !reach.sharedOnly) return { kind: "local" };
  return reach.joined || reach.sharedOnly || reach.live.includes(chosen.node) ? chosen : all;
}

/** What is assumed when the catalogue cannot be read: this machine alone, so
 * memory still shows what this machine saved. */
export const unknownReach: Reach = { localNode: "", joined: false, sharedOnly: false, live: [] };

/** The scope and reach a memory read uses: the catalogue's when it loaded,
 * the chosen scope over `unknownReach` when it failed, nothing while loading. */
export function readingScope(
  scoped: { readonly scope: Scope; readonly reach: Reach } | undefined,
  failed: boolean,
  chosen: Scope | null,
): { readonly scope: Scope; readonly reach: Reach } | undefined {
  if (scoped) return { scope: scoped.scope, reach: scoped.reach };
  if (!failed) return undefined;
  return { scope: effectiveScope(chosen, unknownReach), reach: unknownReach };
}

/** Whether a scope reads a machine's own dashboard rather than its published copy. */
export function isLive(scope: Scope, reach: Reach): scope is Scope & { kind: "node" } {
  return scope.kind === "node" && reach.live.includes(scope.node);
}

/** The chats a scope covers, from the catalogue of this machine and the shared index. */
export function inScope(chat: ChatSummary, scope: Scope, localNode: string): boolean {
  if (scope.kind === "all") return true;
  if (scope.kind === "local") return chat.machine === localNode;
  return chat.machine === scope.node;
}

/** A catalogue narrowed to a scope. Counts and filter choices follow it. */
export function scopeCatalog(catalog: Catalog, scope: Scope): Catalog {
  if (scope.kind === "all") return catalog;
  const kept = (chat: ChatSummary) => inScope(chat, scope, catalog.localNode);
  return {
    ...catalog,
    chats: catalog.chats.filter(kept),
    all: catalog.all.filter(kept),
  };
}

/** The menu's and pill's name for a scope, such as "All machines · 3". */
export function scopeName(scope: Scope, reach: Reach, machines: number): string {
  if (scope.kind === "local") return `This machine · ${reach.localNode}`;
  if (scope.kind === "all") return `All machines · ${machines}`;
  return `${scope.node} · ${isLive(scope, reach) ? "live" : "published copy"}`;
}

/** One sentence on what a scope includes, said under a page's title. */
export function scopeSentence(scope: Scope, reach: Reach, live: boolean): string {
  const local = reach.localNode;
  if (scope.kind === "local")
    return `Only what ${local} indexed, including chats it hasn't published.`;
  if (scope.kind === "all")
    return reach.sharedOnly
      ? "Everything every machine published to the shared index."
      : `${local}'s own chats plus what your other machines published. A chat ${local} published counts once.`;
  return live
    ? `${scope.node}'s own index, read live from its dashboard.`
    : `What ${scope.node} published to the shared index. Work it hasn't published isn't here.`;
}

/** Machines whose own dashboards answered this session; their chats open from
 * there, which holds what they have not published too. */
export const liveMachinesSeen = new Set<string>();

function stored(): Scope | null {
  try {
    return parseScope(globalThis.localStorage?.getItem(KEY));
  } catch {
    return null;
  }
}
function fromAddress(): Scope | null {
  const hash = globalThis.location?.hash ?? "";
  return parseScope(new URLSearchParams(hash.split("?")[1]).get("scope"));
}

/** The hash without its `scope=`, which the pages never see. */
export function withoutScope(hash: string): string {
  const [path, search] = hash.split("?", 2);
  if (search === undefined) return hash;
  const params = new URLSearchParams(search);
  if (!params.has("scope")) return hash;
  params.delete("scope");
  return params.size ? `${path}?${params}` : (path ?? "");
}

class ScopeState {
  /** The reader's choice; null until they make one, which means the default. */
  chosen = $state<Scope | null>(fromAddress() ?? stored());

  constructor() {
    // A shared link's scope is adopted, and remembered like a choice.
    const adopt = () => {
      const linked = fromAddress();
      if (linked && (!this.chosen || !sameScope(linked, this.chosen))) this.choose(linked, false);
    };
    globalThis.addEventListener?.("hashchange", adopt);
    globalThis.addEventListener?.("popstate", adopt);
    globalThis.addEventListener?.("mash:navigate", adopt);
  }

  /** Choose a scope; it is written into the address so the page can be shared. */
  choose(scope: Scope, writeAddress = true): void {
    this.chosen = scope;
    try {
      globalThis.localStorage?.setItem(KEY, scopeText(scope));
    } catch {
      // Private windows may refuse storage; the address still carries it.
    }
    if (!writeAddress || !globalThis.location) return;
    const hash = withoutScope(location.hash || "#/");
    const [path, search] = hash.split("?", 2);
    const params = new URLSearchParams(search ?? "");
    params.set("scope", scopeText(scope));
    history.replaceState(null, "", `${path === "#" || !path ? "#/" : path}?${params}`);
  }
}

export const scope = new ScopeState();
