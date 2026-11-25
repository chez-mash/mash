<script lang="ts">
import Eye from "@lucide/svelte/icons/eye";
import Save from "@lucide/svelte/icons/save";
import Undo2 from "@lucide/svelte/icons/undo-2";
import { mashCommand } from "../lib/command";
import type { Difference } from "../lib/settings";

// What is not yet saved, held at the foot of the screen: how many changes
// and which settings they touch, a way to review each one, and the save.
let {
  changed,
  blocked,
  saving,
  failure,
  saved,
  discard,
  save,
}: {
  changed: readonly Difference[];
  blocked: string;
  saving: boolean;
  failure: string;
  saved: number;
  discard: () => void;
  save: () => void;
} = $props();

let reviewing = $state(false);
$effect(() => {
  if (!changed.length) reviewing = false;
});
</script>

<div class="set-bar card" role="region" aria-label="Unsaved settings">
  {#if changed.length}
    <div class="set-bt">
      <span class="set-bs"><span class="set-dot" aria-hidden="true"></span><strong>{changed.length} unsaved change{changed.length === 1 ? "" : "s"}</strong><span class="meta">: {changed.map((change) => change.label).join(", ")}</span></span>
      <button class="btn ghost" type="button" aria-expanded={reviewing} onclick={() => (reviewing = !reviewing)}><Eye size={14} aria-hidden="true" />{reviewing ? "Hide changes" : "Review changes"}</button>
      <button class="btn ghost" type="button" disabled={saving} onclick={discard}><Undo2 size={14} aria-hidden="true" />Discard</button>
      <button class="btn p" type="button" disabled={saving || !!blocked} onclick={save}><Save size={15} aria-hidden="true" />{saving ? "Saving…" : "Save to file"}</button>
    </div>
    {#if blocked}<p class="meta warn" role="alert">{blocked}</p>{/if}
    {#if reviewing}
      <ul class="set-d">
        {#each changed as change (change.key)}<li><span>{change.label}</span><span class="was">{change.from}</span><span aria-hidden="true">→</span><span class="now">{change.to}</span></li>{/each}
      </ul>
      <p class="meta">The file is rewritten; the previous one is kept beside it as <code class="i">config.toml.previous</code>. Settings the page does not show are kept.</p>
    {/if}
  {/if}
  {#if failure}<p class="fail" role="alert">{failure}</p>{/if}
  {#if saved && !changed.length}
    <p class="ok" role="status">
      Saved {saved} change{saved === 1 ? "" : "s"} to the file. New requests use the saved
      settings. To update the index now, run <code class="i">{mashCommand("index")}</code> on this machine,
      or wait for the next pass if a watcher is running.
    </p>
  {/if}
</div>
