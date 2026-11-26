<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import InfoIcon from "phosphor-svelte/lib/InfoIcon";
import Icon from "$lib/components/Icon.svelte";
import { hint } from "../../../../stories/play";
import { Tip } from "./index";

// A hint for one control or icon, shown on hover and on keyboard focus.
const { Story } = defineMeta({ title: "Kit/Tooltip", component: Tip, tags: ["screen"] });
</script>

{#snippet control(side: "top" | "right" | "bottom" | "left")}
  <div class="stage">
    <Tip text="Copy the command" {side}>{#snippet children(tip)}<button {...tip} type="button" class="btn icon" aria-label="Copy the command"><Icon icon={CopyIcon} size={15} /></button>{/snippet}</Tip>
  </div>
{/snippet}

<Story name="Rest" asChild>{@render control("top")}</Story>

<Story name="Open" play={async ({ canvas }) => hint(canvas.getByRole("button"), "Copy the command")} asChild>
  {@render control("top")}
</Story>

<Story name="Open below" play={async ({ canvas }) => hint(canvas.getByRole("button"), "Copy the command")} asChild>
  {@render control("bottom")}
</Story>

<Story name="Passive" asChild>
  <div class="stage">
    <!-- Not a control: the hint shows on hover, and the icon is no tab stop. -->
    <Tip text="Read from the machine's own dashboard" passive>{#snippet children(tip)}<span {...tip} class="note"><Icon icon={InfoIcon} size={14} />live</span>{/snippet}</Tip>
  </div>
</Story>

<style>
  .stage {
    display: grid;
    place-items: center;
    min-height: 140px;
  }
  .note {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12.5px;
    color: var(--text-muted);
  }
</style>
