<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import Skeleton from "./Skeleton.svelte";

// Placeholder rows shown while a page's first read is in flight.
const { Story } = defineMeta({ title: "Blocks/Skeleton", component: Skeleton, args: { rows: 4 } });
</script>

<Story name="Default" />

<Story name="Forms" asChild>
  <Cases width={640}>
    <Case label="on the page"><Skeleton rows={3} /></Case>
    <Case label="in a card"><div class="card pad0"><Skeleton rows={3} /></div></Case>
    <Case label="one row"><Skeleton rows={1} /></Case>
  </Cases>
</Story>
