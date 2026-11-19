<script lang="ts">
import ArrowLeft from "@lucide/svelte/icons/arrow-left";
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import Bot from "@lucide/svelte/icons/bot";
import Check from "@lucide/svelte/icons/check";
import Clock from "@lucide/svelte/icons/clock";
import Copy from "@lucide/svelte/icons/copy";
import ListChecks from "@lucide/svelte/icons/list-checks";
import Plug from "@lucide/svelte/icons/plug";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { catalogOptions } from "../lib/queries";
import { settingsOptions } from "../lib/settings";
import {
  type Answers,
  answersOf,
  days as dayLimits,
  firstRunKey,
  firstRunOptions,
  type Integration,
  type Outcome,
  problem,
  runSetup,
  stepWord,
  summary,
} from "../lib/setup";
import { statusOptions } from "../lib/status";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import Skeleton from "./Skeleton.svelte";

// The questions `mash init` asks in a terminal, one at a time. Nothing is
// written until the last step, and history locations are never typed here:
// the machine finds them itself.
const queryClient = useQueryClient();
const setup = createQuery(() => firstRunOptions);
const file = $derived(setup.data);
const names = Object.fromEntries(
  Object.entries(agentInfo).map(([agent, info]) => [agent, info.name]),
) as Record<Agent, string>;

const steps = ["histories", "range", "agents", "review"] as const;
const titles = {
  histories: "Which histories should Mash index?",
  range: "How far back?",
  agents: "Connect Mash to your agents?",
  review: "Review and set up",
} as const;
let at = $state(0);
const step = $derived(steps[at] ?? "review");

let answers = $state<Answers | null>(null);
let based = $state<string | null>(null);
// Start from the file's choices, and again if the file changes underneath.
$effect(() => {
  if (file && file.revision !== based && !outcome) {
    answers = answersOf(file);
    based = file.revision;
  }
});
let custom = $state(90);
const blocked = $derived(file && answers ? problem(file, answers) : "");
const review = $derived(file && answers ? summary(file, answers, names) : []);

let running = $state(false);
let failure = $state("");
let outcome = $state<Outcome | null>(null);
async function run(index: boolean): Promise<void> {
  if (!file || !answers || running || blocked) return;
  running = true;
  failure = "";
  try {
    const done = await runSetup(file, answers, index);
    outcome = done.outcome;
    queryClient.setQueryData(firstRunKey, done.setup);
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: statusOptions.queryKey }),
      queryClient.invalidateQueries({ queryKey: settingsOptions.queryKey }),
    ]);
  } catch (error) {
    failure = error instanceof Error ? error.message : "Setup did not finish.";
    // Part of it may have been done; show the machine as it now is.
    await queryClient.invalidateQueries({ queryKey: firstRunKey });
  } finally {
    running = false;
  }
}
// When the first index ends, the chat lists and status are worth rereading.
let watched = $state("");
$effect(() => {
  const state = file?.index.state ?? "";
  if (watched === "running" && state === "finished") {
    void queryClient.invalidateQueries({ queryKey: catalogOptions.queryKey });
    void queryClient.invalidateQueries({ queryKey: statusOptions.queryKey });
  }
  watched = state;
});

let copied = $state("");
async function copy(command: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(command);
    copied = command;
  } catch {
    copied = "";
  }
}
function toggleConnect(agent: Integration): void {
  if (!answers) return;
  answers.connect = answers.connect.includes(agent)
    ? answers.connect.filter((entry) => entry !== agent)
    : [...answers.connect, agent];
}
</script>

{#snippet command(text: string)}
  <div class="cmdline" style:max-width="520px" style:margin-top="6px"><code>{text}</code><button type="button" aria-label={copied === text ? "Copied" : "Copy the command"} title={copied === text ? "Copied" : "Copy"} onclick={() => copy(text)}>{#if copied === text}<Check size={16} aria-hidden="true" />{:else}<Copy size={16} aria-hidden="true" />{/if}</button></div>
{/snippet}
{#snippet toggle(label: string, on: boolean, flip: () => void, disabled = false)}
  <button class="sw" type="button" role="switch" aria-checked={on} aria-label={label} {disabled} onclick={flip}><i></i></button>
{/snippet}

<div class="hd">
  <div><h1 class="h1">Set up Mash</h1><p class="meta" style:font-size="13.5px" style:overflow-wrap="anywhere">{file?.path ?? "This machine's configuration"}</p></div>
  {#if file && !outcome}<span class="meta" style:font-size="13.5px">Step {at + 1} of {steps.length}</span>{/if}
</div>
{#if setup.isPending}<Skeleton rows={4} />{/if}
{#if setup.error}<p class="meta warn" role="alert">Setup can't be shown right now: {setup.error.message}</p>{/if}

{#if file && answers && !outcome}
  {#if !file.editable}
    <p class="meta warn">This dashboard can be reached from other machines and was not started with a services configuration, so it does not run setup. Open the dashboard on the machine itself, or run <code class="i">mash init</code> there.</p>
  {:else if file.configured}
    <p class="meta">This machine is already set up. Going through setup again rewrites the configuration file with your answers and keeps the previous one beside it; settings these questions don't cover are kept.</p>
  {:else}
    <p class="meta">Four questions, then Mash writes its configuration, connects the agents you choose and indexes your history. Nothing is changed until the last step.</p>
  {/if}

  <section class="set-s" aria-labelledby="setup-question">
    <h2 class="h2" id="setup-question">
      {#if step === "histories"}<Bot size={15} aria-hidden="true" />{:else if step === "range"}<Clock size={15} aria-hidden="true" />{:else if step === "agents"}<Plug size={15} aria-hidden="true" />{:else}<ListChecks size={15} aria-hidden="true" />{/if}
      {titles[step]}
    </h2>

    {#if step === "histories"}
      {#each file.histories as entry (entry.agent)}
        {@const on = answers.agents[entry.agent] ?? entry.indexed}
        <div class="set-a" class:off={!on}>
          <AgentMark agent={entry.agent} size={17} />
          <span class="set-n">{agentInfo[entry.agent].name}</span>
          <span class="meta set-p">{entry.roots.join(", ")}</span>
          {@render toggle(`Index ${agentInfo[entry.agent].name}`, on, () => { if (answers) answers.agents[entry.agent] = !on; }, !file.editable)}
        </div>
      {/each}
      {#if file.histories.length}
        <p class="meta" style:margin-top="8px">These are the histories found on this machine in their usual locations. Mash reads them and never changes them. To index a history kept somewhere else, add its location to the configuration file afterwards.</p>
      {:else}
        <p class="meta warn" style:margin-top="8px">No agent history was found in its usual location on this machine. Use an agent here first, or write the configuration file by hand.</p>
      {/if}
    {:else if step === "range"}
      <div class="set-r">
        <span class="set-l">history range<small>older chats can be added later by widening this</small></span>
        <span class="set-c">
          <span class="seg" role="group" aria-label="History range">
            <button type="button" aria-pressed={answers.sinceDays === 90} onclick={() => { if (answers) answers.sinceDays = 90; }}>last 90 days</button>
            <button type="button" aria-pressed={answers.sinceDays === null} onclick={() => { if (answers) answers.sinceDays = null; }}>all history</button>
            <button type="button" aria-pressed={answers.sinceDays !== null && answers.sinceDays !== 90} onclick={() => { if (answers) answers.sinceDays = custom === 90 ? 30 : custom; }}>another number of days</button>
          </span>
          {#if answers.sinceDays !== null && answers.sinceDays !== 90}
            <span class="meta">the last <input class="field" type="number" min={dayLimits.least} max={dayLimits.most} aria-label="Days of history" style:width="92px" style:margin="0 4px" value={answers.sinceDays} oninput={(event) => { custom = event.currentTarget.valueAsNumber; if (answers) answers.sinceDays = Number.isNaN(custom) ? 0 : custom; }} /> days</span>
          {/if}
        </span>
      </div>
      <p class="meta" style:margin-top="8px">A history counts when it was changed within the range. All history takes longer to index the first time.</p>
    {:else if step === "agents"}
      {#each file.integrations as entry (entry.agent)}
        {@const on = answers.connect.includes(entry.agent)}
        <div class="set-a" class:off={!entry.installed}>
          <AgentMark agent={entry.agent} size={17} />
          <span class="set-n">{agentInfo[entry.agent].name}</span>
          <span class="meta">{entry.installed ? (entry.connected ? "already connected; choosing it checks the connection" : "its command-line tool is installed") : "its command-line tool was not found on this machine"}</span>
          {@render toggle(`Connect ${agentInfo[entry.agent].name}`, on, () => toggleConnect(entry.agent), !entry.installed || !file.editable)}
        </div>
      {/each}
      <p class="meta" style:margin-top="8px">Connecting lets an agent search your history through Mash. It is done by the agent's own command-line tool, which edits that agent's user settings and installs the Mash plugin. Setup lists the files it touched and the commands that undo it. Other agents' histories are still indexed without this.</p>
    {:else}
      <dl>
        {#each review as line (line.label)}
          <div class="kv" style:align-items="center" style:min-height="44px" style:border-bottom="1px solid var(--hair)" style:padding="0"><dt>{line.label}</dt><dd style:overflow-wrap="anywhere">{line.value}</dd></div>
        {/each}
      </dl>
      <div style:margin-top="12px">
        {#if file.qdrant.reachable}
          <p class="ok" style:display="flex" style:gap="7px" style:align-items="center" style:font-size="13.5px"><Check size={16} aria-hidden="true" />The search service (Qdrant) answers at {file.qdrant.url}, so indexing can start.</p>
        {:else}
          <p class="warn" style:display="flex" style:gap="7px" style:align-items="flex-start" style:font-size="13.5px"><TriangleAlert size={16} aria-hidden="true" style="flex:none;margin-top:2px" /><span>The search service (Qdrant) does not answer at {file.qdrant.url}. Indexing and every search need it. You can set up now and index afterwards. To install and start it, run this in a terminal on this machine:</span></p>
          {@render command("mash qdrant install")}
        {/if}
        {#if !file.modelsInstalled}
          <p class="meta" style:margin-top="10px">Search will find the words you type. For search by meaning as well, run this in a terminal; it downloads the models (about 177 MB):</p>
          {@render command("mash models install")}
        {/if}
      </div>
    {/if}
  </section>

  {#if blocked && (step === "review" || step === "histories" || step === "range")}<p class="meta warn" role="alert" style:margin-top="8px">{blocked}</p>{/if}
  {#if failure}<p class="fail" role="alert" style:margin-top="8px">{failure}</p>{/if}
  <div class="set-bt" style:max-width="860px" style:margin-top="14px">
    <span style:flex="1 1 auto">
      {#if at > 0}<button class="btn" type="button" disabled={running} onclick={() => (at -= 1)}><ArrowLeft size={15} aria-hidden="true" />Back</button>{/if}
    </span>
    {#if step !== "review"}
      <button class="btn p" type="button" disabled={(step === "histories" || step === "range") && !!blocked} onclick={() => (at += 1)}>Next<ArrowRight size={15} aria-hidden="true" /></button>
    {:else if file.editable}
      <button class="btn" type="button" disabled={running || !!blocked} onclick={() => run(false)}>Set up without indexing</button>
      <button class="btn p" type="button" disabled={running || !!blocked || !file.qdrant.reachable} title={file.qdrant.reachable ? "" : "Start the search service first"} onclick={() => run(true)}>{running ? "Setting up…" : "Set up and index"}</button>
    {/if}
  </div>
  {#if running}<p class="meta" role="status" style:margin-top="8px">Writing the configuration and connecting agents. Connecting an agent can take a minute.</p>{/if}
{/if}

{#if outcome && file}
  <section class="set-s">
    <h2 class="h2"><Check size={15} aria-hidden="true" />setup is saved</h2>
    <dl>
      <div class="kv" style:align-items="center" style:min-height="44px" style:border-bottom="1px solid var(--hair)" style:padding="0"><dt>configuration</dt><dd style:overflow-wrap="anywhere">{outcome.written ? "written to" : "kept at"} {outcome.path}{#if outcome.previous}; the previous file is at {outcome.previous}{/if}</dd></div>
    </dl>
    {#each outcome.agents as agent (agent.agent)}
      {@const mcp = stepWord(agent.mcp)}
      {@const plugin = stepWord(agent.plugin)}
      <div class="set-a">
        <AgentMark agent={agent.agent} size={17} />
        <span class="set-n">{agentInfo[agent.agent].name}</span>
        <span class="meta"><span class={mcp.ok ? "ok" : "warn"}>search tools: {mcp.word}</span> · <span class={plugin.ok ? "ok" : "warn"}>plugin: {plugin.word}</span></span>
        <span></span>
      </div>
      {#each [agent.mcp, agent.plugin] as part, index (index)}
        {#if part.state === "failed" || part.state === "stale"}
          <p class="meta warn" style:margin-top="6px" style:overflow-wrap="anywhere">{part.message} To do it yourself, run:</p>
          {@render command(part.command)}
        {/if}
      {/each}
      <details class="fold" style:margin-top="6px">
        <summary>What changed for {agentInfo[agent.agent].name}, and how to undo it</summary>
        <div>
          <p class="meta" style:overflow-wrap="anywhere">Its own tool edited: {agent.files.join(", ")}</p>
          {#each agent.undo as undo (undo)}{@render command(undo)}{/each}
        </div>
      </details>
    {/each}
  </section>

  <section class="set-s">
    <h2 class="h2"><ListChecks size={15} aria-hidden="true" />first index</h2>
    {#if file.index.state === "running"}
      <p class="meta" role="status">Indexing your history in the background. This page follows it; you can leave and come back. Chats appear in the dashboard as they are saved.</p>
    {:else if file.index.state === "finished"}
      {#if file.index.ok}
        <p class="ok" role="status" style:display="flex" style:gap="7px" style:align-items="center"><Check size={16} aria-hidden="true" />Indexed {(file.index.chats ?? 0).toLocaleString()} chats and {(file.index.messages ?? 0).toLocaleString()} messages.</p>
        {#if file.index.message}<p class="meta warn" style:margin-top="6px" style:overflow-wrap="anywhere">{file.index.message}</p>{/if}
        <p style:margin-top="10px"><a class="btn p" href="#/search">Open your chats</a></p>
      {:else}
        <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing stopped: {file.index.message ?? "it did not say why."}</p>
        <p class="meta" style:margin-top="6px">Setup itself is saved. To try again, run this in a terminal on this machine:</p>
        {@render command("mash index")}
      {/if}
    {:else if file.index.state === "skipped"}
      <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing was not started: {file.index.reason}</p>
      {@render command("mash index")}
    {:else}
      <p class="meta">Not started. When the search service is running, index your history from a terminal on this machine:</p>
      {@render command("mash index")}
    {/if}
  </section>

  {#if outcome.next.length}
    <section class="set-s">
      <h2 class="h2">next steps</h2>
      <ul class="meta" style:padding-left="18px" style:display="grid" style:gap="6px">
        {#each outcome.next as line (line)}<li style:overflow-wrap="anywhere">{line}</li>{/each}
      </ul>
    </section>
  {/if}
  <p class="meta" style:margin-top="10px">Sharing with other machines and backfilling memory are set up from the terminal with <code class="i">mash init</code>. Everything else is on the <a class="a" href="#/settings">settings</a> page.</p>
{/if}
