<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { ComponentProps } from "svelte";
import { currentOf } from "../lib/memoryView";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { recordOf } from "../stories/fixtures";
import Narrow from "../stories/Narrow.svelte";
import SavedMemory from "./SavedMemory.svelte";

// Under a message: each memory saved from it, in a butter ring.
const decision = recordOf("decision");
const preference = recordOf("preference");
const quoted = (record: typeof decision): string => currentOf(record)?.evidence[0]?.messageId ?? "";
const memoryParams = new URLSearchParams();

const { Story } = defineMeta({
  title: "Blocks/SavedMemory",
  component: SavedMemory,
  args: { records: [decision], messageId: quoted(decision), memoryParams },
  render: narrow,
});
</script>

{#snippet narrow(args: ComponentProps<typeof SavedMemory>)}
  <Narrow width={680}><SavedMemory {...args} /></Narrow>
{/snippet}

<Story name="A decision" />

<Story name="A preference" args={{ records: [preference], messageId: quoted(preference) }} />

<Story name="Kinds" asChild>
  <Cases width={680}>
    <Case label="a decision"><SavedMemory records={[decision]} messageId={quoted(decision)} {memoryParams} /></Case>
    <Case label="a preference"><SavedMemory records={[preference]} messageId={quoted(preference)} {memoryParams} /></Case>
    <Case label="another message" note="nothing was saved from it, so nothing is drawn"><SavedMemory records={[decision]} messageId="another-message" {memoryParams} /></Case>
  </Cases>
</Story>
