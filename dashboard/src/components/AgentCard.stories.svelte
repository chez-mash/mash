<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { now } from "../stories/fixtures";
import AgentCard from "./AgentCard.svelte";

const HOUR = 3_600_000;
const indexed = { indexed: true, roots: ["/home/sam/.codex/sessions"] };
const base = {
  agent: "codex",
  chats: 9,
  total: 21,
  latestMs: now - 3 * HOUR,
  machines: ["laptop", "workstation"],
  setting: indexed,
  now,
} as const;

const { Story } = defineMeta({
  title: "Blocks/AgentCard",
  component: AgentCard,
  args: base,
});
</script>

<Story name="Indexed" />

<Story name="Not indexed" args={{ setting: { indexed: false, roots: ["/home/sam/.codex/sessions"] } }} />

<Story name="States" asChild>
  <Cases row>
    <Case label="indexed"><div class="cell"><AgentCard {...base} /></div></Case>
    <Case label="not indexed here"><div class="cell"><AgentCard {...base} setting={{ indexed: false, roots: indexed.roots }} /></div></Case>
    <Case label="with its switch, on"><div class="cell"><AgentCard {...base} onindex={() => {}} /></div></Case>
    <Case label="with its switch, off"><div class="cell"><AgentCard {...base} setting={{ indexed: false, roots: indexed.roots }} onindex={() => {}} /></div></Case>
    <Case label="switch busy"><div class="cell"><AgentCard {...base} onindex={() => {}} busy /></div></Case>
    <Case label="no chats yet"><div class="cell"><AgentCard {...base} agent="cursor" chats={0} latestMs={undefined} machines={[]} setting={{ indexed: true, roots: [] }} /></div></Case>
    <Case label="settings not readable"><div class="cell"><AgentCard {...base} agent="claude-code" setting={undefined} /></div></Case>
  </Cases>
</Story>

<style>
  .cell {
    width: 280px;
  }
</style>
