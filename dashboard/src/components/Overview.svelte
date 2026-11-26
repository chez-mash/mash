<script lang="ts">
// The overview: a greeting and what the history holds, the latest chat to
// pick up, a row of counts, then recent chats beside what is waiting in
// memory and where the history comes from. Every number is counted from what
// the dashboard has read; nothing here is estimated.
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import Clock from "@lucide/svelte/icons/clock";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { groupChats } from "../lib/browse";
import { onThisMachine } from "../lib/openIn";
import { stableRead } from "../lib/queries";
import {
  decisionOptions,
  decisionsByChat,
  type RecordPage,
  scopedRecordPage,
} from "../lib/records";
import { readingScope, scope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { statusOptions } from "../lib/status";
import { foldedNote } from "../lib/subagents";
import OverviewChats from "./OverviewChats.svelte";
import OverviewResume from "./OverviewResume.svelte";
import OverviewSide from "./OverviewSide.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";

// Everything here follows the viewing scope chosen in the top bar.
const catalogue = createScoped();
const status = createQuery(() => statusOptions);
// Memory still shows when the chat list cannot be read (see readingScope).
const reading = $derived(readingScope(catalogue.data, catalogue.isError, scope.chosen));
function memoryQuery(params: string) {
  return () => ({
    ...stableRead,
    queryKey: ["records", `overview:${params}`, reading ? scopeText(reading.scope) : ""],
    enabled: reading !== undefined,
    queryFn: ({ signal }: { signal: AbortSignal }) =>
      reading
        ? scopedRecordPage(params, reading.scope, reading.reach, signal)
        : Promise.reject(new Error("The viewing scope is not known yet.")),
  });
}
const memories = createQuery(memoryQuery(""));
// Memories agents saved that nobody has confirmed yet.
const waiting = createQuery(memoryQuery("review=candidate"));
// Both lists follow the index by themselves (see lib/live.ts).
const chats = $derived(catalogue.data?.chats ?? []);
const latest = $derived(chats[0]);
const recent = $derived(chats.slice(0, 7));
// Chats that produced decisions say how many; the memory page reads the same list.
const decisions = createQuery(() => decisionOptions);
const decided = $derived(decisionsByChat(decisions.data?.records ?? []));
const machines = $derived(groupChats(chats, (chat) => chat.machine));
const providers = $derived(groupChats(chats, (chat) => chat.agent));
const localNode = $derived(catalogue.data?.reach.localNode ?? "");

/** A count read from one page of a longer list is a lower bound, and says so. */
function counted(page: RecordPage): string {
  const more = page.nextOffset !== null || page.nextCursor !== null;
  return `${page.records.length.toLocaleString()}${more ? "+" : ""}`;
}
// Only memories whose current revision still awaits a review count, whatever
// the server's filter let through.
const unconfirmed = $derived(
  (waiting.data?.page.records ?? []).filter((record) => {
    const revision = record.revisions.find((item) => item.revision === record.currentRevision);
    return revision?.review === "candidate";
  }),
);
const waitingCount = $derived(
  waiting.data ? counted({ ...waiting.data.page, records: unconfirmed }) : "",
);
const plural = (count: number, word: string) =>
  `${count.toLocaleString()} ${word}${count === 1 ? "" : "s"}`;

// The greeting follows the reader's own clock.
const hour = new Date().getHours();
const greeting =
  hour < 5 || hour >= 18 ? "Good evening" : hour < 12 ? "Good morning" : "Good afternoon";
const summary = $derived.by(() => {
  if (!catalogue.data) return "";
  const held = `${plural(chats.length, "chat")} from ${plural(providers.length, "agent")} on ${plural(machines.length, "machine")}${foldedNote(catalogue.data.folded)}.`;
  if (!unconfirmed.length) return held;
  const one = waitingCount === "1";
  return `${held} ${waitingCount} ${one ? "memory is" : "memories are"} waiting for you.`;
});

// Each tile's line under the number is drawn only from what was read.
const weekAgo = Date.now() - 7 * 86_400_000;
const activeThisWeek = $derived(chats.filter((chat) => (chat.updatedMs ?? 0) >= weekAgo).length);
const mostUsed = $derived(
  [...providers].sort((a, b) => b.chats.length - a.chats.length)[0]?.chats[0]?.agent,
);
const tiles = $derived([
  {
    label: "Chats",
    value: chats.length.toLocaleString(),
    sub: activeThisWeek ? `${activeThisWeek.toLocaleString()} active this week` : "",
    href: "#/search",
  },
  {
    label: "Memories",
    value: memories.data ? counted(memories.data.page) : "",
    sub: unconfirmed.length ? `${waitingCount} waiting for you` : "",
    href: "#/records?",
  },
  { label: "Machines", value: machines.length.toLocaleString(), sub: "", href: "#/machines" },
  {
    label: "Agents",
    value: providers.length.toLocaleString(),
    sub: providers.length > 1 && mostUsed ? `${agentInfo[mostUsed].name} most used` : "",
    href: "#/agents",
  },
]);
</script>

<div class="ov-head">
  <h1 class="h1 ov-h1">{greeting}</h1>
  <p class="ov-sum">{summary}<ScopeNote scoped={catalogue.data} /></p>
</div>
{#if status.data && !status.data.configuration.loaded}<p class="card tight warn" style:display="flex" style:gap="10px" style:align-items="flex-start"><TriangleAlert size={17} aria-hidden="true" style="flex:none;margin-top:2px" /><span>Mash is not set up on this machine yet. <a class="a" href="#/setup">Set it up here</a>: choose which agent histories to index and connect your agents.</span></p>{/if}
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if latest && catalogue.data}
  <OverviewResume chat={latest} local={onThisMachine(latest, localNode, catalogue.data.reach.sharedOnly)} />
{/if}
{#if catalogue.data && chats.length}
  <div class="ov-tiles">
    {#each tiles as tile (tile.label)}
      <a class="ov-tile" href={tile.href}><span class="ov-tile-l">{tile.label}</span><b>{tile.value || "–"}</b>{#if tile.sub}<span class="ov-tile-s">{tile.sub}</span>{/if}</a>
    {/each}
  </div>
{/if}
<div class="ov-cols">
  <section class="ov-main" aria-labelledby="ov-recent">
    <div class="ov-h">
      <h2 id="ov-recent"><Clock size={15} aria-hidden="true" />Recent chats</h2>
      {#if chats.length}<a href="#/search">All chats<ChevronRight size={13} aria-hidden="true" /></a>{/if}
    </div>
    {#if catalogue.isPending}<div class="card pad0"><Skeleton rows={7} /></div>{/if}
    {#if recent.length}<OverviewChats chats={recent} {decided} />{/if}
    {#if catalogue.data && !chats.length}
      <div class="card empty">
        <img src={asset("mashy.png")} alt="" />
        <p>No indexed chats yet. <a class="a" href="https://getmash.io/synchronization/#refresh-now">Update your chat history</a>.</p>
      </div>
    {/if}
    {#each catalogue.data?.warnings ?? [] as warning (warning)}<p class="meta" role="status">{warning}</p>{/each}
  </section>
  <div class="ov-side">
    <OverviewSide {unconfirmed} recent={memories.data?.page.records} memoriesPending={memories.isPending && reading !== undefined} {machines} {providers} {localNode} />
    <QueryError title="Couldn’t load memory" query={memories} />
  </div>
</div>

<style>
  .ov-head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ov-h1 {
    font-size: 24px;
    font-weight: var(--weight-strong);
    letter-spacing: -0.03em;
  }
  .ov-sum {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
  }
  .ov-tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }
  .ov-tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    padding: 14px 16px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: var(--shadow-pick);
    color: var(--text-strong);
  }
  .ov-tile:hover {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
  .ov-tile-l {
    font-size: 12px;
    color: var(--text-faint);
  }
  .ov-tile b {
    font-size: 22px;
    font-weight: var(--weight-strong);
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }
  .ov-tile-s {
    font-size: 12px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ov-cols {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 24px;
  }
  .ov-main {
    flex: 2 1 440px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .ov-side {
    flex: 1 1 300px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .ov-h {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .ov-h h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 14px;
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .ov-h a {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .ov-h a:hover {
    color: var(--text-strong);
  }
  /* The section's icon shows only on a phone, as the phone wireframe draws it. */
  .ov-h h2 :global(svg) {
    display: none;
    color: var(--text-muted);
  }
  @media (max-width: 640px) {
    .ov-h h2 :global(svg) {
      display: block;
    }
  }
  @media (max-width: 800px) {
    .ov-tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .ov-h1 {
      font-size: 26px;
    }
    .ov-sum {
      font-size: 15px;
    }
  }
</style>
