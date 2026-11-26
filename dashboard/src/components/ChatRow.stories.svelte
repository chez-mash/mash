<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { chat, parentChat, subagentChat } from "../stories/fixtures";
import ChatRow from "./ChatRow.svelte";

// One chat in a card list. The list draws the card; the row is the link.
const { Story } = defineMeta({ title: "Blocks/ChatRow", component: ChatRow });

const plain = chat("sign-in-tokens");
</script>

<Story name="Default" asChild>
  <div class="card pad0 list col"><ChatRow chat={plain} /></div>
</Story>

<Story name="States" asChild>
  <Cases width={720}>
    <Case label="plain"><div class="card pad0 list"><ChatRow chat={plain} /></div></Case>
    <Case label="with decisions"><div class="card pad0 list"><ChatRow chat={plain} decisions={2} /></div></Case>
    <Case label="started subagents"><div class="card pad0 list"><ChatRow chat={parentChat} /></div></Case>
    <Case label="a subagent"><div class="card pad0 list"><ChatRow chat={subagentChat} /></div></Case>
    <Case label="ran over SSH"><div class="card pad0 list"><ChatRow chat={{ ...plain, sshCopy: true }} /></div></Case>
    <Case label="with a line of its text">
      <div class="card pad0 list">
        <ChatRow chat={plain}>
          {#snippet snippet()}OK, go with the tokens. Keep the mobile app on the same scheme.{/snippet}
        </ChatRow>
      </div>
    </Case>
    <Case label="a list">
      <div class="card pad0 list"><ChatRow chat={plain} decisions={1} /><ChatRow chat={parentChat} /><ChatRow chat={subagentChat} /></div>
    </Case>
  </Cases>
</Story>

<Story name="On a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} asChild>
  <div class="card pad0 list"><ChatRow chat={plain} decisions={2} /><ChatRow chat={parentChat} /></div>
</Story>

<style>
  .col {
    max-width: 720px;
  }
</style>
