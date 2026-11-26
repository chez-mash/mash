<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { onMount } from "svelte";
import { appLink, mayOpenInApp, resumeCommand } from "../lib/openIn";
import { stableRead } from "../lib/queries";
import {
  ariaShortcut,
  chatMemoryParams,
  pageTarget,
  type QuickKeys,
  type QuickRow,
  rowTarget,
} from "../lib/quick";
import { recordsHref } from "../lib/route.svelte";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import Keycaps from "./Keycaps.svelte";

// The quick panel's actions menu (⌘K): what the dashboard already offers for
// the selected chat, then Mash's own pages. Opening a chat in its agent and
// copying the resume command act on this computer, so they are offered only
// for a chat of this machine, as the chat page does (OpenInAgent.svelte).
let {
  row,
  local,
  memory,
  desktop,
  keys,
  go,
  oncopylink,
  onnotice,
  element = $bindable(),
}: {
  /** The selected chat, if a chat is selected. */
  row: QuickRow | undefined;
  /** The chat ran on this machine. */
  local: boolean;
  /** Memory is turned on, so a chat may have memory saved from it. */
  memory: boolean;
  desktop: boolean;
  keys: QuickKeys;
  go: (target: string) => void;
  oncopylink: () => void;
  onnotice: (label: string, ok: boolean) => void;
  element?: HTMLDivElement | undefined;
} = $props();

const chat = $derived(row?.chat);
const asked = $derived(!!chat && local && mayOpenInApp(chat.agent, chat.id));
const link = createQuery(() => ({
  ...stableRead,
  enabled: asked,
  queryKey: ["open-in", chat?.agent, chat?.id, chat?.project],
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    chat ? appLink(chat, signal) : Promise.resolve(null),
}));
const command = $derived(chat && local ? resumeCommand(chat.agent, chat.id) : undefined);

interface Action {
  readonly label: string;
  readonly group: "chat" | "app";
  readonly href?: string;
  readonly run?: () => void;
  readonly shortcut?: string;
  readonly agent?: Agent;
}

async function copyCommand(text: string, agent: Agent): Promise<void> {
  try {
    if (!navigator.clipboard) throw new Error("no clipboard");
    await navigator.clipboard.writeText(text);
    // Claude Code finds a session by the folder it ran in.
    onnotice(
      agent === "claude-code" ? "Copied: run it in the chat’s folder" : "Resume command copied",
      true,
    );
  } catch {
    onnotice("Couldn’t copy the command", false);
  }
}

const actions = $derived.by((): Action[] => {
  const list: Action[] = [];
  if (row && chat) {
    list.push({
      label: "Open in Mash",
      group: "chat",
      href: rowTarget(row, desktop),
      shortcut: "↵",
    });
    if (asked && link.data)
      list.push({
        label: `${link.data.addsSession ? "Add to" : "Open in"} ${link.data.app}`,
        group: "chat",
        href: link.data.url,
        agent: chat.agent,
      });
    if (command) {
      const text = command;
      list.push({
        label: "Copy resume command",
        group: "chat",
        run: () => void copyCommand(text, chat.agent),
      });
    }
    list.push({ label: "Copy link", group: "chat", run: oncopylink, shortcut: keys.copy });
    if (memory) {
      const params = chatMemoryParams(chat, local);
      list.push({
        label: "Show memory from this chat",
        group: "chat",
        href: pageTarget(recordsHref(params), desktop),
      });
    }
  }
  list.push({
    label: "Open Mash",
    group: "app",
    href: pageTarget("#/", desktop),
    shortcut: keys.open,
  });
  list.push({
    label: "Settings…",
    group: "app",
    href: pageTarget("#/settings", desktop),
    shortcut: keys.settings,
  });
  return list;
});

let filter = $state("");
let field: HTMLInputElement | undefined = $state();
let active = $state(0);
const shown = $derived(
  actions.filter((action) => action.label.toLowerCase().includes(filter.trim().toLowerCase())),
);
$effect(() => {
  void filter;
  active = 0;
});
onMount(() => field?.focus());

function choose(action: Action | undefined): void {
  if (!action) return;
  if (action.href) go(action.href);
  else action.run?.();
}

function keydown(event: KeyboardEvent): void {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const step = event.key === "ArrowDown" ? 1 : -1;
    active = Math.min(shown.length - 1, Math.max(0, active + step));
  } else if (event.key === "Enter") {
    event.preventDefault();
    choose(shown[active]);
  }
}
</script>

<div bind:this={element} class="qa" role="menu" aria-label="Actions">
  {#each shown as action, index (action.label)}
    {#if index > 0 && action.group !== shown[index - 1]?.group}<div class="qa-rule" aria-hidden="true"></div>{/if}
    {#if action.href}
      <a role="menuitem" class="qa-i" class:on={index === active} href={action.href} aria-keyshortcuts={action.shortcut ? ariaShortcut(action.shortcut, keys.modifier) : undefined} onmouseenter={() => (active = index)} onclick={(event) => {
        event.preventDefault();
        choose(action);
      }}>
        {#if action.agent}<AgentMark agent={action.agent} size="sm" />{/if}
        <span class="qa-l">{action.label}</span>
        {#if action.shortcut}<Keycaps shortcut={action.shortcut} />{/if}
      </a>
    {:else}
      <button type="button" role="menuitem" class="qa-i" class:on={index === active} aria-keyshortcuts={action.shortcut ? ariaShortcut(action.shortcut, keys.modifier) : undefined} onmouseenter={() => (active = index)} onclick={() => choose(action)}>
        <span class="qa-l">{action.label}</span>
        {#if action.shortcut}<Keycaps shortcut={action.shortcut} />{/if}
      </button>
    {/if}
  {:else}
    <p class="qa-none">No actions match.</p>
  {/each}
  <label class="qa-f">
    <span class="sr-only">Search actions</span>
    <input bind:this={field} bind:value={filter} placeholder="Search actions…" autocomplete="off" spellcheck="false" onkeydown={keydown} />
  </label>
</div>

<style>
  .qa {
    position: absolute;
    right: 10px;
    bottom: 52px;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 1px;
    box-sizing: border-box;
    width: min(300px, calc(100% - 20px));
    max-height: calc(100% - 64px);
    overflow-y: auto;
    padding: 6px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-strong),
      var(--shadow-float);
  }
  .qa-i {
    display: flex;
    flex: none;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-5);
    background: none;
    color: var(--text-strong);
    font: var(--weight-label) 13px / 1 var(--type-sans);
    text-align: left;
    text-decoration: none;
    cursor: pointer;
  }
  .qa-i.on {
    background: var(--selected-soft-bg);
  }
  .qa-l {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .qa-rule {
    flex: none;
    height: var(--stroke-divider);
    margin: 5px 8px;
    background: var(--line-divider);
  }
  .qa-none {
    margin: 0;
    padding: 9px 10px;
    color: var(--text-faint);
  }
  .qa-f {
    display: flex;
    flex: none;
    margin-top: 5px;
    padding: 9px 10px 5px;
    border-top: var(--border-divider);
  }
  .qa-f input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--text-body);
    font: var(--weight-regular) 13px / 1 var(--type-sans);
  }
  .qa-f input::placeholder {
    color: var(--text-faint);
  }
</style>
