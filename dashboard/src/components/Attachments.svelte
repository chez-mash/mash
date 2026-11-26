<script lang="ts">
import Download from "@lucide/svelte/icons/download";
import FileIcon from "@lucide/svelte/icons/file";
import FileX from "@lucide/svelte/icons/file-x";
import X from "@lucide/svelte/icons/x";
import * as Dialog from "$lib/components/ui/dialog";
import { kind, size } from "../lib/attachments";
import type { Attachment } from "../lib/types";

// A message's files under its text: kept pictures as thumbnails that open
// full size, other kept files as cards that download, and files Mash did not
// keep as cards that say why. Nothing here is drawn from a file the server
// would not show inline, and a picture that fails to load becomes a card.
let { attachments }: { attachments: readonly Attachment[] } = $props();
let failed = $state<Record<string, boolean>>({});
let open = $state<Attachment>();
// A picture that fails to load moves down among the cards.
const shownAsPicture = (file: Attachment): boolean =>
  !!(file.image && file.url && !failed[file.id]);
const pictures = $derived(attachments.filter(shownAsPicture));
const others = $derived(attachments.filter((file) => !shownAsPicture(file)));
</script>

<!-- Pictures share a row; every other file is a card on a row of its own
     under them, at its own height. -->
{#snippet card(file: Attachment)}
  {#if file.url && !failed[file.id]}
    <div class="card tight file">
      <FileIcon size={18} aria-hidden="true" />
      <span class="what"><b>{file.name}</b><span class="mtx">{kind(file.mediaType)}{file.bytes === undefined ? "" : ` · ${size(file.bytes)}`}</span></span>
      <a class="a get" href={file.downloadUrl ?? file.url} download={file.name} aria-label="Download {file.name}"><Download size={15} aria-hidden="true" />Download</a>
    </div>
  {:else}
    <div class="card tight cream file gone">
      <FileX size={18} aria-hidden="true" />
      <span class="what"><b>{file.name}</b><span class="mtx">Not kept. {failed[file.id] ? "The file could not be loaded" : (file.missing ?? "No copy was kept")}.</span></span>
    </div>
  {/if}
{/snippet}
<div class="atts">
  {#if pictures.length}
    <div class="pics">
      {#each pictures as file (file.id)}
        <button type="button" class="thumb" onclick={() => (open = file)} aria-label="Open {file.name} full size" title={file.name}>
          <img src={file.url} alt={file.name} loading="lazy" decoding="async" onerror={() => (failed[file.id] = true)} />
        </button>
      {/each}
    </div>
  {/if}
  {#each others as file (file.id)}{@render card(file)}{/each}
</div>

<Dialog.Root bind:open={() => open !== undefined, (on) => { if (!on) open = undefined; }}>
  <Dialog.Content class="full flex h-[min(646px,94vh)] w-[min(520px,96vw)] flex-col" aria-describedby={undefined}>
    {#if open}
      <div class="viewbar">
        <Dialog.Title class="viewname" level={2}>{open.name}</Dialog.Title>
        <a class="btn icon" href={open.downloadUrl ?? open.url} download={open.name} aria-label="Download {open.name}" title="Download"><Download size={15} aria-hidden="true" /></a>
        <Dialog.Close class="btn icon" aria-label="Close" title="Close"><X size={15} aria-hidden="true" /></Dialog.Close>
      </div>
      <div class="viewbody"><img src={open.url} alt={open.name} /></div>
    {/if}
  </Dialog.Content>
</Dialog.Root>

<style>
/* Pictures as rounded thumbnails, kept files as raised cards with their
   download, and files Mash did not keep as quieter cards saying why. */
.atts { display: flex; flex-direction: column; align-items: flex-start; gap: 8px; margin-top: 8px; }
.pics { display: flex; flex-wrap: wrap; gap: 8px; }
.thumb { display: block; padding: 0; border: 0; border-radius: var(--radius-7); box-shadow: 0 0 0 var(--stroke-outline) var(--line-outline); overflow: hidden; background: var(--surface-muted); cursor: zoom-in; max-width: min(320px, 100%); }
.thumb:hover { box-shadow: 0 0 0 var(--stroke-outline) var(--line-strong); }
.thumb img { display: block; height: auto; max-height: 180px; max-width: 100%; object-fit: contain; }
.file { display: flex; align-items: center; gap: 12px; min-width: min(300px, 100%); max-width: min(380px, 100%); padding: 12px 16px; border: 0; border-radius: var(--radius-7); background: var(--surface-raised); box-shadow: 0 0 0 var(--stroke-outline) var(--line-outline); font-size: 13.5px; line-height: 1.35; color: var(--text-body); text-decoration: none; }
.file.gone { background: var(--surface-muted); }
.file .what { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.file b { font-weight: var(--weight-strong); color: var(--text-strong); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file .mtx { white-space: normal; font-size: 12px; color: var(--text-faint); }
.file :global(svg) { flex: none; color: var(--text-muted); }
.file .get { display: inline-flex; align-items: center; gap: 6px; margin-left: auto; padding-left: 12px; font-weight: var(--weight-strong); white-space: nowrap; color: var(--text-strong); text-decoration: none; }
.file .get :global(svg) { color: inherit; }
/* The viewer is the kit's dialog; these lay out what is inside it. */
:global(.full) .viewbar { display: flex; align-items: center; gap: 8px; padding: 8px 8px 8px 16px; border-bottom: var(--border-divider); }
:global(.full .viewname) { flex: 1; min-width: 0; margin: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13.5px; font-weight: var(--weight-strong); color: var(--text-strong); }
:global(.full) .viewbar :global(.btn.icon) { width: 32px; height: 32px; min-height: 32px; border-radius: var(--radius-5); }
:global(.full) .viewbody { flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; padding: 24px; background: var(--surface-muted); }
:global(.full) img { display: block; max-width: 100%; max-height: 100%; margin: 0 auto; border-radius: var(--radius-7); object-fit: contain; }
</style>
