<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import CaretDownIcon from "phosphor-svelte/lib/CaretDownIcon";
import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";
import FileCodeIcon from "phosphor-svelte/lib/FileCodeIcon";
import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
import PrinterIcon from "phosphor-svelte/lib/PrinterIcon";
import Icon from "$lib/components/Icon.svelte";
import { openedByPointer } from "../../../../stories/play";
import * as DropdownMenu from "./index";

// A menu under its trigger: plain rows that act, rows that tick, and rows of
// which one is chosen. The panel stays where it is written, so the trigger's
// wrapper places it.
const { Story } = defineMeta({
  title: "Kit/DropdownMenu",
  component: DropdownMenu.Root,
  tags: ["screen"],
});

const agents = ["Claude Code", "Codex", "Cursor"];
</script>

{#snippet actions(open: boolean)}
  <div class="anchor">
    <DropdownMenu.Root {open}>
      <DropdownMenu.Trigger class="btn"><Icon icon={DownloadSimpleIcon} size={15} />Export<Icon icon={CaretDownIcon} size={14} /></DropdownMenu.Trigger>
      <DropdownMenu.ContentStatic class="w-[210px]" aria-label="Export">
        <DropdownMenu.Item><Icon icon={PrinterIcon} size={15} />Export to PDF</DropdownMenu.Item>
        <DropdownMenu.Item><Icon icon={FileCodeIcon} size={15} />Export to HTML</DropdownMenu.Item>
        <DropdownMenu.Item disabled><Icon icon={FileTextIcon} size={15} />Export to Markdown</DropdownMenu.Item>
      </DropdownMenu.ContentStatic>
    </DropdownMenu.Root>
  </div>
{/snippet}

<Story name="Closed" asChild>{@render actions(false)}</Story>

<Story name="Open" play={openedByPointer} asChild>{@render actions(true)}</Story>

<Story name="Row under the keyboard" play={async ({ userEvent }) => { await userEvent.keyboard("{ArrowDown}{ArrowDown}"); }} asChild>
  {@render actions(true)}
</Story>

<Story name="Checkbox rows" play={openedByPointer} asChild>
  <div class="anchor">
    <DropdownMenu.Root open>
      <DropdownMenu.Trigger class="chip on">any agent<Icon icon={CaretDownIcon} size={13} /></DropdownMenu.Trigger>
      <DropdownMenu.ContentStatic class="w-[210px]" aria-label="any agent">
        {#each agents as agent, index (agent)}
          <DropdownMenu.CheckboxItem closeOnSelect={false} checked={index < 2}>{agent}</DropdownMenu.CheckboxItem>
        {/each}
      </DropdownMenu.ContentStatic>
    </DropdownMenu.Root>
  </div>
</Story>

<Story name="Radio rows" play={openedByPointer} asChild>
  <div class="anchor">
    <DropdownMenu.Root open>
      <DropdownMenu.Trigger class="chip">any time<Icon icon={CaretDownIcon} size={13} /></DropdownMenu.Trigger>
      <DropdownMenu.ContentStatic class="w-[210px]" aria-label="any time">
        <DropdownMenu.RadioGroup value="week">
          <DropdownMenu.RadioItem value="">any time</DropdownMenu.RadioItem>
          <DropdownMenu.RadioItem value="week">the last week</DropdownMenu.RadioItem>
          <DropdownMenu.RadioItem value="month">the last month</DropdownMenu.RadioItem>
        </DropdownMenu.RadioGroup>
      </DropdownMenu.ContentStatic>
    </DropdownMenu.Root>
  </div>
</Story>

<style>
  .anchor {
    position: relative;
    display: inline-block;
    min-height: 200px;
  }
</style>
