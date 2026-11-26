<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { agents } from "../lib/agents";
import type { Agent } from "../lib/types";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import AgentMark from "./AgentMark.svelte";

const { Story } = defineMeta({
  title: "Blocks/AgentMark",
  component: AgentMark,
  args: { agent: "claude-code" },
});

const all = Object.keys(agents) as Agent[];
const sizes = ["sm", "md", "lg"] as const;
</script>

<Story name="Default" />

<Story name="Agents and sizes" asChild>
  <Cases row>
    {#each all as agent (agent)}
      <Case label={agents[agent].name}>
        <span class="set">{#each sizes as size (size)}<AgentMark {agent} {size} />{/each}</span>
      </Case>
    {/each}
  </Cases>
</Story>

<style>
  .set {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
</style>
