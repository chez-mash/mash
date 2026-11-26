<script lang="ts">
// A passage the reader picked but has not asked Mashy about yet. It waits at
// the end of the conversation, where the next question goes, with the
// questions to ask about it.
import ArrowElbowDownRightIcon from "phosphor-svelte/lib/ArrowElbowDownRightIcon";
import XIcon from "phosphor-svelte/lib/XIcon";
import Icon from "$lib/components/Icon.svelte";
import { Tip } from "$lib/components/ui/tooltip";
import { clipPassage, mashy, PASSAGE_QUESTIONS } from "../lib/mashy.svelte";

let { ask }: { ask: (question: string) => void } = $props();
</script>

{#if mashy.pendingPassage}
  <div class="mashy-passage">
    <span>About this passage</span>
    <p>“{clipPassage(mashy.pendingPassage)}”</p>
    <Tip text="Remove">{#snippet children(tip)}<button {...tip} type="button" aria-label="Stop asking about this passage" onclick={() => { mashy.passage = ""; }}><Icon icon={XIcon} size={14} /></button>{/snippet}</Tip>
  </div>
  {#if !mashy.busy}
    <div class="mashy-sugg next">
      {#each PASSAGE_QUESTIONS as question (question)}<button type="button" onclick={() => ask(question)}><Icon icon={ArrowElbowDownRightIcon} size={13} />{question}</button>{/each}
    </div>
  {/if}
{/if}
