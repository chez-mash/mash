<script lang="ts">
// A passage the reader picked but has not asked Mashy about yet. It waits at
// the end of the conversation, where the next question goes, with the
// questions to ask about it.
import CornerDownRight from "@lucide/svelte/icons/corner-down-right";
import X from "@lucide/svelte/icons/x";
import { clipPassage, mashy, PASSAGE_QUESTIONS } from "../lib/mashy.svelte";

let { ask }: { ask: (question: string) => void } = $props();
</script>

{#if mashy.pendingPassage}
  <div class="mashy-passage">
    <span>About this passage</span>
    <p>“{clipPassage(mashy.pendingPassage)}”</p>
    <button type="button" aria-label="Stop asking about this passage" title="Remove" onclick={() => { mashy.passage = ""; }}><X size={14} /></button>
  </div>
  {#if !mashy.busy}
    <div class="mashy-sugg next">
      {#each PASSAGE_QUESTIONS as question (question)}<button type="button" onclick={() => ask(question)}><CornerDownRight size={13} aria-hidden="true" />{question}</button>{/each}
    </div>
  {/if}
{/if}
