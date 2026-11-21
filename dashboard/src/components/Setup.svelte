<script lang="ts">
import ArrowLeft from "@lucide/svelte/icons/arrow-left";
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import Check from "@lucide/svelte/icons/check";
import Copy from "@lucide/svelte/icons/copy";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { tick } from "svelte";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { catalogOptions } from "../lib/queries";
import { settingsOptions } from "../lib/settings";
import {
  type Answers,
  answersOf,
  days as dayLimits,
  type FirstRun,
  firstRunKey,
  firstRunOptions,
  type Integration,
  type Outcome,
  problem,
  runSetup,
  selectedHistories,
  shortPath,
  skipSetup,
  stepWord,
  summary,
} from "../lib/setup";
import { statusOptions } from "../lib/status";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import Skeleton from "./Skeleton.svelte";

// Open on the detected choices. The guided steps are available for editing.
// Nothing is written until the reader confirms, and the machine finds its
// own history locations.
const queryClient = useQueryClient();
const setup = createQuery(() => firstRunOptions);
const file = $derived(setup.data);
const searchServiceReady = $derived(file?.qdrant.state === "reachable");
const names = Object.fromEntries(
  Object.entries(agentInfo).map(([agent, info]) => [agent, info.name]),
) as Record<Agent, string>;

const steps = ["histories", "range", "agents", "review"] as const;
const titles = {
  histories: "Which histories should Mash index?",
  range: "How far back?",
  agents: "Connect Mash to your agents?",
  review: "Set up Mash",
} as const;
const helps = {
  histories:
    "Found on this machine in their usual locations. Mash reads them and never changes them.",
  range: "A history counts when it was changed within the range. You can widen it later.",
  agents:
    "A connected agent can look up your indexed chats. Excerpts it reads can be sent to its model provider.",
  review: "Review what Mash will read before you start. Your original chats stay untouched.",
} as const;
let at = $state<number | null>(null);
const step = $derived(at === null ? "review" : (steps[at] ?? "review"));
let heading: HTMLHeadingElement | undefined = $state();
async function goToStep(next: number | null): Promise<void> {
  at = next;
  await tick();
  heading?.focus();
}

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
const histories = $derived(file && answers ? selectedHistories(file, answers) : []);
const canKeepIndex = $derived.by(() => {
  if (!file?.configured || !answers || file.sinceDays !== answers.sinceDays) return false;
  const { agents } = answers;
  return file.histories.every((entry) => (agents[entry.agent] ?? entry.indexed) === entry.indexed);
});
const installedAgents = $derived(file?.integrations.filter((entry) => entry.installed) ?? []);

let running = $state(false);
let failure = $state("");
let outcome = $state<Outcome | null>(null);
let savedIndex = $state<"requested" | "deferred" | "kept">("deferred");
const indexRun = $derived(savedIndex === "requested" ? file?.index : undefined);
const indexed = $derived(indexRun?.state === "finished" && indexRun.ok);
const openChats = $derived(indexed || savedIndex === "kept");
async function run(index: boolean): Promise<void> {
  if (!file?.editable || !answers || running || blocked || (index && !searchServiceReady)) return;
  const indexChoice = index ? "requested" : canKeepIndex ? "kept" : "deferred";
  running = true;
  failure = "";
  try {
    const done = await runSetup(file, answers, index);
    savedIndex = indexChoice;
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
  <div class="cmdline" style:margin-top="6px"><code>{text}</code><button type="button" aria-label={copied === text ? "Copied" : "Copy the command"} title={copied === text ? "Copied" : "Copy"} onclick={() => copy(text)}>{#if copied === text}<Check size={16} aria-hidden="true" />{:else}<Copy size={16} aria-hidden="true" />{/if}</button></div>
{/snippet}
{#snippet searchServiceProblem(service: FirstRun["qdrant"])}
  <div>
    <p class="warn" style:display="flex" style:gap="7px" style:align-items="flex-start" style:font-size="13.5px"><TriangleAlert size={16} aria-hidden="true" style="flex:none;margin-top:2px" /><span>{service.state === "rejected" ? "Mash could not use the configured search service." : "Start the search service (Qdrant) before indexing. Run this in a terminal on this machine, then check again."}</span></p>
    {#if service.state === "rejected"}
      {#if service.message}<p class="meta" style:overflow-wrap="anywhere">{service.message}</p>{/if}
      <a class="a" href="https://getmash.io/troubleshooting/#qdrant-connection-failed">Check the search service</a>
    {:else}
      {@render command("mash qdrant install")}
    {/if}
    <div><button class="btn" type="button" style:margin-top="8px" disabled={setup.isFetching || running} onclick={() => setup.refetch()}>{setup.isFetching ? "Checking…" : "Check search service again"}</button></div>
    <p class="meta" style:margin-top="6px" style:overflow-wrap="anywhere">Search service address <code>{service.url}</code></p>
  </div>
{/snippet}
{#snippet toggle(label: string, on: boolean, flip: () => void, disabled = false)}
  <button class="sw" type="button" role="switch" aria-checked={on} aria-label={label} {disabled} onclick={flip}><i></i></button>
{/snippet}
{#snippet connectAgent(entry: FirstRun["integrations"][number], on: boolean, editable: boolean)}
  <div class="setup-row" class:off={!entry.installed}>
    <AgentMark agent={entry.agent} size={18} />
    <span class="setup-who"><strong>{agentInfo[entry.agent].name}</strong><small>{entry.installed ? (entry.connected ? "already connected" : "its command-line tool is installed") : "its command-line tool was not found here"}</small></span>
    {@render toggle(`Connect ${agentInfo[entry.agent].name}`, on, () => toggleConnect(entry.agent), !entry.installed || !editable || running)}
  </div>
{/snippet}

<main class="setup">
  <header class="setup-top">
    <span class="brand"><img src={asset("mash-mark.svg")} alt="" /><span>mash<i aria-hidden="true">_</i></span></span>
    {#if file && !running}
      <a class="a" href="#/" onclick={skipSetup}>{file.configured || outcome ? "Back to the dashboard" : "Skip to the dashboard"}</a>
    {/if}
  </header>

  <div class="setup-col">
    {#if at !== null && !outcome}
      <ol class="setup-steps" aria-label="Setup steps">
        {#each steps as name, index (name)}
          {@const done = index < at}
          <li class:done aria-current={index === at ? "step" : undefined}>
            <span class="n">{#if done}<Check size={15} aria-label="done" />{:else}{index + 1}{/if}</span>{name}
          </li>
        {/each}
      </ol>
    {/if}

    {#if setup.isPending}<Skeleton rows={4} />{/if}
    {#if setup.error}<p class="meta warn" role="alert">{file ? "Could not refresh setup" : "Setup can't be shown right now"}: {setup.error.message}</p>{/if}

    {#if file && answers && !outcome}
      <h1 class="setup-q" tabindex="-1" bind:this={heading}>{titles[step]}</h1>
      <p class="meta setup-help">{helps[step]}</p>
      {#if !file.editable}
        <p class="meta warn setup-help">This dashboard can be reached from other machines and was not started with a services configuration, so it does not run setup. Open the dashboard on the machine itself, or run <code class="i">mash init</code> there.</p>
      {:else if file.configured && step === "review"}
        <p class="meta setup-help">This machine is already set up. Saving replaces the configuration with these choices and keeps the previous file beside it.</p>
      {/if}

      <section class="setup-card" aria-label={titles[step]}>
        {#if step === "histories"}
          {#each file.histories as entry (entry.agent)}
            {@const on = answers.agents[entry.agent] ?? entry.indexed}
            <div class="setup-row" class:off={!on}>
              <AgentMark agent={entry.agent} size={18} />
              <span class="setup-who"><strong>{agentInfo[entry.agent].name}</strong><small title={entry.roots.join("\n")}>{entry.roots.map((root) => shortPath(root)).join(", ")}</small></span>
              {@render toggle(`Index ${agentInfo[entry.agent].name}`, on, () => { if (answers) answers.agents[entry.agent] = !on; }, !file.editable)}
            </div>
          {/each}
          {#if !file.histories.length}
            <p class="meta warn" style:padding="14px 0">No agent history was found in its usual location on this machine. Use an agent here first, or write the configuration file by hand.</p>
          {/if}
        {:else if step === "range"}
          <div class="setup-pick">
            <span class="seg" role="group" aria-label="History range">
              <button type="button" aria-pressed={answers.sinceDays === 90} onclick={() => { if (answers) answers.sinceDays = 90; }}>last 90 days</button>
              <button type="button" aria-pressed={answers.sinceDays === null} onclick={() => { if (answers) answers.sinceDays = null; }}>all history</button>
              <button type="button" aria-pressed={answers.sinceDays !== null && answers.sinceDays !== 90} onclick={() => { if (answers) answers.sinceDays = custom === 90 ? 30 : custom; }}>another number of days</button>
            </span>
            {#if answers.sinceDays !== null && answers.sinceDays !== 90}
              <span class="meta">the last <input class="field" type="number" min={dayLimits.least} max={dayLimits.most} aria-label="Days of history" style:width="92px" style:margin="0 4px" value={answers.sinceDays} oninput={(event) => { custom = event.currentTarget.valueAsNumber; if (answers) answers.sinceDays = Number.isNaN(custom) ? 0 : custom; }} /> days</span>
            {/if}
            <span class="meta">All history takes longer to index the first time.</span>
          </div>
        {:else if step === "agents"}
          {#each file.integrations as entry (entry.agent)}
            {@render connectAgent(entry, answers.connect.includes(entry.agent), file.editable)}
          {/each}
        {:else}
          <dl>
            {#each review as line (line.label)}
              <div class="kv" style:align-items="center" style:min-height="44px" style:border-bottom="1px solid var(--hair)" style:padding="0"><dt>{line.label}</dt><dd style:overflow-wrap="anywhere">{line.value}</dd></div>
            {/each}
          </dl>
          {#if histories.length}
            <details class="fold" style:margin-bottom="8px">
              <summary>Show history folders</summary>
              <ul class="meta" style:overflow-wrap="anywhere" style:padding-left="18px">
                {#each histories as entry (entry.agent)}
                  {#each entry.roots as root (root)}<li><code>{root}</code></li>{/each}
                {/each}
              </ul>
            </details>
          {/if}
        {/if}
      </section>

      {#if step === "agents"}
        <p class="meta">It is done by the agent's own command-line tool, which edits that agent's user settings and installs the Mash plugin. The last screen lists the files it touched and the commands that undo it. Other agents' histories are still indexed without this.</p>
      {:else if step === "review"}
        {#if file.editable && installedAgents.length && at === null}
          <details class="fold">
            <summary>Choose agents to connect</summary>
            <div>
              <p class="meta">{helps.agents}</p>
              {#each installedAgents as entry (entry.agent)}
                {@render connectAgent(entry, answers.connect.includes(entry.agent), file.editable)}
              {/each}
            </div>
          </details>
        {/if}
        {#if answers.connect.length && file.editable}
          <p class="meta">To connect the selected agents, Mash uses their own tools to update their user settings and install the Mash plugin.</p>
        {/if}
        {#if file.shared}
          <p class="meta warn">This machine is already set to send indexed chats to your shared server.</p>
        {/if}
        {#if searchServiceReady}
          <p class="ok" style:display="flex" style:gap="7px" style:align-items="center" style:font-size="13.5px"><Check size={16} aria-hidden="true" />The search service is ready.</p>
        {:else}
          {@render searchServiceProblem(file.qdrant)}
        {/if}
      {/if}

      {#if blocked && step !== "agents"}<p class="meta warn" role="alert">{blocked}</p>{/if}
      {#if failure}<p class="fail" role="alert">{failure}</p>{/if}
      <div class="setup-acts">
        {#if at !== null}
          {@const previous = at > 0 ? at - 1 : null}
          <button class="btn" type="button" disabled={running} onclick={() => goToStep(previous)}><ArrowLeft size={15} aria-hidden="true" />{previous === null ? "Back to review" : "Back"}</button>
        {:else if at === null && file.editable}
          <button class="btn" type="button" disabled={running || !file.histories.length} onclick={() => goToStep(0)}>Customize setup</button>
        {/if}
        <span class="sp"></span>
        {#if at !== null && step !== "review"}
          {@const next = at + 1}
          <button class="btn p" type="button" disabled={step !== "agents" && !!blocked} onclick={() => goToStep(next)}>{step === "agents" ? "Review setup" : "Next"}<ArrowRight size={15} aria-hidden="true" /></button>
        {:else if file.editable}
          {#if !searchServiceReady || at !== null || file.configured}
            <button class="btn" class:p={!searchServiceReady || canKeepIndex} type="button" disabled={running || !!blocked || setup.isFetching} onclick={() => run(false)}>Set up without indexing</button>
          {/if}
          {#if searchServiceReady}
            <button class="btn" class:p={!canKeepIndex} type="button" disabled={running || !!blocked || setup.isFetching} onclick={() => run(true)}>Set up and index</button>
          {/if}
        {/if}
      </div>
      {#if running}<p class="meta" role="status">Writing the configuration and connecting agents. Connecting an agent can take a minute.</p>{/if}
    {/if}

    {#if outcome && file}
      <h1 class="setup-q">Setup is saved</h1>
      <p class="meta setup-help" style:overflow-wrap="anywhere">Configuration {outcome.written ? "written to" : "kept at"} {outcome.path}{#if outcome.previous}; the previous file is at {outcome.previous}{/if}</p>

      {#if savedIndex !== "kept"}
        <section class="setup-card" aria-label="First index">
          <div class="setup-pick">
            <strong>First index</strong>
            {#if indexRun?.state === "running"}
              <p class="meta" role="status">Indexing your history in the background. This screen follows it; you can also open the dashboard now, and chats appear as they are saved.</p>
            {:else if indexRun?.state === "finished"}
              {#if indexRun.ok}
                <p class="ok" role="status" style:display="flex" style:gap="7px" style:align-items="center"><Check size={16} aria-hidden="true" />Indexed {(indexRun.chats ?? 0).toLocaleString()} chats and {(indexRun.messages ?? 0).toLocaleString()} messages.</p>
                {#if indexRun.message}<p class="meta warn" style:overflow-wrap="anywhere">{indexRun.message}</p>{/if}
              {:else}
                <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing stopped: {indexRun.message ?? "it did not say why."}</p>
              {/if}
            {:else if indexRun?.state === "skipped"}
              <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing was not started: {indexRun.reason}</p>
            {:else}
              <p class="meta">Indexing was skipped.</p>
            {/if}
            {#if !indexed && indexRun?.state !== "running"}
              {#if searchServiceReady}
                <p class="meta">Run this in a terminal on this machine to index your history.</p>
                {@render command("mash index")}
              {:else}
                {@render searchServiceProblem(file.qdrant)}
              {/if}
            {/if}
          </div>
        </section>
      {/if}

      {#if indexed}
        <details class="fold">
          <summary>Search by meaning</summary>
          {#if !file.modelsInstalled}
            <p class="meta">Start by installing the optional local models in your terminal.</p>
            {@render command("mash models install")}
          {/if}
          <p class="meta"><a class="a" href="https://getmash.io/indexing/#enable-semantic-search" target="_blank" rel="noreferrer">Configure and index for search by meaning</a>. You can choose which search the dashboard uses.</p>
        </details>
      {/if}

      {#if outcome.agents.length}
        {#if outcome.agents.some((agent) => stepWord(agent.mcp).ok)}
          <p class="meta">Start a new session in the connected agent, then <a class="a" href="https://getmash.io/agents/#try-it" target="_blank" rel="noreferrer">ask it to find a past chat</a>.</p>
        {/if}
        <section class="setup-card" aria-label="Connected agents">
          {#each outcome.agents as agent (agent.agent)}
            {@const mcp = stepWord(agent.mcp)}
            {@const plugin = stepWord(agent.plugin)}
            <div class="setup-row">
              <AgentMark agent={agent.agent} size={18} />
              <span class="setup-who"><strong>{agentInfo[agent.agent].name}</strong><small><span class={mcp.ok ? "ok" : "warn"}>search tools: {mcp.word}</span> · <span class={plugin.ok ? "ok" : "warn"}>plugin: {plugin.word}</span></small></span>
            </div>
            {#each [agent.mcp, agent.plugin] as part, index (index)}
              {#if part.state === "failed" || part.state === "stale"}
                <p class="meta warn" style:margin-top="6px" style:overflow-wrap="anywhere">{part.message} To do it yourself, run:</p>
                {@render command(part.command)}
              {/if}
            {/each}
            <details class="fold" style:margin="6px 0 10px">
              <summary>What changed for {agentInfo[agent.agent].name}, and how to undo it</summary>
              <div>
                <p class="meta" style:overflow-wrap="anywhere">Its own tool edited: {agent.files.join(", ")}</p>
                {#each agent.undo as undo (undo)}{@render command(undo)}{/each}
              </div>
            </details>
          {/each}
        </section>
      {/if}

      {#if outcome.next.length}
        <details class="fold">
          <summary>Next steps ({outcome.next.length})</summary>
          <ul class="meta" style:padding-left="18px" style:display="grid" style:gap="6px">
            {#each outcome.next as line (line)}<li style:overflow-wrap="anywhere">{line}</li>{/each}
          </ul>
        </details>
      {/if}

      <div class="setup-acts">
        <span class="sp"></span>
        <a class="btn p" href={openChats ? "#/search" : "#/"} onclick={skipSetup}>{openChats ? "Open your chats" : "Open the dashboard"}</a>
      </div>
      <p class="meta setup-help">Sharing with other machines and backfilling memory are set up from the terminal with <code class="i">mash init</code>.</p>
    {/if}
  </div>
</main>
