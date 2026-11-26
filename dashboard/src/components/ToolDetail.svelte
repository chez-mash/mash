<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import Copy from "@lucide/svelte/icons/copy";
import Eraser from "@lucide/svelte/icons/eraser";
import Play from "@lucide/svelte/icons/play";
import * as Select from "$lib/components/ui/select";
import { read } from "../lib/api";
import {
  type Answer,
  argumentsFor,
  foldAfter,
  type Param,
  parseAnswer,
  sentences,
  spans,
  summarize,
  type Tool,
} from "../lib/console";
import JsonView from "./JsonView.svelte";

// One tool, opened from the explorer's list: what it is for, its parameters
// and, for a tool that only reads, a form that runs it and shows the answer.
// The server refuses tools that change data, whatever this page sends.
let {
  entry,
  values = $bindable(),
}: {
  entry: Tool;
  /** Typed values by "tool.parameter", kept while the reader moves between tools. */
  values: Record<string, string>;
} = $props();

// Whether the optional parameters are unfolded.
let more = $state(false);
type Outcome =
  | { readonly state: "idle" }
  | { readonly state: "running" }
  | { readonly state: "done"; readonly answer: Answer; readonly ms: number; readonly bytes: number }
  | { readonly state: "unreachable"; readonly message: string };
let outcome = $state<Outcome>({ state: "idle" });
let controller: AbortController | undefined;
let notice = $state("");
const request = $derived(argumentsFor(entry, values));
const summary = $derived(outcome.state === "done" ? summarize(outcome.answer) : null);

async function run(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  if (!request.ok) return;
  controller?.abort();
  const current = new AbortController();
  controller = current;
  outcome = { state: "running" };
  notice = "";
  const started = performance.now();
  try {
    const body = await read(
      "/api/console/call",
      new URLSearchParams({ name: entry.name, arguments: JSON.stringify(request.arguments) }),
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
<label class="prm" class:bad={!request.ok && request.param === param.name && outcome.state !== "idle"}>
  <span class="prm-n"><code>{param.name}</code><small>{param.type}</small>{#if param.required}<b class="pill">required</b>{/if}</span>
  <span class="prm-a">{@render prose(param.about)}{#if param.fallback}{" "}<small>Default {param.fallback}.</small>{/if}</span>
  {#if !entry.readOnly}<span hidden></span>
  {:else if param.kind === "enum" || param.kind === "boolean"}
    {@const choices = param.kind === "boolean" ? ["true", "false"] : param.choices}
    {@const unset = param.required ? "choose…" : "not set"}
    <Select.Root type="single" bind:value={() => values[key] ?? "", (choice) => (values[key] = choice)}>
      <Select.Trigger class="mt-1 font-mono text-[14px]" aria-invalid={!request.ok && request.param === param.name && outcome.state !== "idle"}>{values[key] || unset}</Select.Trigger>
      <Select.Content class="font-mono">
        <Select.Item class="text-muted" value="" label={unset} />
        {#each choices as choice (choice)}<Select.Item value={choice} label={choice} />{/each}
      </Select.Content>
    </Select.Root>
  {:else}
    <input class="prm-i" class:short={param.kind === "integer"} autocomplete="off" autocapitalize="off" spellcheck="false" inputmode={param.kind === "integer" ? "numeric" : undefined} placeholder={param.kind === "json" ? "JSON" : param.required ? "" : "not set"} bind:value={values[key]} />
  {/if}
</label>
{/snippet}

<div class="mop" class:locked={!entry.readOnly} id="tool-{entry.name}">
  <div class="op-top">
    <code class="op-n">{entry.name}</code>
    <span class="op-k" class:ok={entry.readOnly}>{entry.readOnly ? "READ" : "WRITE"}</span>
  </div>
  <p class="op-lead">{@render prose(sentences(entry.about)[0] ?? entry.about)}</p>
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
        <details class="disclose" bind:open={more}>
          <summary class="a">{more ? "Fewer options" : `${extra.length} ${always.length ? "more " : ""}options`}</summary>
          <div class="prms">{#each extra as param (param.name)}{@render field(entry, param)}{/each}</div>
        </details>
      {/if}
      {#if entry.readOnly}
        <div class="op-r">
          <button class="btn p" type="submit" disabled={outcome.state === "running" || !request.ok}><Play size={15} aria-hidden="true" />{outcome.state === "running" ? "Running…" : "Run"}</button>
          <button class="btn" type="button" onclick={() => { for (const param of entry.params) delete values[`${entry.name}.${param.name}`]; }}><Eraser size={15} aria-hidden="true" />Clear values</button>
          {#if !request.ok}<span class="meta">{request.message}</span>{/if}
        </div>
      {/if}
    </div>
    {#if entry.readOnly}
      <div class="op-out">
        {#if !request.ok}
          <div><div class="op-th"><h3 class="eyebrow">Request</h3></div><p class="json meta">The request appears here once its required parameters are filled in.</p></div>
        {/if}
        {#if request.ok}
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
</div>
