<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import ExportOutcome from "./ExportOutcome.svelte";

// How an export ended, under the export button.
const { Story } = defineMeta({ title: "Blocks/ExportOutcome", component: ExportOutcome });
</script>

<Story name="Done" asChild>
  <div class="col"><ExportOutcome outcome="done">Exported 12 chats and 486 messages to mash-export-2026-11-20.zip.</ExportOutcome></div>
</Story>

<Story name="Failed" asChild>
  <div class="col"><ExportOutcome outcome="failed">Could not reach Mash while reading chat 4 of 12. Nothing was saved.</ExportOutcome></div>
</Story>

<Story name="Outcomes" asChild>
  <Cases width={560}>
    <Case label="done"><ExportOutcome outcome="done">Exported 1 chat and 38 messages to sign-in-tokens.md.</ExportOutcome></Case>
    <Case label="failed"><ExportOutcome outcome="failed">Could not reach Mash while reading chat 4 of 12. Nothing was saved.</ExportOutcome></Case>
    <Case label="narrow" note="a long file name wraps"><div class="narrow"><ExportOutcome outcome="done">Exported 12 chats and 486 messages to mash-export-storefront-checkout-2026-11-20.zip.</ExportOutcome></div></Case>
  </Cases>
</Story>

<style>
  .col {
    max-width: 560px;
  }
  .narrow {
    width: 280px;
  }
</style>
