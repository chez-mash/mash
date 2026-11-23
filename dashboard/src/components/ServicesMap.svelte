<script module lang="ts">
import Database from "@lucide/svelte/icons/database";
import Eye from "@lucide/svelte/icons/eye";
import FileText from "@lucide/svelte/icons/file-text";
import LayoutDashboard from "@lucide/svelte/icons/layout-dashboard";
import Plug from "@lucide/svelte/icons/plug";
import Search from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import type { PlaceId } from "../lib/topology";

export const placeIcon: Record<PlaceId, typeof Database> = {
  histories: FileText,
  indexing: Eye,
  sqlite: Database,
  search: Search,
  mcp: Plug,
  dashboard: LayoutDashboard,
  server: Server,
};
</script>

<script lang="ts">
import Laptop from "@lucide/svelte/icons/laptop";
import { type Machine, machineState, type ServiceState, stateLabel } from "../lib/services.svelte";
import { type Edge, type MachineMap, type Place, placeHref, placeLabel } from "../lib/topology";

// Every machine's services as one picture: the chosen machine's pipeline in
// full, the shared server above it, and the other machines as small cards
// that open their own map. Each place is a link to its details.
let {
  map,
  others,
  selected,
}: { map: MachineMap; others: readonly Machine[]; selected: string } = $props();

const id = $derived(map.machine.id);
const places = $derived(map.places);
const server = $derived(places.get("server"));
const readers = $derived((["mcp", "dashboard"] as const).filter((reader) => places.has(reader)));

// The desktop map is laid out on a fixed grid, in pixels of a 660-wide
// canvas; the phone flow below needs no coordinates.
const H = 76;
const band = $derived(
  server || others.length ? Math.max(server ? 40 + H : 0, 40 + others.length * (H + 14)) + 40 : 0,
);
const top = $derived(band ? band + 20 : 0);
const at = $derived.by(() => {
  const spots: Partial<Record<PlaceId, { x: number; y: number; w: number }>> = {
    server: { x: 240, y: 40, w: 200 },
    histories: { x: 0, y: top + 40, w: 170 },
    indexing: { x: 215, y: top + 40, w: 190 },
    sqlite: { x: 450, y: top + 40, w: 210 },
    search: { x: 450, y: top + 160, w: 210 },
    mcp: { x: 0, y: top + 280, w: 170 },
    dashboard: { x: 215, y: top + 280, w: 190 },
  };
  return spots;
});
const height = $derived(top + (readers.length ? 280 + H : places.has("search") ? 160 + H : 40 + H) + 24);
const other = (index: number) => ({ x: 0, y: 40 + index * (H + 14), w: 170 });

type Point = [number, number];
/** The path of a connection between two places, and where its label goes. */
function route(edge: Edge): { points: Point[]; label: Point; middle?: boolean } | null {
  const a = at[edge.from],
    b = at[edge.to];
  if (!a || !b) return null;
  if (edge.to === "server") {
    const from: Point = [a.x + a.w / 2, a.y],
      to: Point = [b.x + b.w / 2, b.y + H],
      mid = (from[1] + to[1]) / 2;
    return { points: [from, [from[0], mid], [to[0], mid], to], label: [to[0] + 8, mid - 6] };
  }
  if (a.y === b.y) {
    const y = a.y + H / 2;
    return {
      points: [[a.x + a.w, y], [b.x, y]],
      label: [(a.x + a.w + b.x) / 2, y - 8],
      middle: true,
    };
  }
  if (edge.from === "sqlite")
    return {
      points: [[a.x + a.w / 2, a.y + H], [b.x + b.w / 2, b.y]],
      label: [a.x + a.w / 2 + 8, (a.y + H + b.y) / 2 + 4],
    };
  // A reader joins a shared line under search.
  const join = b.y + H + 28,
    x = a.x + a.w / 2;
  return {
    points: [[x, a.y], [x, join], [b.x + b.w / 2, join], [b.x + b.w / 2, b.y + H]],
    label: [b.x + b.w / 2 + 8, join - 6],
  };
}
const routes = $derived(map.edges.flatMap((edge) => {
  const path = route(edge);
  return path ? [{ edge, ...path }] : [];
}));
const verbLabels = $derived(
  // A shared line carries its verb once.
  routes.filter((path, index) => routes.findIndex((other) => other.edge.verb === path.edge.verb && other.edge.to === path.edge.to) === index),
);
function tone(state: ServiceState | null): string {
  if (state === null) return "neutral";
  if (state === "healthy") return "ok";
  if (state === "failed") return "bad";
  if (state === "unknown") return "unknown";
  return "warn";
}
function statusText(place: Place): string {
  if (place.affectedBy)
    return `${place.summary} · ${placeLabel[place.affectedBy].toLowerCase()} is down`;
  return place.summary;
}
function edgeWords(edge: Edge): string {
  const state =
    edge.state === "broken" ? "broken" : edge.state === "unconfirmed" ? "not confirmed" : "working";
  return `${placeLabel[edge.from]} ${edge.verb} ${placeLabel[edge.to]}: ${state}`;
}
const machineTone = (machine: Machine) =>
  machine.reachable ? tone(machineState(machine)) : "unknown";
const machineText = (machine: Machine) =>
  machine.reachable ? stateLabel[machineState(machine)] : "Did not answer";
</script>

{#snippet node(place: Place, flow = false)}
  {@const Icon = placeIcon[place.id]}
  <a
    class="node {tone(place.state)}"
    class:affected={!!place.affectedBy}
    class:sel={selected === place.id}
    class:flow
    href={placeHref(id, place.id)}
    aria-current={selected === place.id ? "true" : undefined}
  >
    <Icon size={17} aria-hidden="true" />
    <span class="nt"><strong>{place.label}</strong><small title={statusText(place)}>{#if place.state !== null}<i class="dot {tone(place.state)}" aria-hidden="true"></i>{/if}<span>{statusText(place)}</span></small></span>
  </a>
{/snippet}
{#snippet machineCard(machine: Machine, flow = false)}
  <a class="node small {machineTone(machine)}" class:flow href={placeHref(machine.id)}>
    <Laptop size={16} aria-hidden="true" />
    <span class="nt"><strong>{machine.id}</strong><small><i class="dot {machineTone(machine)}" aria-hidden="true"></i><span>{machineText(machine)}</span></small></span>
  </a>
{/snippet}

<section class="card pad0 smap" aria-label="How your services connect">
  <header class="smap-h"><strong>How your services connect</strong><span class="meta">{id}{others.length ? ` · ${others.length + 1} machines` : ""}</span></header>
  <!-- Desktop: the map. -->
  <div class="canvas-wrap">
    <div class="canvas" style:height="{height}px">
      {#if band}
        <div class="band" style:top="0px" style:height="{band}px"><span class="band-l">{server ? "Shared server" : "Other machines"}{#if server}<span class="meta">&nbsp;· address not reported</span>{/if}</span></div>
      {/if}
      <div class="band" style:top="{top}px" style:height="{height - top - 8}px"><span class="band-l">This machine<span class="meta">&nbsp;· {id}</span></span></div>
      <svg class="edges" width="660" height={height} aria-hidden="true">
        <defs>
          <marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L8 4L0 8z" class="arrowhead" /></marker>
        </defs>
        {#each routes as path (`${path.edge.from}>${path.edge.to}`)}
          <polyline class="edge {path.edge.state}" points={path.points.map((point) => point.join(",")).join(" ")} marker-end="url(#arrow)" />
        {/each}
        {#each others as machine, index (machine.id)}
          {#if server && machine.reachable && machine.services.some((entry) => entry.kind === "server")}
            {@const card = other(index)}
            <polyline class="edge ok" points="{card.w},{card.y + H / 2} {card.w + 40},{card.y + H / 2} {card.w + 40},{40 + H / 2} 240,{40 + H / 2}" marker-end="url(#arrow)" />
          {/if}
        {/each}
        {#each verbLabels as path (`${path.edge.from}>${path.edge.to}`)}
          <text class="edge-l {path.edge.state}" x={path.label[0]} y={path.label[1]} text-anchor={path.middle ? "middle" : "start"}>{path.edge.verb}</text>
        {/each}
      </svg>
      {#each others as machine, index (machine.id)}
        {@const card = other(index)}
        <div class="spot" style:left="{card.x}px" style:top="{card.y}px" style:width="{card.w}px">{@render machineCard(machine)}</div>
      {/each}
      {#each [...places.values()] as place (place.id)}
        {@const spot = at[place.id]}
        {#if spot}<div class="spot" style:left="{spot.x}px" style:top="{spot.y}px" style:width="{spot.w}px">{@render node(place)}</div>{/if}
      {/each}
    </div>
  </div>
  <!-- Phone: the same places as one flow, top to bottom. -->
  <ol class="flow-list">
    {#if server || others.length}
      <li class="flow-group">
        <span class="band-l">{server ? "Shared server" : "Other machines"}</span>
        {#if server}{@render node(server, true)}{/if}
        {#if others.length}<div class="flow-pair">{#each others as machine (machine.id)}{@render machineCard(machine, true)}{/each}</div>{/if}
      </li>
      {#if server}<li class="link {map.edges.find((edge) => edge.to === 'server')?.state ?? 'ok'}">{id} publishes ↑</li>{/if}
    {/if}
    <li class="flow-group">
      <span class="band-l">This machine<span class="meta">&nbsp;· {id}</span></span>
      {#each (["histories", "indexing", "sqlite", "search"] as const).filter((step) => places.has(step)) as step, index (step)}
        {@const into = map.edges.find((edge) => edge.to === step && edge.from !== "mcp" && edge.from !== "dashboard")}
        {#if index && into}<span class="link {into.state}">{into.verb}</span>{/if}
        {@const place = places.get(step)}
        {#if place}{@render node(place, true)}{/if}
      {/each}
      {#if readers.length}
        {@const reads = map.edges.find((edge) => edge.from === readers[0])}
        <span class="link up {reads?.state ?? 'ok'}">searched by</span>
        <div class="flow-pair">{#each readers as reader (reader)}{@const place = places.get(reader)}{#if place}{@render node(place, true)}{/if}{/each}</div>
      {/if}
    </li>
  </ol>
  <ul class="sr-only" aria-label="Connections">
    {#each map.edges as edge (`${edge.from}>${edge.to}`)}<li>{edgeWords(edge)}</li>{/each}
  </ul>
  <footer class="legend meta" aria-hidden="true">
    <span><i class="dot ok"></i>healthy</span><span><i class="dot warn"></i>stopped or needs attention</span><span><i class="dot bad"></i>failed</span><span><i class="dot none"></i>unknown</span>
    {#each [["ok", "works"], ["unconfirmed", "not confirmed"], ["broken", "broken by a fault"]] as [kind, words] (kind)}
      <span><svg width="24" height="6"><line class="edge {kind}" x1="0" y1="3" x2="24" y2="3" /></svg>{words}</span>
    {/each}
  </footer>
</section>

<style>
  .smap-h {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 4px 12px;
    padding: 12px 16px;
    border-bottom: var(--border-strong);
    background: var(--surface-muted);
  }
  .canvas-wrap {
    overflow-x: auto;
    padding: 18px 16px 8px;
  }
  .canvas {
    position: relative;
    width: 660px;
    margin: 0 auto;
  }
  .band {
    position: absolute;
    left: -8px;
    right: -8px;
    border-radius: var(--radius-7);
    background: var(--surface-muted);
  }
  .band-l {
    display: block;
    padding: 10px 12px;
    font-size: 13px;
    font-weight: 700;
    color: var(--text-strong);
  }
  .edges {
    position: absolute;
    inset: 0;
    overflow: visible;
  }
  .edge {
    fill: none;
    stroke: var(--line-ink);
    stroke-width: 1.5;
  }
  .edge.broken {
    stroke: var(--status-bad);
    stroke-dasharray: 5 4;
  }
  .edge.unconfirmed {
    stroke: var(--text-muted);
    stroke-dasharray: 5 4;
  }
  .arrowhead {
    fill: var(--line-ink);
  }
  .edge-l {
    font: 700 12px var(--type-sans);
    fill: var(--text-muted);
  }
  .edge-l.broken {
    fill: var(--status-bad-text);
  }
  .spot {
    position: absolute;
  }
  /* On the map every box has the height the connectors are drawn for. */
  .spot .node {
    height: 76px;
    overflow: hidden;
  }
  .spot .nt strong {
    font-size: 14px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .spot .nt small > span {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
  .node {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 64px;
    padding: 10px 12px;
    border: var(--border-strong);
    border-radius: var(--radius-6);
    background: var(--surface-raised);
    color: var(--text-strong);
    text-decoration: none;
  }
  .node:hover {
    background: var(--surface-hover);
  }
  .node:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }
  .node.small {
    min-height: 56px;
  }
  .node.warn,
  .node.bad {
    background: var(--status-warn-bg);
  }
  .node.bad {
    background: var(--status-bad-bg);
  }
  .node.unknown,
  .node.affected {
    border-style: dashed;
  }
  .node.sel {
    outline: 3px solid var(--line-ink);
    outline-offset: 3px;
  }
  .node.sel.ok {
    background: var(--selected-surface);
  }
  .nt {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .nt small {
    display: flex;
    align-items: baseline;
    gap: 6px;
    color: var(--text-body);
    font-size: 12.5px;
    overflow-wrap: anywhere;
  }
  .nt small .dot {
    transform: translateY(1px);
  }
  .dot.ok {
    color: var(--status-ok-dot);
  }
  .dot.unknown,
  .dot.neutral {
    background: transparent;
    border-color: var(--text-muted);
  }
  .flow-list,
  .sr-only {
    list-style: none;
  }
  .flow-list {
    display: none;
    margin: 0;
    padding: 14px;
  }
  .flow-group {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 4px 10px 12px;
    border-radius: var(--radius-7);
    background: var(--surface-muted);
  }
  .flow-pair {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
  }
  .link {
    margin-left: 22px;
    padding: 6px 0 6px 12px;
    border-left: 2px solid var(--line-ink);
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
  }
  .link.broken {
    border-left: 2px dashed var(--status-bad);
    color: var(--status-bad-text);
  }
  .link.unconfirmed {
    border-left-style: dashed;
  }
  li.link {
    margin: 0 0 0 32px;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    padding: 10px 16px;
    border-top: var(--border-divider);
  }
  .legend span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  @media (max-width: 900px) {
    .canvas-wrap {
      display: none;
    }
    .flow-list {
      display: flex;
      flex-direction: column;
    }
  }
</style>
