<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { Tone } from "../lib/tone";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import StatusBadge from "./StatusBadge.svelte";

const { Story } = defineMeta({
  title: "Blocks/StatusBadge",
  component: StatusBadge,
  args: { tone: "ok", words: "Running" },
});

const tones: readonly { tone: Tone; words: string }[] = [
  { tone: "ok", words: "Running" },
  { tone: "warn", words: "Needs a look" },
  { tone: "bad", words: "Failed" },
  { tone: "info", words: "Proposed" },
  { tone: "run", words: "Indexing" },
  { tone: "pause", words: "Stopped" },
  { tone: "idle", words: "Not indexed" },
  { tone: "unknown", words: "No status" },
];
</script>

<Story name="Default" />

<Story name="Pill" args={{ pill: true }} />

<Story name="Tones" asChild>
  <Cases row>
    {#each tones as entry (entry.tone)}
      <Case label={entry.tone}><StatusBadge {...entry} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="Pills" asChild>
  <Cases row>
    {#each tones as entry (entry.tone)}
      <Case label={entry.tone}><StatusBadge {...entry} pill /></Case>
    {/each}
  </Cases>
</Story>
