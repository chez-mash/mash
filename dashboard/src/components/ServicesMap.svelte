<script module lang="ts">
import DatabaseIcon from "phosphor-svelte/lib/DatabaseIcon";
import EyeIcon from "phosphor-svelte/lib/EyeIcon";
import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import LayoutIcon from "phosphor-svelte/lib/LayoutIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import PlugIcon from "phosphor-svelte/lib/PlugIcon";
import type { PlaceId } from "../lib/topology";

export const placeIcon: Record<PlaceId, typeof DatabaseIcon> = {
  histories: FileTextIcon,
  indexing: EyeIcon,
  sqlite: DatabaseIcon,
  search: MagnifyingGlassIcon,
  mcp: PlugIcon,
  dashboard: LayoutIcon,
  server: HardDrivesIcon,
  serverSearch: MagnifyingGlassIcon,
};
</script>

<script lang="ts">
import MapNode from "./MapNode.svelte";
import { bandCards, bandWords, publishes, publishVerb } from "../lib/mapBand";
import { reported, type ServerReport } from "../lib/serverStatus";
import type { Machine } from "../lib/services.svelte";
import ToneIcon from "./ToneIcon.svelte";
import { type Edge, type MachineMap, type Place, placeHref, placeLabel } from "../lib/topology";

// Every machine's services as one picture: the chosen machine's pipeline in
// full, the shared server above it, and the other machines as small cards
// that open their own map. Each place is a link to its details. The shared
// server's own report adds its address, its machines and when each published;
// a machine only the server knows of has a card but no map to open.
let {
  map,
  others,
  selected,
  report = { kind: "none" },
}: {
  map: MachineMap;
  others: readonly Machine[];
  selected: string;
  report?: ServerReport;
} = $props();

const id = $derived(map.machine.id);
const places = $derived(map.places);
const server = $derived(places.get("server"));
const readers = $derived((["mcp", "dashboard"] as const).filter((reader) => places.has(reader)));
const serverSearch = $derived(places.get("serverSearch"));
const now = $derived(report.kind === "reported" ? report.now : Math.floor(Date.now() / 1000));

const cards = $derived(bandCards(others, report, id));
const shared = $derived(bandWords(report));
const ownPublish = $derived(reported(report, id));

// The desktop map is laid out on a fixed grid, in pixels of a 600-wide
// canvas; the phone flow below needs no coordinates. The pipeline runs along
// the top row; the readers sit under it on the left, each with its own line
// across to search, which is as tall as the readers' rows together.
const H = 76;
const gap = 12;
const band = $derived(
  server || cards.length ? Math.max(server ? 40 + H : 0, 40 + cards.length * (H + 14)) + 16 : 0,
);
const top = $derived(band ? band + 20 : 0);
const at = $derived.by(() => {
  const row = top + (band ? 40 : 0);
  const below = row + H + 56;
  const spots: Partial<Record<PlaceId, Spot>> = {
    server: { x: 218, y: 40, w: 164, h: H },
    serverSearch: { x: 436, y: 40, w: 164, h: H },
    histories: { x: 0, y: row, w: 164, h: H },
    indexing: { x: 218, y: row, w: 164, h: H },
    sqlite: { x: 436, y: row, w: 164, h: H },
    search: { x: 436, y: below, w: 164, h: Math.max(1, readers.length) * (H + gap) - gap },
  };
  readers.forEach((reader, index) => {
    spots[reader] = { x: 0, y: below + index * (H + gap), w: 164, h: H };
  });
  return spots;
});
const height = $derived(
  (at.search ? at.search.y + at.search.h : (at.sqlite?.y ?? top) + H) + (band ? 16 : 0),
);
const other = (index: number): Spot => ({ x: 0, y: 40 + index * (H + 14), w: 164, h: H });

type Point = [number, number];
type Spot = { x: number; y: number; w: number; h: number };
/** The path of a connection between two places, and where its label goes. */
function route(edge: Edge): { points: Point[]; label: Point; middle?: boolean } | null {
  const a = at[edge.from],
    b = at[edge.to];
  if (!a || !b) return null;
  if (edge.to === "server") {
    const from: Point = [a.x + a.w / 2, a.y],
      to: Point = [b.x + b.w / 2, b.y + b.h],
      mid = (from[1] + to[1]) / 2;
    return { points: [from, [from[0], mid], [to[0], mid], to], label: [to[0] + 8, mid - 6] };
  }
  // Side by side, at the height of the place the line starts from: the
  // pipeline's steps, and each reader beside the tall search box.
  if (a.y + a.h <= b.y + b.h && a.y >= b.y) {
    const y = a.y + a.h / 2;
    return {
      points: [[a.x + a.w, y], [b.x, y]],
      label: [(a.x + a.w + b.x) / 2, y - 8],
      middle: true,
    };
  }
  return {
    points: [[a.x + a.w / 2, a.y + a.h], [b.x + b.w / 2, b.y]],
    label: [a.x + a.w / 2 + 8, (a.y + a.h + b.y) / 2 + 4],
  };
}
const routes = $derived(map.edges.flatMap((edge) => {
  const path = route(edge);
  return path ? [{ edge, ...path }] : [];
}));
// Every line has its own label; none are shared.
const verbLabels = $derived(routes);
function statusText(place: Place): string {
  if (place.id === "server" && report.kind === "reported") {
    const count = report.machines.filter((machine) => !machine.removed).length;
    return `${report.group} · ${count} machine${count === 1 ? "" : "s"}`;
  }
  if (place.affectedBy)
    return `${place.summary} · ${placeLabel[place.affectedBy].toLowerCase()} is down`;
  return place.summary;
}
function edgeWords(edge: Edge): string {
  const state =
    edge.state === "broken" ? "broken" : edge.state === "unconfirmed" ? "not confirmed" : "working";
  return `${placeLabel[edge.from]} ${edge.verb} ${placeLabel[edge.to]}: ${state}`;
}
const verbText = (edge: Edge) =>
  edge.to === "server" ? publishVerb(edge.verb, ownPublish, now) : edge.verb;
</script>


<section class="card pad0 smap" aria-label="How your services connect">
  <header class="smap-h"><h2>How your services connect</h2><span class="meta">This machine · {id}{cards.length ? ` · ${cards.length + 1} machines` : ""}</span></header>
  <!-- Desktop: the map. -->
  <div class="canvas-wrap">
    <div class="canvas" style:height="{height}px">
      {#if band}
        <div class="band" style:top="0px" style:height="{band}px"><span class="band-l">{server ? "Shared server" : "Other machines"}{#if server}<span class="meta">&nbsp;· {shared}</span>{/if}</span></div>
      {/if}
      {#if band}<div class="band" style:top="{top}px" style:height="{height - top - 8}px"><span class="band-l">This machine<span class="meta">&nbsp;· {id}</span></span></div>{/if}
      <svg class="edges" width="600" height={height} aria-hidden="true">
        <defs>
          <marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L8 4L0 8z" class="arrowhead" /></marker>
        </defs>
        {#each routes as path (`${path.edge.from}>${path.edge.to}`)}
          <polyline class="edge {path.edge.state}" points={path.points.map((point) => point.join(",")).join(" ")} marker-end="url(#arrow)" />
        {/each}
        {#each cards as entry, index (entry.id)}
          {#if server && publishes(entry)}
            {@const card = other(index)}
            <polyline class="edge ok" points="{card.w},{card.y + H / 2} {card.w + 30},{card.y + H / 2} {card.w + 30},{40 + H / 2} 218,{40 + H / 2}" marker-end="url(#arrow)" />
          {/if}
        {/each}
        {#each verbLabels as path (`${path.edge.from}>${path.edge.to}`)}
          <text class="edge-l {path.edge.state}" x={path.label[0]} y={path.label[1]} text-anchor={path.middle ? "middle" : "start"}>{verbText(path.edge)}</text>
        {/each}
      </svg>
      {#each cards as entry, index (entry.id)}
        {@const card = other(index)}
        <div class="spot" style:left="{card.x}px" style:top="{card.y}px" style:width="{card.w}px" style:height="{card.h}px"><MapNode card={entry} {now} spot /></div>
      {/each}
      {#each [...places.values()] as place (place.id)}
        {@const spot = at[place.id]}
        {#if spot}<div class="spot" style:left="{spot.x}px" style:top="{spot.y}px" style:width="{spot.w}px" style:height="{spot.h}px"><MapNode {place} href={placeHref(id, place.id)} selected={selected === place.id} status={statusText(place)} {now} spot /></div>{/if}
      {/each}
    </div>
  </div>
  <!-- Phone: the same places as one flow, top to bottom. -->
  <ol class="flow-list">
    {#if server || cards.length}
      <li class="flow-group">
        <span class="band-l">{server ? "Shared server" : "Other machines"}{#if server}<span class="meta">&nbsp;· {shared}</span>{/if}</span>
        {#if server}<MapNode place={server} href={placeHref(id, "server")} selected={selected === "server"} status={statusText(server)} {now} />{/if}
        {#if serverSearch}
          <span class="link {map.edges.find((edge) => edge.to === 'serverSearch')?.state ?? 'ok'}">builds</span>
          <MapNode place={serverSearch} href={placeHref(id, "serverSearch")} selected={selected === "serverSearch"} status={statusText(serverSearch)} {now} />
        {/if}
        {#if cards.length}<p class="also">Also publishing here</p><div class="flow-pair">{#each cards as entry (entry.id)}<MapNode card={entry} {now} />{/each}</div>{/if}
      </li>
      {#if server}{@const publish = map.edges.find((edge) => edge.to === "server")}<li class="link {publish?.state ?? 'ok'}">{id} {publish ? verbText(publish) : "publishes"} ↑</li>{/if}
    {/if}
    <!-- Alone, this machine needs no band: the header already names it. -->
    <li class="flow-group" class:plain={!server && !cards.length}>
      {#if server || cards.length}<span class="band-l">This machine<span class="meta">&nbsp;· {id}</span></span>{/if}
      {#each (["histories", "indexing", "sqlite", "search"] as const).filter((step) => places.has(step)) as step, index (step)}
        {@const into = map.edges.find((edge) => edge.to === step && edge.from !== "mcp" && edge.from !== "dashboard")}
        {#if index && into}<span class="link {into.state}">{into.verb}</span>{/if}
        {@const place = places.get(step)}
        {#if place}<MapNode {place} href={placeHref(id, place.id)} selected={selected === place.id} status={statusText(place)} {now} />{/if}
      {/each}
      {#if readers.length}
        {@const reads = map.edges.find((edge) => edge.from === readers[0])}
        <span class="link up {reads?.state ?? 'ok'}">searched by</span>
        <div class="flow-pair">{#each readers as reader (reader)}{@const place = places.get(reader)}{#if place}<MapNode {place} href={placeHref(id, place.id)} selected={selected === place.id} status={statusText(place)} {now} />{/if}{/each}</div>
      {/if}
    </li>
  </ol>
  <ul class="sr-only" aria-label="Connections">
    {#each map.edges as edge (`${edge.from}>${edge.to}`)}<li>{edgeWords(edge)}</li>{/each}
  </ul>
  <footer class="legend meta" aria-hidden="true">
    <span><ToneIcon tone="ok" size={13} />healthy</span><span><ToneIcon tone="warn" size={13} />stopped or needs attention</span><span><ToneIcon tone="bad" size={13} />failed</span><span><ToneIcon tone="unknown" size={13} />unknown</span>
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
    align-items: center;
    min-height: 46px;
    padding: 10px 18px;
    border-bottom: var(--border-divider);
  }
  .smap-h h2 {
    margin: 0;
    color: var(--text-strong);
    font-size: 14px;
    font-weight: var(--weight-strong);
  }
  .smap-h .meta {
    color: var(--text-faint);
    font-size: 12px;
  }
  .canvas-wrap {
    overflow-x: auto;
    padding: 20px 18px;
  }
  .canvas {
    position: relative;
    width: 600px;
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
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .edges {
    position: absolute;
    inset: 0;
    overflow: visible;
  }
  .edge {
    fill: none;
    stroke: var(--text-faint);
    stroke-width: 1;
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
    fill: var(--text-faint);
  }
  .edge-l {
    font: var(--weight-regular) 11px var(--type-sans);
    fill: var(--text-faint);
  }
  .edge-l.broken {
    fill: var(--status-bad-text);
  }
  .spot {
    position: absolute;
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
  .flow-group.plain {
    padding: 0;
    background: transparent;
  }
  .also {
    margin: 12px 0 6px;
    color: var(--text-muted);
    font: var(--weight-strong) 11px var(--type-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
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
    font-weight: var(--weight-strong);
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
    padding: 12px 18px;
    border-top: var(--border-divider);
    color: var(--text-muted);
    font-size: 12px;
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
