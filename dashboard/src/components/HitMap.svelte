<script lang="ts">
import type { FindMatch } from "../lib/find";

// Where Find in chat's matches fall in the whole chat: a column of ticks from
// the first message to the last, the ones holding a match in butter and the
// one in focus stronger. A long chat shares each tick among several messages.
let {
  count,
  matches,
  current,
}: {
  /** Messages in the whole chat. */
  count: number;
  matches: readonly FindMatch[];
  /** The match in focus, or -1 before the reader moves to one. */
  current: number;
} = $props();
const most = 40;
const ticks = $derived.by(() => {
  const size = Math.max(1, Math.min(count, most));
  const per = Math.max(1, count) / size;
  const hit = new Set(matches.map((match) => Math.floor(match.position / per)));
  const focus = matches[current];
  const at = focus ? Math.floor(focus.position / per) : -1;
  return Array.from({ length: size }, (_, index) => ({ hit: hit.has(index), on: index === at }));
});
const label = $derived(`${matches.length.toLocaleString()} hit${matches.length === 1 ? "" : "s"}`);
</script>

<div class="hm" role="img" aria-label="{label} in this chat">
  <span class="hm-n" aria-hidden="true">{label}</span>
  <span class="hm-t" aria-hidden="true">
    {#each ticks as tick, index (index)}<i class:hit={tick.hit} class:on={tick.on}></i>{/each}
  </span>
</div>

<style>
  .hm {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    width: 52px;
  }
  .hm-n {
    font: 11px / 1 var(--type-mono);
    color: var(--text-faint);
    white-space: nowrap;
  }
  .hm-t {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .hm-t i {
    display: block;
    width: 12px;
    height: 2px;
    border-radius: var(--radius-1);
    background: var(--tick);
  }
  .hm-t i.hit {
    width: 22px;
    background: var(--tick-hit);
  }
  .hm-t i.on {
    width: 28px;
    height: 3px;
    box-shadow: var(--find-glow);
  }
</style>
