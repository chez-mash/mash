// What the shared server reports about its group, read once and used by the
// services map's shared band and the viewing switcher's machine rows.
//
// Everything here comes from the server's own status report, read with this
// machine's token (`/api/shared/server`). When the report is missing, or lacks
// a field, the caller says "not reported"; nothing here guesses at health.
import { queryOptions } from "@tanstack/svelte-query";
import { read } from "./api";
import type { ServerView, Machine as SharedMachine } from "./shared";

/** The same query the Machines page's server card reads. */
export const serverViewOptions = queryOptions({
  queryKey: ["shared-server"],
  queryFn: ({ signal }) => read("/api/shared/server", new URLSearchParams(), signal),
  staleTime: 60_000,
  retry: false,
  refetchOnWindowFocus: false,
});

/** A machine whose last publish is older than this is shown as stale. */
export const STALE_AFTER_SECONDS = 2 * 24 * 3600;

/** One machine as the shared server reports it. */
export interface ServerMachine {
  readonly id: string;
  /** "joined", "waiting to join" and so on, in words. */
  readonly standing: string;
  /** Seconds since 1970 by the server's clock; null when it never published. */
  readonly publishedAt: number | null;
  readonly chats: number | null;
  /** Its first publish is still arriving. */
  readonly receiving: { covered: number; total: number } | null;
  /** It last published longer ago than `STALE_AFTER_SECONDS`. */
  readonly stale: boolean;
  /** It was removed from the group. */
  readonly removed: boolean;
}

/** The shared server and its group, or why they can't be shown. */
export type ServerReport =
  | { readonly kind: "none" }
  | { readonly kind: "unreachable"; readonly address: string; readonly message: string }
  | {
      readonly kind: "reported";
      readonly address: string;
      readonly group: string;
      readonly build: string;
      /** The server's clock, in seconds since 1970. */
      readonly now: number;
      readonly restartNeeded: boolean;
      readonly machines: readonly ServerMachine[];
    };

function seconds(value: string | null | undefined): number | null {
  if (!value) return null;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? Math.floor(parsed / 1000) : null;
}

function standing(machine: SharedMachine): string {
  switch (machine.state) {
    case "configured":
      return "set in the server's configuration";
    case "waiting":
      return "waiting to join";
    case "expired":
      return "never joined";
    case "joined":
      return "joined";
    case "removed":
      return "removed";
  }
}

/** The server's report as the map and the switcher read it. */
export function serverReport(view: ServerView | undefined): ServerReport {
  if (!view?.server) return { kind: "none" };
  const status = view.status;
  if (!status)
    return {
      kind: "unreachable",
      address: view.server,
      message: view.problem?.message ?? "The shared server didn't report its status.",
    };
  const machines = status.machines.map((machine): ServerMachine => {
    const publishedAt = seconds(machine.published?.last_synced_at);
    return {
      id: machine.node_id,
      standing: standing(machine),
      publishedAt,
      chats: machine.published ? machine.published.chats : null,
      receiving: machine.receiving ?? null,
      stale: publishedAt !== null && status.now - publishedAt > STALE_AFTER_SECONDS,
      removed: machine.state === "removed",
    };
  });
  return {
    kind: "reported",
    address: view.server,
    group: status.group_id,
    build: status.build,
    now: status.now,
    restartNeeded: status.restart_needed,
    machines: machines.sort((a, b) => a.id.localeCompare(b.id)),
  };
}

/** "12m ago", "3h ago", "5 days ago". */
export function ago(then: number, now: number): string {
  const elapsed = Math.max(0, now - then);
  if (elapsed < 60) return "just now";
  if (elapsed < 3600) return `${Math.floor(elapsed / 60)}m ago`;
  if (elapsed < 86_400) return `${Math.floor(elapsed / 3600)}h ago`;
  const days = Math.floor(elapsed / 86_400);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

/** How a machine's publishing stands, in a few words for a map box or menu row. */
export function publishWords(machine: ServerMachine, now: number): string {
  if (machine.removed) return "removed from the group";
  if (machine.publishedAt !== null) return `published ${ago(machine.publishedAt, now)}`;
  if (machine.receiving)
    return `first publish: ${machine.receiving.covered.toLocaleString()} of ${machine.receiving.total.toLocaleString()} chats`;
  return machine.standing === "joined" ? "nothing published yet" : machine.standing;
}

/** "5 days old" for a stale machine, otherwise nothing. */
export function staleWords(machine: ServerMachine, now: number): string {
  if (!machine.stale || machine.publishedAt === null) return "";
  const days = Math.floor((now - machine.publishedAt) / 86_400);
  return `${days} day${days === 1 ? "" : "s"} old`;
}

/** One machine's report, when the server reported it. */
export function reported(report: ServerReport, id: string): ServerMachine | undefined {
  return report.kind === "reported"
    ? report.machines.find((machine) => machine.id === id)
    : undefined;
}
