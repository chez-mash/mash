<script lang="ts">
// The right-click menu the dashboard draws in a browser, where there is no
// system menu to ask for (lib/contextMenu.svelte.ts). It opens at the pointer,
// kept on screen, and closes on a choice, Escape, a click elsewhere or a scroll.
import { tick } from "svelte";
import { contextMenu, type MenuLine } from "../lib/contextMenu.svelte";

let menu: HTMLDivElement | undefined = $state();
let place = $state({ left: 0, top: 0 });

$effect(() => {
  const open = contextMenu.open;
  if (!open) return;
  place = { left: open.x, top: open.y };
  void tick().then(() => {
    if (!menu) return;
    const box = menu.getBoundingClientRect();
    place = {
      left: Math.max(8, Math.min(open.x, innerWidth - box.width - 8)),
      top: Math.max(8, Math.min(open.y, innerHeight - box.height - 8)),
    };
    menu.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  });
});

function choose(line: MenuLine): void {
  contextMenu.close();
  line.run();
}
function onkeydown(event: KeyboardEvent): void {
  if (!contextMenu.open) return;
  if (event.key === "Escape") {
    event.preventDefault();
    contextMenu.close();
    return;
  }
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  const buttons = [...(menu?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
  const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
  const next = (at + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
  buttons[next]?.focus();
}
function onpointerdown(event: PointerEvent): void {
  if (contextMenu.open && !menu?.contains(event.target as Node)) contextMenu.close();
}
</script>

<svelte:window {onkeydown} {onpointerdown} onscroll={() => contextMenu.close()} onblur={() => contextMenu.close()} />

{#if contextMenu.open}
  <div class="ctx" role="menu" tabindex="-1" bind:this={menu} style:left="{place.left}px" style:top="{place.top}px">
    {#each contextMenu.open.entries as entry, index (index)}
      {#if entry === "separator"}
        <hr />
      {:else}
        <button type="button" role="menuitem" disabled={entry.enabled === false} onclick={() => choose(entry)}>{entry.title}</button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .ctx {
    position: fixed;
    z-index: 80;
    display: flex;
    flex-direction: column;
    min-width: 200px;
    padding: 5px;
    border: var(--border-outline);
    border-radius: var(--radius-box);
    background: var(--surface-raised);
    box-shadow: var(--shadow-float);
  }
  button {
    padding: 5px 10px;
    border-radius: var(--radius-4);
    color: var(--text-strong);
    font-size: 13.5px;
    text-align: left;
  }
  button:hover:not(:disabled),
  button:focus-visible {
    outline: none;
    background: var(--surface-hover);
  }
  button:disabled {
    color: var(--text-faint);
  }
  hr {
    margin: 5px 8px;
    border: 0;
    border-top: var(--border-outline);
  }
</style>
