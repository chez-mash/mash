<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { kinds } from "../lib/records";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import KindIcon from "./KindIcon.svelte";

const { Story } = defineMeta({
  title: "Blocks/KindIcon",
  component: KindIcon,
  args: { kind: "decision" },
});
</script>

<Story name="Default" />

<Story name="Kinds" asChild>
  <Cases row>
    {#each kinds as kind (kind)}
      <Case label={kind}><KindIcon {kind} size={16} /></Case>
    {/each}
  </Cases>
</Story>
