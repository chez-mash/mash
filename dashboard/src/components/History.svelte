<script lang="ts">
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import { createQuery } from "@tanstack/svelte-query";
import { catalogOptions } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import ChatRow from "./ChatRow.svelte";
import EmptyState from "./EmptyState.svelte";

// Chats opened since the page was loaded, most recent first. Positions live
// in the page, so they are gone after a reload.
const catalogue = createQuery(() => catalogOptions);
const chats = $derived(
  reading.positions.flatMap(
    (position) =>
      catalogue.data?.chats.find(
        (chat) => chat.id === position.id && chat.machine === position.node,
      ) ?? [],
  ),
);
</script>

<div><h1 class="h1">Viewing history</h1><p class="meta" style:margin-top="6px">Chats you opened since this page was loaded. Each reopens where you left it.</p></div>
<section>
  {#if chats.length}
    <div class="card pad0 list">
      {#each chats as chat (chat.machine + chat.id)}<ChatRow {chat} />{/each}
    </div>
  {:else}
    <EmptyState card>
      Nothing yet. Open a chat and it will be listed here until you reload the page.
      {#snippet action()}<a class="btn" href="#/search"><MessagesSquare size={15} aria-hidden="true" />Browse chats</a>{/snippet}
    </EmptyState>
  {/if}
</section>
