<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import ChatPlace from "./ChatPlace.svelte";

const { Story } = defineMeta({
  title: "Blocks/ChatPlace",
  component: ChatPlace,
  args: { agent: "claude-code", machine: "laptop", project: "/home/sam/code/storefront" },
});
</script>

<Story name="Default" />

<Story name="Parts" asChild>
  <Cases>
    <Case label="agent, machine and project"><ChatPlace agent="codex" machine="laptop" project="/home/sam/code/storefront" /></Case>
    <Case label="led" note="the row already leads with the agent's tile"><ChatPlace agent="codex" machine="laptop" project="/home/sam/code/storefront" led /></Case>
    <Case label="machine only"><ChatPlace machine="workstation" /></Case>
    <Case label="no project"><ChatPlace agent="cursor" machine="laptop" /></Case>
    <Case label="narrow" note="the names wrap, each with its mark">
      <div class="narrow"><ChatPlace agent="claude-code" machine="workstation" project="/home/sam/code/storefront-mobile" /></div>
    </Case>
  </Cases>
</Story>

<style>
  .narrow {
    width: 190px;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
