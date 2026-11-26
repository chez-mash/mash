<script lang="ts">
import ClockIcon from "phosphor-svelte/lib/ClockIcon";
import Icon from "$lib/components/Icon.svelte";
import { type Card, cardHealth, cardTone, tone } from "../lib/mapBand";
import { publishWords, staleWords } from "../lib/serverStatus";
import { dotTone } from "../lib/tone";
import { type Place, placeHref } from "../lib/topology";
import MachineIcon from "./MachineIcon.svelte";
import { placeIcon } from "./ServicesMap.svelte";
import ToneIcon from "./ToneIcon.svelte";

// One box on the services map: a place on the chosen machine, which links to
// its details, or another machine, which opens that machine's own map. A
// machine only the shared server knows of has no map, so it is not a link.
let {
  place,
  card,
  href = "",
  selected = false,
  status = "",
  now,
  spot = false,
}: {
  place?: Place;
  card?: Card;
  href?: string;
  selected?: boolean;
  /** What the place's last check said, in words. */
  status?: string;
  now: number;
  /** Placed on the desktop map, where every box has a fixed height. */
  spot?: boolean;
} = $props();
</script>

{#snippet cardBody(card: Card)}
  <MachineIcon machine={card.id} size={16} />
  <span class="nt">
    <strong>{card.id}</strong>
    {#if card.fleet}<small class="tone-{dotTone(cardTone(card))}"><ToneIcon tone={dotTone(cardTone(card))} size={12} /><span>{cardHealth(card)}</span></small>{/if}
    {#if card.reported}
      {@const stale = staleWords(card.reported, now)}
      {#if stale}<span class="pill warn stale" title={publishWords(card.reported, now)}><Icon icon={ClockIcon} size={11} />{stale}</span>{:else}<small class="pub">{publishWords(card.reported, now)}</small>{/if}
    {:else if !card.fleet}<small>Not reported</small>{/if}
  </span>
{/snippet}

{#if place}
  {@const Glyph = placeIcon[place.id]}
  <a class="node {tone(place.state)}" class:spot class:affected={!!place.affectedBy} class:sel={selected} {href} aria-current={selected ? "true" : undefined}>
    <Icon icon={Glyph} size={17} />
    <span class="nt"><strong>{place.label}</strong><small title={status} class={place.state === null ? "" : `tone-${dotTone(tone(place.state))}`}>{#if place.state !== null}<ToneIcon tone={dotTone(tone(place.state))} size={12} />{/if}<span>{status}</span></small></span>
  </a>
{:else if card?.fleet}
  <a class="node small {cardTone(card)}" class:spot href={placeHref(card.id)}>{@render cardBody(card)}</a>
{:else if card}
  <div class="node small neutral" class:spot>{@render cardBody(card)}</div>
{/if}

<style>
  .node {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 64px;
    padding: 10px 12px;
    border: var(--border-divider);
    border-radius: var(--radius-7);
    background: var(--surface-well);
    color: var(--text-strong);
    text-decoration: none;
  }
  .node > :global(svg) {
    flex-shrink: 0;
    color: var(--text-muted);
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
  /* A place that needs attention stands out from the quiet wells: raised,
     with a butter ring; a failed one keeps the red tint. */
  .node.warn {
    border-color: transparent;
    background: var(--surface-raised);
    box-shadow:
      0 0 0 1px var(--memory-ring),
      var(--memory-glow);
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
    align-items: center;
    gap: 5px;
    color: var(--text-muted);
    font-weight: var(--weight-label);
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  /* On the map every box has the height the connectors are drawn for. A long
     name wraps to a second line rather than being cut; another machine's
     card keeps to one line each. */
  .spot {
    height: 100%;
    overflow: hidden;
    padding: 10px 14px;
  }
  .spot .nt strong,
  .spot .nt small > span:not(.tone-i) {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
  .spot .nt strong {
    font-size: 13px;
    font-weight: var(--weight-strong);
    line-height: 1.25;
  }
  .spot.small .nt strong,
  .spot.small .nt small > span:not(.tone-i) {
    -webkit-line-clamp: 1;
    line-clamp: 1;
  }
  .nt small.tone-warn {
    color: var(--status-warn-text);
  }
  .nt small.tone-bad {
    color: var(--status-bad-text);
  }
  .pub {
    color: var(--text-muted);
  }
  .stale {
    align-self: flex-start;
    height: 18px;
    margin-top: 2px;
    font-size: 11px;
  }
</style>
