// The services map's picture of one machine: which of its reported services
// fill each place in Mash's pipeline, and the connections between those places.
//
// The connections are not reported by the services API. They are the fixed
// shape of Mash itself, the same on every machine: agents' history files are
// read by indexing, which writes the SQLite index, from which local search is
// built, which the dashboard and agents (over MCP) search; indexing also
// publishes to the shared server when the machine shares, and the server builds
// its own search index. So the map draws an edge only between two places that
// are both present, and never guesses at anything else (an embedding model,
// another machine's internals): what nothing reports is shown as not reported.
// The shared server's own report (lib/serverStatus.ts) supplies its address,
// its machines and when each last published.
import type { ServerSearch } from "./serverStatus";
import { incidental, type Machine, type Service, type ServiceState } from "./services.svelte";

export type PlaceId =
  | "histories"
  | "indexing"
  | "sqlite"
  | "search"
  | "mcp"
  | "dashboard"
  | "server"
  | "serverSearch";

/** One place on the map and the services the machine reported for it. */
export interface Place {
  readonly id: PlaceId;
  readonly label: string;
  /** Empty for the agents' history files, which are files, not a service. */
  readonly services: readonly Service[];
  /** The worst state among its services; null for history files and for a
   * place whose only rows are not faults. */
  readonly state: ServiceState | null;
  /** A short line under the name. */
  readonly summary: string;
  /** Set when a place this one needs is down: what it needs and why. */
  readonly affectedBy: PlaceId | null;
  /** Nothing reports this place's state, so connections into it are not confirmed. */
  readonly unreported?: boolean;
}

/** How a connection looks: working, broken by a fault at the end it needs,
 * or not confirmed because that end's state is unknown. */
export type EdgeState = "ok" | "broken" | "unconfirmed";

export interface Edge {
  readonly from: PlaceId;
  readonly to: PlaceId;
  readonly verb: string;
  readonly state: EdgeState;
}

export interface MachineMap {
  readonly machine: Machine;
  readonly places: ReadonlyMap<PlaceId, Place>;
  readonly edges: readonly Edge[];
}

const kindPlace: Record<Service["kind"], PlaceId> = {
  watcher: "indexing",
  sqlite: "sqlite",
  qdrant: "search",
  mcp: "mcp",
  dashboard: "dashboard",
  server: "server",
};

export const placeLabel: Record<PlaceId, string> = {
  histories: "Agent histories",
  indexing: "Automatic indexing",
  sqlite: "SQLite index",
  search: "Local search",
  mcp: "Mash MCP",
  dashboard: "Dashboard",
  server: "Shared server",
  serverSearch: "Search on server",
};

/**
 * Mash's pipeline, written once. `needs` names the end that must be working
 * for the connection to work: indexing needs the index it writes, search is
 * built into the search service, a reader needs search, and publishing needs
 * the shared server. The other end being down never breaks the connection.
 */
const pipeline: readonly { from: PlaceId; to: PlaceId; verb: string; needs: PlaceId }[] = [
  { from: "histories", to: "indexing", verb: "reads", needs: "indexing" },
  { from: "indexing", to: "sqlite", verb: "writes", needs: "sqlite" },
  { from: "sqlite", to: "search", verb: "builds", needs: "search" },
  { from: "mcp", to: "search", verb: "searches", needs: "search" },
  { from: "dashboard", to: "search", verb: "searches", needs: "search" },
  { from: "indexing", to: "server", verb: "publishes", needs: "server" },
  { from: "server", to: "serverSearch", verb: "builds", needs: "serverSearch" },
];

const severity: readonly ServiceState[] = ["failed", "unknown", "degraded", "stopped", "healthy"];

/** The worst state among services; a sleeping agent session or an optional
 * watcher nobody supervises does not count against its place. */
export function worst(services: readonly Service[]): ServiceState | null {
  const counted = services.filter((service) => !incidental(service));
  // Only incidental rows (say, an optional watcher nothing runs): no verdict.
  const states = counted.map((service) => service.state);
  return severity.find((state) => states.includes(state)) ?? null;
}

function summary(id: PlaceId, services: readonly Service[]): string {
  if (id === "mcp") {
    const sessions = `${services.length} session${services.length === 1 ? "" : "s"}`;
    // An agent's session older than the installed build is not a fault;
    // the agent reconnects it.
    return services.some(incidental) ? `${sessions} · reconnect to update` : sessions;
  }
  if (services.length === 1) return services[0]?.summary ?? "";
  return `${services.length} reported`;
}

function edgeState(target: Place | undefined): EdgeState {
  if (target?.unreported) return "unconfirmed";
  const state = target?.state;
  if (state === "stopped" || state === "failed") return "broken";
  if (state === "unknown" || state === "degraded") return "unconfirmed";
  return "ok";
}

/** The map of one machine from its latest service check. `serverSearch` adds
 * the shared server's search index: `false` when there is no shared server,
 * `null` when the server doesn't report the index's state, otherwise that
 * state. Ready is healthy; missing or stale documents are drawn as degraded,
 * since searching those machines by meaning fails until they catch up. */
export function machineMap(
  machine: Machine,
  serverSearch: ServerSearch | null | false = false,
): MachineMap {
  const grouped = new Map<PlaceId, Service[]>();
  for (const service of machine.services) {
    const id = kindPlace[service.kind];
    grouped.set(id, [...(grouped.get(id) ?? []), service]);
  }
  const places = new Map<PlaceId, Place>();
  // Agent history files are where everything starts. They are drawn with the
  // indexing that reads them; they have no state of their own.
  if (machine.reachable && grouped.has("indexing"))
    places.set("histories", {
      id: "histories",
      label: placeLabel.histories,
      services: [],
      state: null,
      summary: "Files your agents write",
      affectedBy: null,
    });
  for (const [id, services] of grouped)
    places.set(id, {
      id,
      label: placeLabel[id],
      services,
      state: worst(services),
      summary: summary(id, services),
      affectedBy: null,
    });
  if (serverSearch !== false && places.has("server"))
    places.set(
      "serverSearch",
      serverSearch
        ? {
            id: "serverSearch",
            label: placeLabel.serverSearch,
            services: [],
            state: serverSearch.state === "ready" ? "healthy" : "degraded",
            summary: serverSearch.summary,
            affectedBy: null,
          }
        : {
            id: "serverSearch",
            label: placeLabel.serverSearch,
            services: [],
            state: null,
            summary: "State not reported",
            affectedBy: null,
            unreported: true,
          },
    );
  const edges: Edge[] = [];
  for (const link of pipeline) {
    if (!places.has(link.from) || !places.has(link.to)) continue;
    const state = edgeState(places.get(link.needs));
    edges.push({ from: link.from, to: link.to, verb: link.verb, state });
    // A reader whose search is down is affected; it is not itself at fault.
    const reader = places.get(link.from);
    if (state === "broken" && reader && (link.from === "mcp" || link.from === "dashboard"))
      places.set(link.from, { ...reader, affectedBy: link.needs });
  }
  return { machine, places, edges };
}

/** Places a fault on this machine makes unusable, for the side panel's
 * "what this stops" list. */
export function dependents(map: MachineMap, id: PlaceId): Place[] {
  return [...map.places.values()].filter((place) => place.affectedBy === id);
}

/** The address of a place on the map: its machine, and the place to open. */
export function placeHref(machine: string, place: PlaceId | "" = ""): string {
  return `#/services?${new URLSearchParams({ ...(machine ? { machine } : {}), ...(place ? { place } : {}) })}`;
}
