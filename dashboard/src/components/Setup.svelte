<script lang="ts">
import ArrowLeft from "@lucide/svelte/icons/arrow-left";
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import Check from "@lucide/svelte/icons/check";
import Copy from "@lucide/svelte/icons/copy";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
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
  skipSetup,
  stepWord,
  summary,
} from "../lib/setup";
import { statusOptions } from "../lib/status";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import Skeleton from "./Skeleton.svelte";

// The questions `mash init` asks in a terminal, one to a screen, on a page of
// its own: no rail and no search, because before setup they lead nowhere.
// Nothing is written until the last step, and history locations are never
// typed here: the machine finds them itself.
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
const helps = {
  histories:
    "Found on this machine in their usual locations. Mash reads them and never changes them.",
  range: "A history counts when it was changed within the range. You can widen it later.",
  agents: "Connecting lets an agent search your history through Mash.",
  review: "Nothing has been changed yet.",
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
  <div class="cmdline" style:margin-top="6px"><code>{text}</code><button type="button" aria-label={copied === text ? "Copied" : "Copy the command"} title={copied === text ? "Copied" : "Copy"} onclick={() => copy(text)}>{#if copied === text}<Check size={16} aria-hidden="true" />{:else}<Copy size={16} aria-hidden="true" />{/if}</button></div>
{/snippet}
{#snippet toggle(label: string, on: boolean, flip: () => void, disabled = false)}
  <button class="sw" type="button" role="switch" aria-checked={on} aria-label={label} {disabled} onclick={flip}><i></i></button>
{/snippet}

<main class="setup">
  <header class="setup-top">
    <span class="brand"><img src={asset("mash-mark.svg")} alt="" /><span>mash<i aria-hidden="true">_</i></span></span>
    {#if file && !running}
      <a class="a" href="#/" onclick={skipSetup}>{file.configured || outcome ? "Back to the dashboard" : "Skip to the dashboard"}</a>
    {/if}
  </header>

  <div class="setup-col">
    <ol class="setup-steps" aria-label="Setup steps">
      {#each steps as name, index (name)}
        {@const done = !!outcome || index < at}
        <li class:done aria-current={!outcome && index === at ? "step" : undefined}>
          <span class="n">{#if done}<Check size={15} aria-label="done" />{:else}{index + 1}{/if}</span>{name}
        </li>
      {/each}
    </ol>

    {#if setup.isPending}<Skeleton rows={4} />{/if}
    {#if setup.error}<p class="meta warn" role="alert">Setup can't be shown right now: {setup.error.message}</p>{/if}

    {#if file && answers && !outcome}
      <h1 class="setup-q">{titles[step]}</h1>
      <p class="meta setup-help">{helps[step]}</p>
      {#if !file.editable}
        <p class="meta warn setup-help">This dashboard can be reached from other machines and was not started with a services configuration, so it does not run setup. Open the dashboard on the machine itself, or run <code class="i">mash init</code> there.</p>
      {:else if file.configured && step === "histories"}
        <p class="meta setup-help">This machine is already set up. Finishing these steps rewrites the configuration file with your answers and keeps the previous one beside it.</p>
      {/if}

      <section class="setup-card" aria-label={titles[step]}>
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
            {@const on = answers.connect.includes(entry.agent)}
            <div class="set-a" class:off={!entry.installed}>
              <AgentMark agent={entry.agent} size={17} />
              <span class="set-n">{agentInfo[entry.agent].name}</span>
              <span class="meta">{entry.installed ? (entry.connected ? "already connected; choosing it checks the connection" : "its command-line tool is installed") : "its command-line tool was not found on this machine"}</span>
              {@render toggle(`Connect ${agentInfo[entry.agent].name}`, on, () => toggleConnect(entry.agent), !entry.installed || !file.editable)}
            </div>
          {/each}
        {:else}
          <dl>
            {#each review as line (line.label)}
              <div class="kv" style:align-items="center" style:min-height="44px" style:border-bottom="1px solid var(--hair)" style:padding="0"><dt>{line.label}</dt><dd style:overflow-wrap="anywhere">{line.value}</dd></div>
            {/each}
          </dl>
        {/if}
      </section>

      {#if step === "agents"}
        <p class="meta">It is done by the agent's own command-line tool, which edits that agent's user settings and installs the Mash plugin. The last screen lists the files it touched and the commands that undo it. Other agents' histories are still indexed without this.</p>
      {:else if step === "review"}
        {#if file.qdrant.reachable}
          <p class="ok" style:display="flex" style:gap="7px" style:align-items="center" style:font-size="13.5px"><Check size={16} aria-hidden="true" />The search service (Qdrant) answers at {file.qdrant.url}, so indexing can start.</p>
        {:else}
          <div>
            <p class="warn" style:display="flex" style:gap="7px" style:align-items="flex-start" style:font-size="13.5px"><TriangleAlert size={16} aria-hidden="true" style="flex:none;margin-top:2px" /><span>The search service (Qdrant) does not answer at {file.qdrant.url}. Indexing and every search need it. You can set up now and index afterwards. To install and start it, run this in a terminal on this machine:</span></p>
            {@render command("mash qdrant install")}
          </div>
        {/if}
        {#if !file.modelsInstalled}
          <div>
            <p class="meta">Search will find the words you type. For search by meaning as well, run this in a terminal; it downloads the models (about 177 MB):</p>
            {@render command("mash models install")}
          </div>
        {/if}
      {/if}

      {#if blocked && step !== "agents"}<p class="meta warn" role="alert">{blocked}</p>{/if}
      {#if failure}<p class="fail" role="alert">{failure}</p>{/if}
      <div class="setup-acts">
        {#if at > 0}<button class="btn" type="button" disabled={running} onclick={() => (at -= 1)}><ArrowLeft size={15} aria-hidden="true" />Back</button>{/if}
        <span class="sp"></span>
        {#if step !== "review"}
          <button class="btn p" type="button" disabled={step !== "agents" && !!blocked} onclick={() => (at += 1)}>Next<ArrowRight size={15} aria-hidden="true" /></button>
        {:else if file.editable}
          <button class="btn" type="button" disabled={running || !!blocked} onclick={() => run(false)}>Set up without indexing</button>
          <button class="btn p" type="button" disabled={running || !!blocked || !file.qdrant.reachable} title={file.qdrant.reachable ? "" : "Start the search service first"} onclick={() => run(true)}>{running ? "Setting up…" : "Set up and index"}</button>
        {/if}
      </div>
      {#if running}<p class="meta" role="status">Writing the configuration and connecting agents. Connecting an agent can take a minute.</p>{/if}
    {/if}

    {#if outcome && file}
      <h1 class="setup-q">Setup is saved</h1>
      <p class="meta setup-help" style:overflow-wrap="anywhere">Configuration {outcome.written ? "written to" : "kept at"} {outcome.path}{#if outcome.previous}; the previous file is at {outcome.previous}{/if}</p>

      <section class="setup-card" aria-label="First index">
        <div class="setup-pick">
          <strong>First index</strong>
          {#if file.index.state === "running"}
            <p class="meta" role="status">Indexing your history in the background. This screen follows it; you can also open the dashboard now, and chats appear as they are saved.</p>
          {:else if file.index.state === "finished"}
            {#if file.index.ok}
              <p class="ok" role="status" style:display="flex" style:gap="7px" style:align-items="center"><Check size={16} aria-hidden="true" />Indexed {(file.index.chats ?? 0).toLocaleString()} chats and {(file.index.messages ?? 0).toLocaleString()} messages.</p>
              {#if file.index.message}<p class="meta warn" style:overflow-wrap="anywhere">{file.index.message}</p>{/if}
            {:else}
              <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing stopped: {file.index.message ?? "it did not say why."}</p>
              <p class="meta">Setup itself is saved. To try again, run this in a terminal on this machine:</p>
              {@render command("mash index")}
            {/if}
          {:else if file.index.state === "skipped"}
            <p class="warn" role="alert" style:overflow-wrap="anywhere">Indexing was not started: {file.index.reason}</p>
            {@render command("mash index")}
          {:else}
            <p class="meta">Not started. When the search service is running, index your history from a terminal on this machine:</p>
            {@render command("mash index")}
          {/if}
        </div>
      </section>

      {#if outcome.agents.length}
        <section class="setup-card" aria-label="Connected agents">
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
        <a class="btn p" href={file.index.state === "finished" && file.index.ok ? "#/search" : "#/"} onclick={skipSetup}>{file.index.state === "finished" && file.index.ok ? "Open your chats" : "Open the dashboard"}</a>
      </div>
      <p class="meta setup-help">Sharing with other machines and backfilling memory are set up from the terminal with <code class="i">mash init</code>.</p>
    {/if}
  </div>
</main>
