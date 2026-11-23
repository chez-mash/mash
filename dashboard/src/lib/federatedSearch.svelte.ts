// One federated search on screen: its rows, every source's state, and the
// settle-then-freeze rule (see `settle` in federated.ts).
import {
  type FederatedHit,
  type FederatedParams,
  hitKey,
  SETTLE_MS,
  type SourceState,
  settle,
  streamSearch,
} from "./federated";
import { sameScope } from "./scope.svelte";

function sameQuestion(a: FederatedParams | undefined, b: FederatedParams): boolean {
  return (
    a !== undefined &&
    a.query === b.query &&
    a.live === b.live &&
    sameScope(a.scope, b.scope) &&
    (a.agents ?? []).join() === (b.agents ?? []).join() &&
    (a.projects ?? []).join() === (b.projects ?? []).join() &&
    (a.days ?? null) === (b.days ?? null)
  );
}

/** Finished searches, so going Back shows a result list again without asking. */
const finished = new Map<string, { hits: FederatedHit[]; sources: SourceState[] }>();
const FINISHED_KEPT = 20;

/** The address a reader just typed and submitted: it reads current results,
 * not kept ones, while Back and Forward still show what was kept. */
let freshAddress: string | null = null;
export function askAfresh(address: string): void {
  freshAddress = address;
}
/** Whether `address` was just submitted; asking clears it. */
export function takeAfresh(address: string): boolean {
  const fresh = freshAddress === address;
  if (fresh) freshAddress = null;
  return fresh;
}

export class FederatedSearch {
  hits = $state<FederatedHit[]>([]);
  sources = $state<SourceState[]>([]);
  done = $state(true);
  error = $state<string | null>(null);
  /** Rows already shown keep their places; later results only add below. */
  frozen = false;
  #controller: AbortController | undefined;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #params: FederatedParams | undefined;
  #localNode = "";

  /** Whether this machine can search by meaning: unknown until the first answer. */
  get meaningPossible(): boolean {
    const meaning = this.sources.find((source) => source.source === "meaning");
    if (!this.sources.length) return true;
    return (
      meaning !== undefined &&
      (meaning.state !== "skipped" || (meaning.reason ?? "").includes("short"))
    );
  }

  /** Show a finished answer to the same question at once, if one is kept. */
  reuse(params: FederatedParams, localNode: string): boolean {
    const kept = finished.get(JSON.stringify([params, localNode]));
    if (!kept) return false;
    this.stop();
    this.#params = params;
    this.#localNode = localNode;
    this.hits = kept.hits;
    this.sources = kept.sources;
    this.frozen = true;
    return true;
  }

  /**
   * Ask `params`, replacing any search still running. The same question asked
   * again (say, now with search by meaning) keeps the rows it has; a new one
   * starts empty and unfrozen.
   */
  start(params: FederatedParams, localNode: string): void {
    this.#controller?.abort();
    const controller = new AbortController();
    this.#controller = controller;
    if (!sameQuestion(this.#params, params)) {
      clearTimeout(this.#timer);
      this.#timer = undefined;
      this.frozen = false;
      this.hits = [];
      this.sources = [];
    }
    this.#params = params;
    this.#localNode = localNode;
    this.done = false;
    this.error = null;
    streamSearch(params, localNode, controller.signal, (update) => {
      // An older question's answer never reaches a newer one's list.
      if (controller.signal.aborted) return;
      this.hits = settle(this.hits, update.hits, this.frozen, hitKey);
      this.sources = [...update.sources];
      this.done = update.done;
      if (update.done) {
        finished.delete(JSON.stringify([params, localNode]));
        finished.set(JSON.stringify([params, localNode]), {
          hits: this.hits,
          sources: this.sources,
        });
        if (finished.size > FINISHED_KEPT) finished.delete(finished.keys().next().value ?? "");
      }
      if (update.hits.length && !this.frozen && this.#timer === undefined)
        this.#timer = setTimeout(() => (this.frozen = true), SETTLE_MS);
    }).catch((error: unknown) => {
      if (controller.signal.aborted) return;
      this.error = error instanceof Error ? error.message : "Search failed.";
      this.done = true;
    });
  }

  /** The reader moved to a result: keep what is shown where it is. */
  interact(): void {
    this.frozen = true;
  }

  /** Ask again, keeping what is shown; new results add below. */
  retry(): void {
    if (!this.#params) return;
    this.frozen = true;
    this.start(this.#params, this.#localNode);
  }

  /** Ask for `more` further results, keeping what is shown. */
  more(more: number): void {
    if (!this.#params) return;
    this.frozen = true;
    this.start({ ...this.#params, limit: this.#params.limit + more }, this.#localNode);
  }

  stop(): void {
    this.#controller?.abort();
    this.#controller = undefined;
    clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#params = undefined;
    this.hits = [];
    this.sources = [];
    this.done = true;
    this.error = null;
    this.frozen = false;
  }
}
