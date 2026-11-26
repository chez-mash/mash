<script lang="ts">
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import TerminalIcon from "phosphor-svelte/lib/TerminalIcon";
import Icon from "$lib/components/Icon.svelte";
import { argumentHighlight, outputHighlight } from "../lib/codeLanguage";
import { durationLabel, type ToolCall } from "../lib/toolcalls";
import Code from "./Code.svelte";
import JsonView from "./JsonView.svelte";
import ToneIcon from "./ToneIcon.svelte";

// One tool call with its output, laid out by the same rules for every tool:
// a header to scan, the arguments as labelled rows, then the output. All of
// it is archived text and is rendered as text.
let { call, selected = "" }: { call: ToolCall; selected?: string } = $props();
const LINES = 24;
const CHARACTERS = 4000;
const hit = $derived(
  !!selected && (selected === call.id || (!!call.resultId && selected === call.resultId)),
);
let all = $state(false);
let raw = $state(false);
const lines = $derived(call.output?.text.split("\n") ?? []);
const long = $derived(lines.length > LINES || (call.output?.text.length ?? 0) > CHARACTERS);
// Colour only where the language is certain; see lib/codeLanguage.ts.
const outputLanguage = $derived(outputHighlight(call));
const shown = $derived(
  all || !long ? (call.output?.text ?? "") : lines.slice(0, LINES).join("\n").slice(0, CHARACTERS),
);
</script>

<details class="tc" class:hit class:bad={call.status?.failed} id="message-{call.id}" data-message={call.id} open={hit} style:scroll-margin-top="80px">
  <summary>
    <Icon icon={TerminalIcon} size={15} />
    <span class="tc-n">{call.name}</span>
    <span class="tc-s">{call.summary}</span>
    {#if call.status}<span class="pill" class:ok={!call.status.failed} class:bad={call.status.failed}><ToneIcon tone={call.status.failed ? "bad" : "ok"} size={12} />{call.status.label}</span>{/if}
    {#if call.durationMs !== undefined}<span class="pill">{durationLabel(call.durationMs)}</span>{/if}
    {#if call.server}<span class="pill">{call.server}</span>{/if}
    <span class="mtx">{call.at}</span>
    <span class="cv"><Icon icon={CaretRightIcon} size={15} /></span>
  </summary>
  <div class="tc-b">
    {#each call.args as argument (argument.key)}
      <div class="tc-r">
        <span class="tc-k">{argument.key}</span>
        {#if argument.block}{@const coloured = argumentHighlight(call, argument)}<Code class="tc-pre" wrap={!(coloured?.text ?? argument.text).includes("\n")} text={coloured?.text ?? argument.text} language={coloured?.language} tone={coloured?.tone} />{:else}<span class="tc-v">{argument.text}</span>{/if}
      </div>
    {/each}
    {#if call.output}
      <div class="tc-o" id={call.resultId ? `message-${call.resultId}` : undefined} data-message={call.resultId} style:scroll-margin-top="80px">
        <span class="tc-h">output</span>
        {#if call.output.json !== undefined}
          <div class="tc-pre"><JsonView value={call.output.json} /></div>
        {:else}
          <Code class="tc-pre" wrap={lines.length < 3} text={shown} language={outputLanguage?.language} numbered={outputLanguage?.numbered} />
          {#if long}
            <p class="tc-f">
              <span>{all ? `${lines.length.toLocaleString()} lines` : `${Math.min(LINES, lines.length).toLocaleString()} of ${lines.length.toLocaleString()} lines`}</span>
              <button class="a" type="button" onclick={() => (all = !all)}>{all ? "Show less" : "Show all output"}</button>
            </p>
          {/if}
        {/if}
      </div>
    {:else if !call.args.length}
      <p class="meta">Nothing further was recorded for this call.</p>
    {/if}
    <p class="tc-f"><button class="a" type="button" aria-expanded={raw} onclick={() => (raw = !raw)}>{raw ? "Hide raw record" : "Raw record"}</button></p>
    {#if raw}<pre class="tc-pre wrap">{call.raw}</pre>{/if}
  </div>
</details>
