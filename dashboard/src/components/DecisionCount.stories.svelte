<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import DecisionCount from "./DecisionCount.svelte";

const { Story } = defineMeta({
  title: "Blocks/DecisionCount",
  component: DecisionCount,
  args: { count: 3 },
});
</script>

<Story name="Default" />

<!-- The caller draws the pill around it, as a chat's row does. -->
<Story name="In a pill" asChild>
  <Cases row>
    {#each [1, 3, 1240] as count (count)}
      <Case label={count === 1 ? "one" : count.toLocaleString()}><span class="pill decision"><DecisionCount {count} /></span></Case>
    {/each}
  </Cases>
</Story>
