<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { kinds } from "../lib/records";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import KindPill from "./KindPill.svelte";

const { Story } = defineMeta({
  title: "Blocks/KindPill",
  component: KindPill,
  args: { kind: "decision" },
});
</script>

<Story name="Default" />

<Story name="Kinds" asChild>
  <Cases row>
    {#each kinds as kind (kind)}
      <Case label={kind}><KindPill {kind} /></Case>
      <Case label="{kind}, lower"><KindPill {kind} lower /></Case>
    {/each}
  </Cases>
</Story>
