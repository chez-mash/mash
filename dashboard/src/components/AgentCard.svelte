<script lang="ts">
import { agents as agentInfo } from "../lib/agents";
import { whenLabel } from "../lib/browse";
import { agentHref } from "../lib/route.svelte";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

// One agent as a card: its mark and name, whether this machine indexes it
// (from this machine's settings, when they can be read), how many chats it
// has and when the latest was, its share of all chats, where this machine
// finds its history, and the machines it has chats on.
let {
  agent,
  chats,
  total,
  latestMs,
  machines,
  setting,
  now,
}: {
  agent: Agent;
  chats: number;
  total: number;
  latestMs: number | undefined;
  machines: readonly string[];
  /** This machine's settings for the agent, when they are known. */
  setting: { readonly indexed: boolean; readonly roots: readonly string[] } | undefined;
  now: number;
} = $props();
const status = $derived(
  setting === undefined
    ? undefined
    : setting.indexed
      ? { tone: "ok", words: "Indexed" }
      : { tone: "none", words: "Not indexed here" },
);
</script>

<a class="card ac" class:quiet={!chats} href={agentHref(agent)}>
  <span class="ac-h">
    <AgentMark {agent} size="lg" />
    <span class="ac-n">
      <b>{agentInfo[agent].name}</b>
      {#if status}<span class="ac-s {status.tone}"><i class="dot {status.tone}" aria-hidden="true"></i>{status.words}</span>{/if}
    </span>
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
</a>

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
  a.ac:hover {
    color: var(--text-body);
    background: var(--surface-hover);
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
