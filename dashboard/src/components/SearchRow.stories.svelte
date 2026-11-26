<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { chat, subagentChat } from "../stories/fixtures";
import SearchRow from "./SearchRow.svelte";

const found = chat("sign-in-tokens");
const passage =
  "Move sign-in to short-lived tokens: fifteen-minute access tokens with a refresh token the server holds, so signing out reaches every device.";

const { Story } = defineMeta({
  title: "Blocks/SearchRow",
  component: SearchRow,
  args: { chat: found, href: "#/chat/sign-in-tokens", query: "tokens", snippet: passage },
});
</script>

<Story name="Default" />

<Story name="States" asChild>
  <Cases width={640}>
    <Case label="found by its words"><SearchRow chat={found} href="#/chat/sign-in-tokens" query="tokens" snippet={passage} /></Case>
    <Case label="current" note="the row the preview shows"><SearchRow chat={found} href="#/chat/sign-in-tokens" query="tokens" snippet={passage} selected /></Case>
    <Case label="found by meaning"><SearchRow chat={found} href="#/chat/sign-in-tokens" query="logging out everywhere" snippet={passage} byMeaning /></Case>
    <Case label="with decisions"><SearchRow chat={found} href="#/chat/sign-in-tokens" query="tokens" snippet={passage} decisions={2} /></Case>
    <Case label="a subagent"><SearchRow chat={subagentChat} href="#/chat/{subagentChat.id}" query="release" snippet="Checked the release notes against the merged pull requests." parentTitle="Ship the 0.2.0 release" /></Case>
    <Case label="no passage" note="listed, not searched"><SearchRow chat={found} href="#/chat/sign-in-tokens" /></Case>
  </Cases>
</Story>

<Story name="On a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} args={{ decisions: 1 }} />
