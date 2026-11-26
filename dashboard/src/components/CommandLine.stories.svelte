<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { ComponentProps } from "svelte";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import Narrow from "../stories/Narrow.svelte";
import CommandLine from "./CommandLine.svelte";

// A terminal command on one line with a button that copies it.
const { Story } = defineMeta({
  title: "Blocks/CommandLine",
  component: CommandLine,
  args: { text: "mash export --format archive" },
  render: narrow,
});
</script>

{#snippet narrow(args: ComponentProps<typeof CommandLine>)}
  <Narrow width={480}><CommandLine {...args} /></Narrow>
{/snippet}

<Story name="Default" />

<Story name="Lengths" asChild>
  <Cases width={480}>
    <Case label="short"><CommandLine text="mash models install" /></Case>
    <Case label="long"><CommandLine text="mash export --format archive --output ~/Backups/mash-2026-11-20.zip" /></Case>
  </Cases>
</Story>
