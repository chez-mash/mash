<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Mashy from "../../components/Mashy.svelte";
import { mashy } from "../../lib/mashy.svelte";
import type { StoryData } from "../answers";

// Mashy's circle in the corner and the window it opens. The conversation is
// kept outside the components (lib/mashy.svelte.ts), so each story sets it.
const { Story } = defineMeta({
  title: "Composed/Mashy",
  component: Mashy,
  parameters: { layout: "fullscreen" },
  tags: ["screen"],
});

const passage =
  "Sign-in uses fifteen-minute access tokens with a server-held refresh token. The mobile app follows the same scheme.";
const model = { "/api/assistant": { enabled: true, model: "story-model" } } as const;

/** Starts every story from a closed, empty conversation in its own corner. */
function conversation(set: () => void = () => {}): () => void {
  return () => {
    try {
      localStorage.removeItem("mashy.place");
    } catch {
      // Without storage the window is in its corner already.
    }
    mashy.clear();
    mashy.open = false;
    mashy.unread = false;
    mashy.answer = "";
    mashy.phase = "idle";
    set();
  };
}
const story = (data: StoryData): { data: StoryData } => ({ data });
const phone = { viewport: { value: "phone" } };

const opened = conversation(() => {
  mashy.open = true;
});
const answering = conversation(() => {
  mashy.open = true;
  mashy.passage = passage;
  mashy.turns = [{ role: "user", content: "Was this changed later?", passage }];
  mashy.answer =
    "Not since it was saved. The decision was made in **Move sign-in to short-lived tokens** and the mobile app";
  mashy.phase = "answering";
});
const answered = conversation(() => {
  mashy.open = true;
  mashy.turns = [
    { role: "user", content: "What have I decided recently?" },
    {
      role: "assistant",
      content:
        "Two decisions in the last week:\n\n- Sign-in uses fifteen-minute access tokens with a server-held refresh token.\n- Invoice exports run as background jobs kept in a Postgres table.",
    },
  ];
});
</script>

{#snippet host()}
  <p class="page meta">The page behind Mashy.</p>
  <div class="mashy-host"><Mashy /></div>
{/snippet}

<Story name="Button" parameters={story({ before: conversation() })} asChild>{@render host()}</Story>

<Story name="Button with a new answer" parameters={story({ before: conversation(() => { mashy.unread = true; }) })} asChild>
  {@render host()}
</Story>

<!-- The demo has no model, which is what a dashboard without one shows. -->
<Story name="Panel, unset" parameters={story({ before: opened })} asChild>{@render host()}</Story>

<Story name="Panel, empty" parameters={story({ answers: model, before: opened })} asChild>{@render host()}</Story>

<Story name="Panel, a passage picked" parameters={story({ answers: model, before: conversation(() => { mashy.about(passage); }) })} asChild>
  {@render host()}
</Story>

<Story name="Panel, answering about a passage" parameters={story({ answers: model, before: answering })} asChild>{@render host()}</Story>

<Story name="Panel, answered" parameters={story({ answers: model, before: answered })} asChild>{@render host()}</Story>

<Story name="Panel, could not answer" parameters={story({ answers: model, before: conversation(() => { mashy.open = true; mashy.turns = [{ role: "user", content: "What did I work on yesterday?" }]; mashy.note = { kind: "error", title: "Couldn’t reach the model", text: "The model’s service did not answer within 30 seconds.", retry: true }; }) })} asChild>
  {@render host()}
</Story>

<Story name="Phone sheet" tags={["phone"]} globals={phone} parameters={story({ answers: model, before: answered })} asChild>{@render host()}</Story>

<style>
  .page {
    margin: 0;
    padding: 24px;
  }
</style>
