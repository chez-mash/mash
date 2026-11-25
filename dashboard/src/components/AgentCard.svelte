<script lang="ts">
import { agents as agentInfo } from "../lib/agents";
import { whenLabel } from "../lib/browse";
import { agentHref } from "../lib/route.svelte";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

// One agent as a card: its mark and name, whether this machine indexes it
// (from this machine's settings, when they can be read), how many chats it
// has and when the latest was, its share of all chats, where this machine
// finds its history, and the machines it has chats on. With `onindex` the
// card carries this machine's switch for indexing the agent; the switch
// writes the same setting as the Settings page.
let {
  agent,
  chats,
  total,
  latestMs,
  machines,
  setting,
  now,
  onindex,
  busy = false,
}: {
  agent: Agent;
  chats: number;
  total: number;
  latestMs: number | undefined;
  machines: readonly string[];
  /** This machine's settings for the agent, when they are known. */
  setting: { readonly indexed: boolean; readonly roots: readonly string[] } | undefined;
  now: number;
  /** Turns indexing this agent on this machine on or off; left out, no switch. */
  onindex?: ((on: boolean) => void) | undefined;
  busy?: boolean;
} = $props();
const status = $derived(
  setting === undefined
    ? undefined
    : setting.indexed
      ? { tone: "ok", words: "Indexed" }
      : { tone: "none", words: "Not indexed here" },
);
</script>

<!-- The name is the card's link and covers the whole card, so the switch
     beside it stays a control of its own. -->
<div class="card ac" class:quiet={!chats}>
  <span class="ac-h">
    <AgentMark {agent} />
    <span class="ac-n">
      <a class="ac-a" href={agentHref(agent)}><b>{agentInfo[agent].name}</b></a>
      {#if status}<span class="ac-s {status.tone}"><i class="dot {status.tone}" aria-hidden="true"></i>{status.words}</span>{/if}
    </span>
    {#if onindex && setting}
      <button class="sw ac-sw" type="button" role="switch" aria-checked={setting.indexed} aria-label="Index {agentInfo[agent].name} on this machine" disabled={busy} onclick={() => onindex(!setting.indexed)}><i></i></button>
    {/if}
  </span>
  <span class="ac-c">
    <span class="big"><b>{chats.toLocaleString()}</b><span>chat{chats === 1 ? "" : "s"}</span></span>
    <span class="ac-t">{latestMs === undefined ? "Never" : whenLabel(latestMs, now)}</span>
  </span>
  <span class="ac-b" aria-hidden="true"><i style:width="{chats ? Math.max(2, Math.round((chats / Math.max(1, total)) * 100)) : 0}%"></i></span>
  <span class="ac-f">
    {#if setting}<code title={setting.roots.join("\n")}>{setting.roots[0] ?? "No history found on this machine"}</code>{/if}
    <span>{machines.length ? machines.join(" · ") : "None yet"}</span>
  </span>
</div>

<style>
  .ac {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
    color: var(--text-body);
  }
  .ac {
    position: relative;
  }
  .ac:has(.ac-a:hover) {
    background: var(--surface-hover);
  }
  .ac:has(.ac-a:focus-visible) {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }
  .ac-a {
    min-width: 0;
    color: inherit;
    text-decoration: none;
    outline: 0;
  }
  .ac-a::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
  }
  .ac-sw {
    position: relative;
    z-index: 1;
    flex: none;
    min-height: 0;
    margin-left: auto;
    align-self: flex-start;
    border: 0;
    padding: 0;
    background: transparent;
    cursor: pointer;
  }
  .ac.quiet {
    background: var(--surface-rail);
    box-shadow: none;
  }
  .ac-h {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .ac-n {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .ac-n b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .ac-s {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .ac-s.ok {
    color: var(--status-ok-text);
  }
  .ac-c {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
  }
  .ac-c :global(.big b) {
    font-size: 28px;
    font-weight: 600;
    letter-spacing: -0.03em;
  }
  .ac-c :global(.big span) {
    font-size: 13px;
    color: var(--text-muted);
  }
  .ac-t {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  /* The agent's share of all chats: a thin track filling with butter. */
  .ac-b {
    display: block;
    height: 4px;
    border-radius: var(--radius-pill);
    background: var(--surface-muted);
    overflow: hidden;
  }
  .ac-b > i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--progress);
  }
  .ac-f {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    font-size: 12px;
    color: var(--text-muted);
  }
  .ac-f code {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 12px var(--type-mono);
    color: var(--text-body);
  }
  .ac-f > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
