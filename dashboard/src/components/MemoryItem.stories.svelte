<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { note, now, recordOf, revised } from "../stories/fixtures";
import MemoryItem from "./MemoryItem.svelte";

// One memory in the memory page's list. Anything that should give the reader
// pause is a pill under the statement.
const decision = recordOf("decision");
const preference = recordOf("preference");
const shared = { back: "#/records", machine: "laptop", now };

const { Story } = defineMeta({
  title: "Blocks/MemoryItem",
  component: MemoryItem,
  args: { record: decision, ...shared },
});

const kinds = [
  { label: "decision", record: decision },
  { label: "preference", record: preference },
  { label: "note", record: note() },
];
const states = [
  { label: "established", record: revised(decision, { review: "established" }) },
  { label: "unconfirmed", record: revised(decision, { review: "candidate" }) },
  { label: "stale evidence", record: revised(decision, { evidenceCurrent: false }) },
  { label: "conflicting", record: revised(decision, {}, { conflicted: true }) },
  {
    label: "replaced",
    record: revised(decision, {
      details: {
        kind: "decision",
        state: "superseded",
        decidedBy: "user",
        alternatives: [],
        consequences: [],
      },
    }),
  },
  {
    label: "everything at once",
    record: revised(
      decision,
      { review: "candidate", evidenceCurrent: false },
      { conflicted: true },
    ),
  },
];
</script>

<Story name="Default" />

<Story name="Current" args={{ selected: true }} />

<Story name="Kinds" asChild>
  <Cases width={760}>
    {#each kinds as entry (entry.label)}
      <Case label={entry.label}><MemoryItem record={entry.record} {...shared} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="States" asChild>
  <Cases width={760}>
    {#each states as entry (entry.label)}
      <Case label={entry.label}><MemoryItem record={entry.record} {...shared} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="Kinds by state" asChild>
  <Cases width={760}>
    {#each kinds as kind (kind.label)}
      <Case label="{kind.label}, unconfirmed"><MemoryItem record={revised(kind.record, { review: "candidate" })} {...shared} /></Case>
      <Case label="{kind.label}, stale evidence"><MemoryItem record={revised(kind.record, { evidenceCurrent: false })} {...shared} /></Case>
      <Case label="{kind.label}, conflicting"><MemoryItem record={revised(kind.record, {}, { conflicted: true })} {...shared} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="On a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} asChild>
  <Cases>
    <Case label="established"><MemoryItem record={decision} {...shared} /></Case>
    <Case label="unconfirmed, conflicting"><MemoryItem record={revised(preference, { review: "candidate" }, { conflicted: true })} {...shared} /></Case>
  </Cases>
</Story>
