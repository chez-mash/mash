<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import StatusBadge from "../../components/StatusBadge.svelte";
import ToneIcon from "../../components/ToneIcon.svelte";
import type { Tone } from "../../lib/tone";

// What a state looks like at a glance: each tone's icon, its colour as text,
// and its soft fill as a pill. The word is always there; the colour and the
// shape only help it read (lib/tone.ts).
const { Story } = defineMeta({ title: "Foundations/Status tones" });

const tones: readonly { tone: Tone; name: string; words: string; means: string }[] = [
  { tone: "ok", name: "ok", words: "Running", means: "healthy, done, in force" },
  { tone: "warn", name: "warn", words: "Needs a look", means: "needs a look" },
  { tone: "bad", name: "bad", words: "Failed", means: "failed, replaced, contradicted" },
  { tone: "info", name: "info", words: "By meaning", means: "worth knowing, not a judgement" },
  { tone: "run", name: "running", words: "Indexing", means: "in progress" },
  { tone: "pause", name: "paused", words: "Stopped", means: "stopped on purpose" },
  {
    tone: "idle",
    name: "neutral",
    words: "Not indexed",
    means: "quiet or switched off, not a fault",
  },
  { tone: "unknown", name: "unknown", words: "No status", means: "nothing said either way" },
];
</script>

<Story name="Tones" asChild>
  <table>
    <thead>
      <tr><th>tone</th><th>icon</th><th>badge</th><th>pill</th><th>means</th></tr>
    </thead>
    <tbody>
      {#each tones as entry (entry.tone)}
        <tr>
          <th>{entry.name}</th>
          <td><ToneIcon tone={entry.tone} size={16} /></td>
          <td><StatusBadge tone={entry.tone} words={entry.words} /></td>
          <td><StatusBadge tone={entry.tone} words={entry.words} pill /></td>
          <td class="use">{entry.means}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</Story>

<Story name="Tinted cards" asChild>
  <div class="cards">
    <p class="card tight ok">A green card: something finished as asked.</p>
    <p class="card tight warn">An amber card: something needs a look.</p>
    <p class="card tight bad">A red card: something failed.</p>
    <p class="card tight cream">A quiet card: worth knowing before going on.</p>
  </div>
</Story>

<style>
  table {
    border-collapse: collapse;
  }
  th,
  td {
    padding: 9px 26px 9px 0;
    text-align: left;
    vertical-align: middle;
  }
  th {
    font-family: var(--type-mono);
    font-size: 11.5px;
    font-weight: var(--weight-regular);
    color: var(--text-faint);
  }
  .use {
    font-size: 12.5px;
    color: var(--text-muted);
  }
  .cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 560px;
  }
  .cards p {
    margin: 0;
    font-size: 13px;
  }
</style>
