<script lang="ts">
import Download from "@lucide/svelte/icons/download";
import FileIcon from "@lucide/svelte/icons/file";
import FileX from "@lucide/svelte/icons/file-x";
import X from "@lucide/svelte/icons/x";
import { kind, size } from "../lib/attachments";
import type { Attachment } from "../lib/types";

// A message's files under its text: kept pictures as thumbnails that open
// full size, other kept files as cards that download, and files Mash did not
// keep as cards that say why. Nothing here is drawn from a file the server
// would not show inline, and a picture that fails to load becomes a card.
let { attachments }: { attachments: readonly Attachment[] } = $props();
let failed = $state<Record<string, boolean>>({});
let open = $state<Attachment>();
let dialog = $state<HTMLDialogElement>();
$effect(() => {
  if (open && dialog && !dialog.open) dialog.showModal();
});
function close(): void {
  dialog?.close();
}
</script>

<div class="atts">
  {#each attachments as file (file.id)}
    {#if file.image && file.url && !failed[file.id]}
      <button type="button" class="thumb" onclick={() => (open = file)} aria-label="Open {file.name} full size" title={file.name}>
        <img src={file.url} alt={file.name} loading="lazy" decoding="async" onerror={() => (failed[file.id] = true)} />
      </button>
    {:else if file.url && !failed[file.id]}
      <div class="card tight file">
        <FileIcon size={18} aria-hidden="true" />
        <span class="what"><b>{file.name}</b><span class="mtx">{kind(file.mediaType)}{file.bytes === undefined ? "" : ` · ${size(file.bytes)}`}</span></span>
        <a class="a get" href={file.url} download={file.name} aria-label="Download {file.name}"><Download size={15} aria-hidden="true" />Download</a>
      </div>
    {:else}
      <div class="card tight cream file gone">
        <FileX size={18} aria-hidden="true" />
        <span class="what"><b>{file.name}</b><span class="mtx">Not kept. {failed[file.id] ? "The file could not be loaded" : (file.missing ?? "No copy was kept")}.</span></span>
      </div>
    {/if}
  {/each}
</div>

{#if open}
  <dialog bind:this={dialog} class="full" aria-label={open.name} onclose={() => (open = undefined)} onclick={(event) => { if (event.target === dialog) close(); }}>
    <div class="viewbar">
      <span class="viewname">{open.name}</span>
      <a class="btn icon" href={open.url} download={open.name} aria-label="Download {open.name}" title="Download"><Download size={15} aria-hidden="true" /></a>
      <button type="button" class="btn icon" onclick={close} aria-label="Close" title="Close"><X size={15} aria-hidden="true" /></button>
    </div>
    <img src={open.url} alt={open.name} />
  </dialog>
{/if}

<style>
.atts { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px; }
.thumb { display: block; padding: 0; border: var(--border-outline); border-radius: var(--radius-5); overflow: hidden; background: var(--surface-raised); cursor: zoom-in; max-width: min(320px, 100%); }
.thumb:hover { border-color: var(--line-strong); }
.thumb img { display: block; height: auto; max-height: 180px; max-width: 100%; object-fit: contain; }
.file { display: flex; align-items: center; gap: 10px; min-width: 0; max-width: min(360px, 100%); font-size: 13.5px; line-height: 1.35; color: var(--text-body); text-decoration: none; }
.file .what { display: flex; flex-direction: column; min-width: 0; }
.file b { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file .mtx { white-space: normal; }
.file :global(svg) { flex: none; }
.file .get { display: inline-flex; align-items: center; gap: 4px; margin-left: auto; padding-left: 8px; font-weight: 600; white-space: nowrap; }
.full { position: fixed; inset: 0; margin: auto; min-width: min(320px, 96vw); max-width: min(96vw, 1200px); max-height: 94vh; padding: 0; border: var(--border-strong); border-radius: var(--radius-7); background: var(--surface-raised); color: var(--text-body); }
.full::backdrop { background: var(--scrim); }
.full .viewbar { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-bottom: var(--border-outline); }
.full .viewname { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13.5px; font-weight: 600; }
.full img { display: block; max-width: 100%; max-height: calc(94vh - 56px); margin: 0 auto; object-fit: contain; }
</style>
