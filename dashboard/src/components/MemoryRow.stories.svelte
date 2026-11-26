<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { note, recordOf, revised } from "../stories/fixtures";
import MemoryRow from "./MemoryRow.svelte";

// One memory in a short card list, as the overview's side column shows them.
const decision = recordOf("decision");
const preference = recordOf("preference");

const { Story } = defineMeta({ title: "Blocks/MemoryRow", component: MemoryRow });
</script>

<Story name="Default" asChild>
  <div class="card pad0 list col"><MemoryRow record={decision} /></div>
</Story>

<Story name="Kinds and standing" asChild>
  <Cases width={360}>
    <Case label="decision"><div class="card pad0 list"><MemoryRow record={revised(decision, { review: "established" })} /></div></Case>
    <Case label="preference"><div class="card pad0 list"><MemoryRow record={revised(preference, { review: "established" })} /></div></Case>
    <Case label="note"><div class="card pad0 list"><MemoryRow record={note()} /></div></Case>
    <Case label="unconfirmed"><div class="card pad0 list"><MemoryRow record={revised(decision, { review: "candidate" })} /></div></Case>
    <Case label="agent decided">
      <div class="card pad0 list">
        <MemoryRow record={revised(decision, { review: "established", details: { kind: "decision", state: "accepted", decidedBy: "agent", alternatives: [], consequences: [] } })} />
      </div>
    </Case>
    <Case label="a list">
      <div class="card pad0 list"><MemoryRow record={decision} /><MemoryRow record={preference} /><MemoryRow record={note()} /></div>
    </Case>
  </Cases>
</Story>

<style>
  .col {
    max-width: 360px;
  }
</style>
