<script lang="ts">
// The viewing switcher: whose history the overview, chats, search, memory,
// projects and agents show. It sits at the right of the top bar, and on a
// phone at the top of the navigation sheet. Services and Machines are
// about the machines themselves, so they show every machine whatever it says.
import Check from "@lucide/svelte/icons/check";
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import Globe from "@lucide/svelte/icons/globe";
import Laptop from "@lucide/svelte/icons/laptop";
import Network from "@lucide/svelte/icons/network";
import Radio from "@lucide/svelte/icons/radio";
import { createQuery } from "@tanstack/svelte-query";
import { groupChats } from "../lib/browse";
import { catalogOptions, liveOptions, reachOf } from "../lib/queries";
import { type Scope, sameScope, scope, scopeName, scopeSentence } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import {
  publishWords,
  reported,
  serverReport,
  serverViewOptions,
  staleWords,
} from "../lib/serverStatus";

/** Where it is drawn: the top bar's compact button, or the phone sheet's full-width field. */
let { place }: { place: "bar" | "sheet" } = $props();
const labelId = $derived(`viewing-label-${place}`);
const whole = createQuery(() => catalogOptions);
const liveList = createQuery(() => liveOptions);
const scoped = createScoped();
let open = $state(false);
let root: HTMLElement | undefined = $state();

const reach = $derived(whole.data ? reachOf(whole.data, liveList.data ?? []) : undefined);
// When each machine last published, as the shared server reports it.
const serverView = createQuery(() => ({
  ...serverViewOptions,
  enabled: reach?.joined === true && !reach.sharedOnly,
}));
const report = $derived(serverReport(serverView.data));
const serverNow = $derived(report.kind === "reported" ? report.now : 0);
/** For a copy the machine has not refreshed lately, how old it is ("5 days
 * old") and, for the tooltip, when it last published. */
function staleness(machine: string): { age: string; words: string } | undefined {
  const entry = reported(report, machine);
  const age = entry ? staleWords(entry, serverNow) : "";
  if (!entry || !age) return undefined;
  const words = publishWords(entry, serverNow);
  return { age, words: words.charAt(0).toUpperCase() + words.slice(1) };
}
const counts = $derived(
  new Map(
    groupChats(whole.data?.chats ?? [], (chat) => chat.machine).map((g) => [g.key, g.chats.length]),
  ),
);
/** Every other machine: those that published, and those read live. */
const others = $derived.by(() => {
  if (!reach) return [];
  const names = new Set([...counts.keys(), ...reach.live]);
  if (!reach.sharedOnly) names.delete(reach.localNode);
  return [...names].sort((a, b) => a.localeCompare(b));
});
const current = $derived(scoped.data?.scope);
const name = $derived(
  scoped.data
    ? scopeName(scoped.data.scope, {
        ...scoped.data.reach,
        live: scoped.data.live ? scoped.data.reach.live : [],
      })
    : "loading…",
);
/** The button names the machine in full; whether it is this machine, read
 * live or a published copy goes in a small pill beside it. */
const label = $derived.by((): { text: string; pill: string } => {
  if (!current || !scoped.data) return { text: name, pill: "" };
  if (current.kind === "local") return { text: scoped.data.reach.localNode, pill: "this machine" };
  if (current.kind === "node")
    return { text: current.node, pill: scoped.data.live ? "live" : "published" };
  return { text: name, pill: "" };
});
const total = $derived(whole.data?.chats.length ?? 0);
// What the current choice includes, said at the top of the menu; pages name
// the scope only briefly beside their subtitle.
const sentence = $derived(
  scoped.data ? scopeSentence(scoped.data.scope, scoped.data.reach, scoped.data.live) : "",
);

function choose(next: Scope): void {
  scope.choose(next);
  open = false;
}
const on = (candidate: Scope) => current !== undefined && sameScope(candidate, current);
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape" && open) open = false;
  }}
  onpointerdown={(event) => {
    const target = event.target;
    if (!open || !(target instanceof Node)) return;
    if (!root?.contains(target)) open = false;
  }}
/>

<div class="vw vw-{place}" bind:this={root}>
  <span class="vw-l" class:sr-only={place === "bar"} id={labelId}>Viewing</span>
  <button class="vw-b" type="button" aria-haspopup="true" aria-expanded={open} aria-describedby={labelId} onclick={() => (open = !open)}>
    {#if scoped.data?.live}<Radio size={15} aria-hidden="true" />{:else}<Laptop size={15} aria-hidden="true" />{/if}
    <span class="vw-n" title={name}>{label.text}</span>{#if label.pill}<span class="pill vw-p">{label.pill}</span>{/if}
    <ChevronDown size={14} aria-hidden="true" class="vw-c" />
  </button>
  {#if open && reach}
    <div class="vw-m" role="group" aria-label="Show chats and memory from">
      {#if sentence}<p class="vw-d">{sentence}</p>{/if}
      {#if !reach.sharedOnly}
        <button type="button" role="menuitemradio" aria-checked={on({ kind: "local" })} onclick={() => choose({ kind: "local" })}>
          <span class="vw-i">{#if on({ kind: "local" })}<Check size={14} aria-hidden="true" />{:else}<Laptop size={14} aria-hidden="true" />{/if}</span>
          <span class="vw-t"><b>This machine · {reach.localNode}</b></span>
          <span class="vw-k">{(counts.get(reach.localNode) ?? 0).toLocaleString()}</span>
        </button>
      {/if}
      {#if reach.joined || reach.sharedOnly}
        <button type="button" role="menuitemradio" aria-checked={on({ kind: "all" })} onclick={() => choose({ kind: "all" })}>
          <span class="vw-i">{#if on({ kind: "all" })}<Check size={14} aria-hidden="true" />{:else}<Globe size={14} aria-hidden="true" />{/if}</span>
          <span class="vw-t"><b>All machines</b></span>
          <span class="vw-k">{total.toLocaleString()}</span>
        </button>
      {/if}
      {#if others.length}
        <p class="vw-s">Other machines</p>
        {#each others as machine (machine)}
          {@const live = reach.live.includes(machine)}
          {@const candidate = { kind: "node", node: machine } as const}
          {@const stale = staleness(machine)}
          <button type="button" role="menuitemradio" aria-checked={on(candidate)} onclick={() => choose(candidate)}>
            <span class="vw-i">{#if on(candidate)}<Check size={14} aria-hidden="true" />{:else if live}<Radio size={14} aria-hidden="true" />{:else}<Laptop size={14} aria-hidden="true" />{/if}</span>
            <span class="vw-t"><b>{machine}</b>{#if live}<span class="pill vw-st" title="Read live from its own dashboard">live</span>{:else if stale}<span class="pill warn vw-st" title={stale.words}>{stale.age}</span>{/if}</span>
            {#if counts.has(machine)}<span class="vw-k">{(counts.get(machine) ?? 0).toLocaleString()}</span>{/if}
          </button>
        {/each}
      {/if}
      {#if !reach.joined && !reach.sharedOnly}
        <div class="vw-j">
          <p>See your other machines here by joining a shared index.</p>
          <!-- The machines page has the join form. -->
          <a class="btn sm" href="#/machines" onclick={() => (open = false)}><Network size={14} aria-hidden="true" />Join a shared index</a>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .vw {
    position: relative;
  }
  /* In the sheet the label sits inside the field, before the name. */
  .vw-l {
    flex: none;
    color: var(--text-muted);
    font-size: 13px;
  }
  .vw.vw-sheet .vw-l {
    position: absolute;
    top: 0;
    left: 12px;
    height: 44px;
    display: flex;
    align-items: center;
    pointer-events: none;
  }
  .vw-b {
    display: flex;
    align-items: center;
    gap: 7px;
    border-radius: var(--radius-5);
    color: var(--text-body);
    font-size: 13px;
    font-weight: 500;
    text-align: left;
  }
  .vw.vw-bar .vw-b {
    height: 30px;
    max-width: 260px;
    padding: 0 10px;
  }
  .vw.vw-bar .vw-b:hover,
  .vw.vw-bar .vw-b[aria-expanded="true"] {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .vw.vw-sheet .vw-b {
    width: 100%;
    min-height: 44px;
    padding: 0 12px 0 72px;
    border: var(--border-outline);
    border-radius: var(--radius-box);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pick);
    color: var(--text-strong);
    font-size: 14px;
  }
  .vw-n {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .vw-p {
    flex: none;
    height: 20px;
    padding: 0 7px;
    font-size: 11px;
  }
  .vw-st {
    flex: none;
    height: 18px;
    font-size: 11px;
  }
  .vw-b :global(.vw-c) {
    flex: none;
    color: var(--text-muted);
  }
  .vw-b[aria-expanded="true"] :global(.vw-c) {
    transform: rotate(180deg);
  }
  /* The menu drops below the button: under the bar's right edge, or as wide
     as the sheet's field. */
  .vw-m {
    position: absolute;
    z-index: 30;
    top: calc(100% + 6px);
    right: 0;
    width: 320px;
    max-width: calc(100vw - 32px);
    max-height: min(560px, 70vh);
    overflow-y: auto;
    padding: 6px;
    border: var(--border-outline);
    border-radius: var(--radius-box);
    background: var(--surface-raised);
    /* The page behind the open menu dims; the button stays bright above. */
    box-shadow:
      var(--shadow-float),
      0 0 0 100vmax var(--scrim);
  }
  .vw:has(> .vw-m) > .vw-b {
    position: relative;
    z-index: 31;
  }
  .vw.vw-bar:has(> .vw-m) > .vw-b {
    background: var(--surface-raised);
  }
  .vw.vw-sheet .vw-m {
    left: 0;
    width: auto;
    max-width: none;
  }
  .vw-m button {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 44px;
    padding: 9px 10px;
    border-radius: var(--radius-tile);
    text-align: left;
  }
  .vw-m button:hover {
    background: var(--surface-hover);
  }
  .vw-m button[aria-checked="true"] {
    background: var(--selected-bg);
    color: var(--selected-text);
  }
  .vw-i {
    flex: none;
    display: inline-flex;
  }
  .vw-t {
    display: flex;
    flex: 1;
    min-width: 0;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
  }
  /* A long name keeps its words whole; a pill beside it moves to the next line. */
  .vw-t b {
    font-size: 13.5px;
    overflow-wrap: break-word;
  }
  .vw-k {
    flex: none;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
  .vw-d {
    margin: 4px 10px 8px;
    padding-bottom: 10px;
    border-bottom: var(--border-divider);
    color: var(--text-muted);
    font-size: 12.5px;
    line-height: 1.45;
  }
  .vw-s {
    margin: 10px 10px 4px;
    color: var(--text-muted);
    font: 700 11px var(--type-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .vw-j {
    margin: 6px 4px;
    padding: 10px;
    border: var(--border-divider);
    border-radius: var(--radius-tile);
  }
  .vw-j p {
    margin: 0 0 8px;
    font-size: 12.5px;
  }
</style>
