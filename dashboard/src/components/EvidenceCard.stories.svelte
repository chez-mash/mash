<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { Evidence } from "../lib/records";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { chat } from "../stories/fixtures";
import EvidenceCard from "./EvidenceCard.svelte";

// A quoted message behind a memory. The reader's own words carry a butter
// edge, since they are what settled the memory.
const source: Evidence = {
  chatId: "sign-in-tokens",
  messageId: "sign-in-tokens-2",
  quote: "OK, go with the tokens. Keep the mobile app on the same scheme.",
  purpose: "endorsement",
  sourceRole: "user",
};
const agents: Evidence = {
  ...source,
  quote:
    "Fifteen-minute access tokens with a server-held refresh token mean signing out reaches every device within fifteen minutes.",
  purpose: "context",
  sourceRole: "assistant",
};
const from = chat("sign-in-tokens");

const { Story } = defineMeta({
  title: "Blocks/EvidenceCard",
  component: EvidenceCard,
  args: { source, node: "laptop", chat: from },
});
</script>

<Story name="Default" />

<Story name="States" asChild>
  <Cases width={640}>
    <Case label="the reader's words"><EvidenceCard {source} node="laptop" chat={from} /></Case>
    <Case label="the agent's words"><EvidenceCard source={agents} node="laptop" chat={from} /></Case>
    <Case label="chat not indexed here"><EvidenceCard {source} node="laptop" /></Case>
    <Case label="no purpose recorded"><EvidenceCard source={{ ...agents, purpose: null }} node="laptop" chat={from} /></Case>
  </Cases>
</Story>

<Story name="Brief" asChild>
  <Cases width={340}>
    <Case label="the reader's words"><EvidenceCard {source} node="laptop" chat={from} brief /></Case>
    <Case label="the agent's words"><EvidenceCard source={agents} node="laptop" chat={from} brief /></Case>
  </Cases>
</Story>
