<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { ComponentProps } from "svelte";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import Narrow from "../stories/Narrow.svelte";
import CommandCopy from "./CommandCopy.svelte";

// A command to run elsewhere, in a muted box that wraps it.
const { Story } = defineMeta({
  title: "Blocks/CommandCopy",
  component: CommandCopy,
  args: { text: "mash dash remote --code" },
  render: narrow,
});

const long = "mash index --rebuild-local-views --full && mash status --shared";
</script>

{#snippet narrow(args: ComponentProps<typeof CommandCopy>)}
  <Narrow width={420}><CommandCopy {...args} /></Narrow>
{/snippet}

<Story name="Default" />

<Story name="Lengths" asChild>
  <Cases width={420}>
    <Case label="short"><CommandCopy text="mash dash remote --code" /></Case>
    <Case label="long" note="it wraps instead of scrolling"><CommandCopy text={long} /></Case>
    <Case label="narrow"><div class="narrow"><CommandCopy text={long} /></div></Case>
  </Cases>
</Story>

<style>
  .narrow {
    width: 240px;
  }
</style>
