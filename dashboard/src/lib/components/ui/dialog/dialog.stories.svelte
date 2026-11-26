<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import XIcon from "phosphor-svelte/lib/XIcon";
import Icon from "$lib/components/Icon.svelte";
import { openedByPointer } from "../../../../stories/play";
import * as Dialog from "./index";

// A dialog: a raised panel centred over the dimmed page. Its size and inner
// layout are the caller's.
const { Story } = defineMeta({ title: "Kit/Dialog", component: Dialog.Root, tags: ["screen"] });
</script>

{#snippet dialog(open: boolean)}
  <p class="behind">The page behind the dialog. It is dimmed and inert while the dialog is open.</p>
  <Dialog.Root {open}>
    <Dialog.Content class="flex h-[min(646px,94vh)] w-[min(520px,96vw)] flex-col" aria-describedby={undefined}>
      <header class="head">
        <Dialog.Title level={2}>receipt-2026-11.pdf</Dialog.Title>
        <Dialog.Close class="btn icon" aria-label="Close"><Icon icon={XIcon} size={15} /></Dialog.Close>
      </header>
      <div class="body">A file a message carried is shown here, at the size the panel gives it.</div>
    </Dialog.Content>
  </Dialog.Root>
{/snippet}

<Story name="Closed" asChild>{@render dialog(false)}</Story>

<Story name="Open" play={openedByPointer} asChild>{@render dialog(true)}</Story>

<Story name="Open on a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} play={openedByPointer} asChild>{@render dialog(true)}</Story>

<style>
  .behind {
    margin: 0;
    font-size: 14px;
    color: var(--text-body);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 12px 12px 18px;
    border-bottom: var(--border-divider);
    font-size: 14px;
    color: var(--text-strong);
  }
  .body {
    flex: 1;
    padding: 18px;
    font-size: 13.5px;
    color: var(--text-muted);
  }
</style>
