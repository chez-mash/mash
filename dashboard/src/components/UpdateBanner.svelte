<script lang="ts">
import ArrowsClockwiseIcon from "phosphor-svelte/lib/ArrowsClockwiseIcon";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";
import Icon from "$lib/components/Icon.svelte";
import { type UpdateOffer, updateView } from "../lib/appUpdate.svelte";
import ToneIcon from "./ToneIcon.svelte";

// The strip across the top of the desktop app's window when a newer Mash
// exists: which version, one button that gets it the way this copy can
// (install and restart, the downloads page, or the apt command) and Later.
// While the app downloads or installs it shows that instead, and a failure
// with its reason. The app decides all of it (lib/appUpdate.svelte.ts).
let {
  offer,
  oninstall,
  onlater,
}: { offer: UpdateOffer; oninstall: () => void; onlater: () => void } = $props();
const view = $derived(updateView(offer));
let copied = $state(false);

async function copy(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    copied = true;
  } catch {
    copied = false;
  }
}
</script>

<aside class="upd" class:warn={view.tone === "warn"} aria-label="Mash update">
  <ToneIcon tone={view.tone} size={16} />
  <div class="upd-t" role="status">
    <b>{view.title}</b>
    {#if view.detail}<span>{view.detail}</span>{/if}
    {#if view.command}<code>{view.command}</code>{/if}
    {#if typeof view.progress === "number"}
      <span class="upd-bar" role="progressbar" aria-label="Download" aria-valuemin={0} aria-valuemax={100} aria-valuenow={view.progress}><i style:width="{view.progress}%"></i></span>
    {/if}
  </div>
  {#if typeof view.progress === "number"}<span class="upd-n">{view.progress}%</span>{/if}
  {#if view.action || view.later}
    <div class="upd-a">
      {#if view.action?.kind === "install"}
        <button class="btn p sm" type="button" onclick={oninstall}><Icon icon={ArrowsClockwiseIcon} size={14} state="strong" />{view.action.label}</button>
      {:else if view.action?.kind === "link"}
        <a class="btn p sm" href={view.action.href} target="_blank" rel="noreferrer"><Icon icon={DownloadSimpleIcon} size={14} state="strong" />{view.action.label}</a>
      {:else if view.action?.kind === "copy"}
        {@const text = view.action.text}
        <button class="btn p sm" type="button" onclick={() => copy(text)}><Icon icon={copied ? CheckIcon : CopyIcon} size={14} state="strong" />{copied ? "Copied" : view.action.label}</button>
      {/if}
      {#if view.later}<button class="btn sm ghost" type="button" onclick={onlater}>Later</button>{/if}
    </div>
  {/if}
</aside>

<style>
  .upd {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
    padding: 8px max(16px, env(safe-area-inset-right, 0px)) 8px max(16px, env(safe-area-inset-left, 0px));
    border-bottom: var(--border-divider);
    background: var(--status-info-bg);
    color: var(--text-strong);
    font-family: var(--type-sans);
    font-size: 13.5px;
    line-height: 1.45;
  }
  .upd.warn {
    background: var(--status-warn-bg);
  }
  .upd-t {
    flex: 1 1 240px;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 2px 10px;
  }
  .upd-t b {
    font-weight: var(--weight-strong);
  }
  .upd-t span {
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }
  .upd-t code {
    flex-basis: 100%;
    font-family: var(--type-mono);
    font-size: 12.5px;
    color: var(--text-strong);
    overflow-wrap: anywhere;
    user-select: all;
  }
  /* The download's share that has arrived: a thin track filling with butter. */
  .upd-bar {
    flex-basis: 100%;
    display: block;
    height: 4px;
    max-width: 420px;
    margin-top: 4px;
    border-radius: var(--radius-pill);
    background: var(--surface-hover);
    overflow: hidden;
  }
  .upd-bar > i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--progress);
  }
  .upd-n {
    font-family: var(--type-mono);
    font-size: 12.5px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }
  .upd-a {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
</style>
