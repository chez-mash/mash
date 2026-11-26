<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../../../../stories/Case.svelte";
import Cases from "../../../../stories/Cases.svelte";
import * as Select from "./index";

// A select looks like the text fields beside it; its list is a raised panel
// under the button, at least as wide.
const { Story } = defineMeta({ title: "Kit/Select", component: Select.Root, tags: ["screen"] });

const choices = ["related-discussion", "exact-phrase", "recent-first"];
</script>

{#snippet select(value: string, options: { open?: boolean; disabled?: boolean; invalid?: boolean } = {})}
  <Select.Root type="single" {value} open={options.open ?? false} disabled={options.disabled ?? false}>
    <Select.Trigger aria-label="Intent" aria-invalid={options.invalid ?? false}>{value || "not set"}</Select.Trigger>
    <Select.Content>
      <Select.Item class="text-muted" value="" label="not set" />
      {#each choices as choice (choice)}<Select.Item value={choice} label={choice} />{/each}
    </Select.Content>
  </Select.Root>
{/snippet}

<Story name="Chosen" asChild>{@render select("related-discussion")}</Story>

<Story name="States" asChild>
  <Cases row>
    <Case label="chosen">{@render select("related-discussion")}</Case>
    <Case label="not set">{@render select("")}</Case>
    <Case label="invalid">{@render select("", { invalid: true })}</Case>
    <Case label="disabled">{@render select("exact-phrase", { disabled: true })}</Case>
  </Cases>
</Story>

<Story name="Open" asChild>{@render select("exact-phrase", { open: true })}</Story>

<Story name="Focused" play={async ({ canvas }) => { canvas.getByRole("combobox").focus(); }} asChild>
  {@render select("related-discussion")}
</Story>
