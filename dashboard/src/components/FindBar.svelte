<script lang="ts">
// The Find in chat box. It only takes the words and says how many matches
// there are; finding them, and going to one, belong to the transcript.
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronUp from "@lucide/svelte/icons/chevron-up";
import Search from "@lucide/svelte/icons/search";
import X from "@lucide/svelte/icons/x";

let {
  query = $bindable(),
  input = $bindable(),
  status,
  matches,
  onstep,
  onclose,
}: {
  query: string;
  input?: HTMLInputElement | undefined;
  status: string;
  /** How many matches there are; going between them needs one. */
  matches: number;
  onstep: (step: 1 | -1) => void;
  onclose: () => void;
} = $props();
</script>

<div class="findbar" role="search">
  <Search size={16} aria-hidden="true" />
  <label class="sr-only" for="find-in-chat">Find in this chat</label>
  <input
    id="find-in-chat"
    type="text"
    enterkeyhint="search"
    placeholder="Find in this chat"
    autocomplete="off"
    bind:this={input}
    bind:value={query}
    onkeydown={(event) => {
      if (event.key === "Enter") {
        event.preventDefault();
        onstep(event.shiftKey ? -1 : 1);
      }
    }}
  />
  <span class="findn" role="status">{status}</span>
  <button class="btn sm icon" type="button" aria-label="Previous match" title="Previous match" disabled={!matches} onclick={() => onstep(-1)}><ChevronUp size={15} /></button>
  <button class="btn sm icon" type="button" aria-label="Next match" title="Next match" disabled={!matches} onclick={() => onstep(1)}><ChevronDown size={15} /></button>
  <button class="btn sm icon ghost" type="button" aria-label="Close find" title="Close find" onclick={onclose}><X size={15} /></button>
</div>
