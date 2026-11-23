import { describe, expect, it } from "vitest";
import type { Machine, Service } from "./services.svelte";
import { dependents, machineMap, placeHref } from "./topology";

function service(id: string, kind: Service["kind"], state: Service["state"] = "healthy"): Service {
  return {
    id,
    name: id,
    kind,
    state,
    summary: state,
    checked_at: 1,
    pid: null,
    actions: [],
    control_note: "",
    diagnostics: [],
    logs_available: false,
    control_revision: null,
    compatibility: null,
  };
}
function machine(services: Service[], reachable = true): Machine {
  return { id: "laptop", platform: "macos", checked_at: 1, reachable, services, warnings: [] };
}
const pipeline = [
  service("watcher-leadership", "watcher"),
  service("sqlite", "sqlite"),
  service("qdrant", "qdrant"),
  service("runtime-1", "mcp"),
  service("runtime-2", "mcp"),
];

describe("the services map", () => {
  it("places each reported service and connects only places that are present", () => {
    const map = machineMap(machine(pipeline));
    expect([...map.places.keys()].sort()).toEqual(
      ["histories", "indexing", "mcp", "search", "sqlite"].sort(),
    );
    expect(map.places.get("mcp")?.summary).toBe("2 sessions");
    expect(map.edges.map((edge) => `${edge.from}>${edge.to}:${edge.verb}`)).toEqual([
      "histories>indexing:reads",
      "indexing>sqlite:writes",
      "sqlite>search:builds",
      "mcp>search:searches",
    ]);
    expect(map.edges.every((edge) => edge.state === "ok")).toBe(true);
  });

  it("draws publishing only for a machine that reports the shared server", () => {
    const sharing = machineMap(machine([...pipeline, service("shared-server", "server")]));
    expect(sharing.edges.some((edge) => edge.to === "server" && edge.verb === "publishes")).toBe(
      true,
    );
    const alone = machineMap(machine(pipeline));
    expect(alone.places.has("server")).toBe(false);
    expect(alone.edges.some((edge) => edge.to === "server")).toBe(false);
  });

  it("marks what a stopped search breaks, without blaming the readers", () => {
    const map = machineMap(
      machine([
        ...pipeline.filter((entry) => entry.kind !== "qdrant"),
        service("qdrant", "qdrant", "stopped"),
      ]),
    );
    expect(map.edges.filter((edge) => edge.state === "broken").map((edge) => edge.from)).toEqual([
      "sqlite",
      "mcp",
    ]);
    expect(map.places.get("mcp")?.state).toBe("healthy");
    expect(map.places.get("mcp")?.affectedBy).toBe("search");
    expect(dependents(map, "search").map((place) => place.id)).toEqual(["mcp"]);
    // A fault upstream of search leaves the readers' connection alone.
    expect(map.places.get("indexing")?.affectedBy).toBeNull();
  });

  it("does not call a connection broken when the far end's state is unknown", () => {
    const map = machineMap(machine([...pipeline, service("shared-server", "server", "unknown")]));
    expect(map.edges.find((edge) => edge.to === "server")?.state).toBe("unconfirmed");
  });

  it("does not count an agent's out-of-date session against its place", () => {
    const old = { ...service("runtime-3", "mcp", "degraded"), summary: "Older build" };
    const map = machineMap(machine([...pipeline, old]));
    expect(map.places.get("mcp")?.state).toBe("healthy");
    expect(map.places.get("mcp")?.summary).toBe("3 sessions · reconnect to update");
  });

  it("gives no verdict for a place whose only row is not a fault", () => {
    const optional = service("watcher-leadership", "watcher", "stopped");
    const map = machineMap(machine([optional, service("sqlite", "sqlite")]));
    expect(map.places.get("indexing")?.state).toBeNull();
    expect(map.edges.find((edge) => edge.to === "indexing")?.state).toBe("ok");
  });

  it("shows nothing it was not told about for a machine that did not answer", () => {
    const map = machineMap(machine([], false));
    expect(map.places.size).toBe(0);
    expect(map.edges).toEqual([]);
  });

  it("links to a place on a machine", () => {
    expect(placeHref("laptop", "search")).toBe("#/services?machine=laptop&place=search");
    expect(placeHref("")).toBe("#/services?");
  });
});
