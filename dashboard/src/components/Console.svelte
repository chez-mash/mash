<script lang="ts">
import Bookmark from "@lucide/svelte/icons/bookmark";
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import Copy from "@lucide/svelte/icons/copy";
import Eraser from "@lucide/svelte/icons/eraser";
import History from "@lucide/svelte/icons/history";
import Lock from "@lucide/svelte/icons/lock";
import Play from "@lucide/svelte/icons/play";
import SearchIcon from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import Wrench from "@lucide/svelte/icons/wrench";
import { createQuery } from "@tanstack/svelte-query";
import { read } from "../lib/api";
import {
  type Answer,
  argumentsFor,
  foldAfter,
  groupOrder,
  headline,
  locked,
  type Param,
  parseAnswer,
  parseConsole,
  sentences,
  spans,
  summarize,
  type Tool,
} from "../lib/console";
import { stableRead } from "../lib/queries";
import JsonView from "./JsonView.svelte";
import QueryError from "./QueryError.svelte";
import Skeleton from "./Skeleton.svelte";

// The tools an agent can call on Mash, listed from the server's own
// description of them, with a form to run the ones that only read. The server
// refuses the rest, whatever this page sends.
const listing = createQuery(() => ({
  ...stableRead,
  queryKey: ["console-tools"],
  queryFn: async ({ signal }) =>
    parseConsole(await read("/api/console/tools", new URLSearchParams(), signal)),
}));
let filter = $state("");
// Every tool starts folded, so the page first reads as a list of what exists.
let open = $state("");
// Whether a tool's optional parameters are unfolded, by tool name.
let more = $state<Record<string, boolean>>({});
let values = $state<Record<string, string>>({});
type Outcome =
  | { readonly state: "idle" }
  | { readonly state: "running" }
  | { readonly state: "done"; readonly answer: Answer; readonly ms: number; readonly bytes: number }
  | { readonly state: "unreachable"; readonly message: string };
let outcome = $state<Outcome>({ state: "idle" });
let controller: AbortController | undefined;
let notice = $state("");

// Section headings carry an icon and lower-case words, like every other
// section heading on the dashboard.
const sectionIcons: Record<string, typeof Wrench> = {
  History,
  Memory: Bookmark,
  Service: Server,
  [locked]: Lock,
};
const tools = $derived(listing.data?.tools ?? []);
const matching = $derived.by(() => {
  const words = filter.trim().toLowerCase();
  return words
    ? tools.filter((tool) => `${tool.name} ${tool.about}`.toLowerCase().includes(words))
    : tools;
});
const sections = $derived(
  groupOrder
    .map((name) => ({ name, tools: matching.filter((tool) => tool.group === name) }))
    .filter((section) => section.tools.length),
);
const runnable = $derived(tools.filter((tool) => tool.readOnly).length);
const tool = $derived(tools.find((entry) => entry.name === open));
const request = $derived(tool ? argumentsFor(tool, values) : null);
const summary = $derived(outcome.state === "done" ? summarize(outcome.answer) : null);

function toggle(name: string): void {
  controller?.abort();
  open = open === name ? "" : name;
  outcome = { state: "idle" };
  notice = "";
}
async function run(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  if (!tool || !request?.ok) return;
  controller?.abort();
  const current = new AbortController();
  controller = current;
  outcome = { state: "running" };
  notice = "";
  const started = performance.now();
  try {
    const body = await read(
      "/api/console/call",
      new URLSearchParams({ name: tool.name, arguments: JSON.stringify(request.arguments) }),
      current.signal,
    );
    const answer = parseAnswer(body);
    outcome = {
      state: "done",
      answer,
      ms: Math.round(performance.now() - started),
      bytes: new Blob([JSON.stringify(answer.reply)]).size,
    };
  } catch (error) {
    if (current.signal.aborted) return;
    outcome = {
      state: "unreachable",
      message: error instanceof Error ? error.message : "The request failed.",
    };
  }
}
async function copy(text: string, what: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    notice = `${what} copied.`;
  } catch {
    notice = "Copy failed. Select the text to copy it.";
  }
}
/** The parameters shown at once, and the optional ones folded away when
 * there are too many to scan: a required parameter is never folded. */
function fields(tool: Tool): { always: readonly Param[]; extra: readonly Param[] } {
  const optional = tool.params.filter((param) => !param.required);
  return optional.length > foldAfter
    ? { always: tool.params.filter((param) => param.required), extra: optional }
    : { always: tool.params, extra: [] };
}
function size(bytes: number): string {
  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(bytes < 10_240 ? 1 : 0)} KB`;
}
$effect(() => () => controller?.abort());
</script>

{#snippet prose(text: string)}{#each spans(text) as span, index (index)}{#if span.code}<code>{span.text}</code>{:else}{span.text}{/if}{/each}{/snippet}
{#snippet field(entry: Tool, param: Param)}
{@const key = `${entry.name}.${param.name}`}
<label class="prm" class:bad={request && !request.ok && request.param === param.name && outcome.state !== "idle"}>
  <span class="prm-n"><code>{param.name}</code><small>{param.type}</small>{#if param.required}<b class="pill">required</b>{/if}</span>
  <span class="prm-a">{@render prose(param.about)}{#if param.fallback}{" "}<small>Default {param.fallback}.</small>{/if}</span>
  {#if !entry.readOnly}<span hidden></span>
  {:else if param.kind === "enum" || param.kind === "boolean"}
    <span class="prm-sel">
      <select class="prm-i" class:unset={!values[key]} bind:value={values[key]}>
        <option value="">{param.required ? "choose…" : "not set"}</option>
        {#each param.kind === "boolean" ? ["true", "false"] : param.choices as choice (choice)}<option value={choice}>{choice}</option>{/each}
      </select>
      <ChevronDown size={15} aria-hidden="true" />
    </span>
  {:else}
    <input class="prm-i" class:short={param.kind === "integer"} autocomplete="off" autocapitalize="off" spellcheck="false" inputmode={param.kind === "integer" ? "numeric" : undefined} placeholder={param.kind === "json" ? "JSON" : param.required ? "" : "not set"} bind:value={values[key]} />
  {/if}
</label>
{/snippet}

<div class="hd" style:align-items="flex-start">
  <div style:min-width="0">
    <h1 class="h1">Tool explorer</h1>
    <p class="meta" style:max-width="64ch" style:margin-top="4px">Every tool your agents can call on Mash, with what it takes and what it returns. Run the read-only ones here and see exactly what comes back.</p>
  </div>
  {#if listing.data}
    <span class="pills" style:gap="10px"><span class="pill plain mono">build {listing.data.build.slice(0, 8)}</span><span class="pill plain mono">contract {listing.data.contract.slice(0, 8)}</span></span>
  {/if}
</div>
<QueryError title="Couldn’t load the tool list" query={listing} />
{#if listing.isPending}<Skeleton rows={5} />{/if}
{#if listing.data}
  <div class="chips" style:gap="10px 14px">
    <label class="find"><SearchIcon size={15} aria-hidden="true" /><input type="search" aria-label="Filter tools" placeholder="Filter tools" autocomplete="off" bind:value={filter} /></label>
    <span class="meta" role="status">{tools.length} tools · {runnable} can run here · {tools.length - runnable} change data and are locked</span>
  </div>
  {#if !sections.length}<p class="card tight cream">No tool matches “{filter}”.</p>{/if}
  {#each sections as section (section.name)}
    {@const SectionIcon = sectionIcons[section.name] ?? Wrench}
    <section class="ops" aria-label={section.name}>
      <h2 class="h2"><SectionIcon size={15} aria-hidden="true" />{section.name === locked ? "changes data · locked" : section.name.toLowerCase()}</h2>
      <!-- History's descriptions are long, so its tools take whole rows. -->
      <div class="opl" class:g2={section.name.toLowerCase() !== "history"}>
      {#each section.tools as entry (entry.name)}
        {@const expanded = open === entry.name}
        <div class="mop" class:open={expanded} class:locked={!entry.readOnly} id="tool-{entry.name}">
          <button class="op-h" type="button" aria-expanded={expanded} onclick={() => toggle(entry.name)}>
            <span class="pill" class:ok={entry.readOnly} class:warn={!entry.readOnly}>{entry.readOnly ? "READ" : "WRITE"}</span>
            <code class="op-n">{entry.name}</code>
            {#if !entry.readOnly}<Lock size={13} aria-label="locked" />{/if}
            <span class="op-a">{headline(entry.about)}</span>
            {#if expanded}<ChevronDown size={16} aria-hidden="true" class="op-c" />{:else}<ChevronRight size={16} aria-hidden="true" class="op-c" />{/if}
          </button>
          {#if expanded}
            <form class="op-b" onsubmit={run}>
              {#if sentences(entry.about).length > 1}
                {@const notes = sentences(entry.about).slice(1)}
                <details class="disclose op-notes">
                  <summary class="a">How agents use it ({notes.length} note{notes.length === 1 ? "" : "s"})</summary>
                  <ul class="op-d">{#each notes as point, index (index)}<li>{@render prose(point)}</li>{/each}</ul>
                </details>
              {/if}
              {#if !entry.readOnly}<p class="meta">This tool changes data, so it can be run by an agent or from the command line, not from the dashboard. Its parameters are listed for reference.</p>{/if}
              <div class="op-cols" class:solo={!entry.readOnly}>
              <div class="op-in">
                <h3 class="eyebrow">Parameters</h3>
                {#if !entry.params.length}<p class="meta">This tool takes no parameters.</p>{/if}
                {#if fields(entry).always.length}<div class="prms">{#each fields(entry).always as param (param.name)}{@render field(entry, param)}{/each}</div>{/if}
                {#if fields(entry).extra.length}
                  {@const { always, extra } = fields(entry)}
                  <details class="disclose" bind:open={more[entry.name]}>
                    <summary class="a">{more[entry.name] ? "Fewer options" : `${extra.length} ${always.length ? "more " : ""}options`}</summary>
                    <div class="prms">{#each extra as param (param.name)}{@render field(entry, param)}{/each}</div>
                  </details>
                {/if}
                {#if entry.readOnly}
                  <div class="op-r">
                    <button class="btn p" type="submit" disabled={outcome.state === "running" || !request?.ok}><Play size={15} aria-hidden="true" />{outcome.state === "running" ? "Running…" : "Run"}</button>
                    <button class="btn" type="button" onclick={() => { for (const param of entry.params) delete values[`${entry.name}.${param.name}`]; }}><Eraser size={15} aria-hidden="true" />Clear values</button>
                    {#if request && !request.ok}<span class="meta">{request.message}</span>{/if}
                  </div>
                {/if}
              </div>
              {#if entry.readOnly}
                <div class="op-out">
                  {#if !request?.ok}
                    <div><div class="op-th"><h3 class="eyebrow">Request</h3></div><p class="json meta">The request appears here once its required parameters are filled in.</p></div>
                  {/if}
                  {#if request?.ok}
                    {@const call = JSON.stringify({ name: entry.name, arguments: request.arguments }, null, 2)}
                    <div>
                      <div class="op-th"><h3 class="eyebrow">Request</h3><button class="btn sm" type="button" style:margin-left="auto" onclick={() => copy(call, "Request")}><Copy size={13} aria-hidden="true" />Copy</button></div>
                      <div class="json"><JsonView value={{ name: entry.name, arguments: request.arguments }} /></div>
                    </div>
                  {/if}
                  {#if outcome.state === "unreachable"}<p class="card tight bad" role="alert"><span class="pill">Failed</span> {outcome.message}</p>{/if}
                  {#if outcome.state === "done" && summary}
                    {@const reply = JSON.stringify(outcome.answer.reply, null, 2)}
                    <section aria-label="Response">
                      <div class="op-th">
                        <h3 class="eyebrow">Response</h3>
                        <span class="pill {summary.state}">{summary.state === "ok" ? "OK" : summary.state === "busy" ? "Busy" : "Failed"}</span>
                        <span class="meta">{[summary.items === null ? "" : `${summary.items.toLocaleString()} item${summary.items === 1 ? "" : "s"}`, `${outcome.ms.toLocaleString()} ms`, size(outcome.bytes)].filter(Boolean).join(" · ")}</span>
                        <button class="btn sm" type="button" style:margin-left="auto" onclick={() => copy(reply, "JSON")}><Copy size={13} aria-hidden="true" />Copy JSON</button>
                      </div>
                      {#if summary.heading}<p class="op-e" role="alert">{summary.heading}</p>{/if}
                      {#if outcome.answer.reply !== null}<div class="json"><JsonView value={outcome.answer.reply} /></div>
                      {:else if outcome.answer.text}<div class="json"><pre>{outcome.answer.text}</pre></div>{/if}
                      {#if summary.state === "busy"}<p class="meta">Another Mash operation is using the index. It usually clears within a minute; run it again.</p>{/if}
                    </section>
                  {/if}
                  {#if notice}<p class="meta" role="status">{notice}</p>{/if}
                </div>
              {/if}
              </div>
            </form>
          {/if}
        </div>
      {/each}
      </div>
    </section>
  {/each}
{/if}
