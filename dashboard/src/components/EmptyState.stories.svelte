<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import EmptyState from "./EmptyState.svelte";

// Nothing to show, or nowhere to go: Mashy, a heading, one plain sentence and
// the one thing to do next.
const { Story } = defineMeta({ title: "Blocks/EmptyState", component: EmptyState });
</script>

<Story name="Default" asChild>
  <EmptyState title="No chats yet">Mash has not indexed any chats on this machine.</EmptyState>
</Story>

<Story name="Forms" asChild>
  <Cases width={640}>
    <Case label="a sentence"><EmptyState>No chat matches these filters.</EmptyState></Case>
    <Case label="in a card"><EmptyState card>No project matches. Change the name, agent or machine above.</EmptyState></Case>
    <Case label="with a title"><EmptyState card title="Nothing viewed yet">Chats you open are listed here, so you can pick up where you left off.</EmptyState></Case>
    <Case label="with the next thing to do">
      <EmptyState card title="Page not found" heading="h1">
        That address leads nowhere in this dashboard.
        {#snippet action()}<a class="btn p" href="#/">Go to the overview</a>{/snippet}
      </EmptyState>
    </Case>
  </Cases>
</Story>
