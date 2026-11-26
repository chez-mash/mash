<script lang="ts">
// The Find in chat box. It only takes the words and says how many matches
// there are; finding them, and going to one, belong to the transcript.
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronUp from "@lucide/svelte/icons/chevron-up";
import Search from "@lucide/svelte/icons/search";
import X from "@lucide/svelte/icons/x";
import { Tip } from "$lib/components/ui/tooltip";

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
  <div class="fb-field">
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
    <Tip text="Previous match">{#snippet children(tip)}<button {...tip} class="fb-b" type="button" aria-label="Previous match" disabled={!matches} onclick={() => onstep(-1)}><ChevronUp size={16} /></button>{/snippet}</Tip>
    <Tip text="Next match">{#snippet children(tip)}<button {...tip} class="fb-b" type="button" aria-label="Next match" disabled={!matches} onclick={() => onstep(1)}><ChevronDown size={16} /></button>{/snippet}</Tip>
    <span class="fb-sep" aria-hidden="true"></span>
    <Tip text="Close find">{#snippet children(tip)}<button {...tip} class="fb-b" type="button" aria-label="Close find" onclick={onclose}><X size={16} /></button>{/snippet}</Tip>
  </div>
  <span class="fb-k" aria-hidden="true"><kbd class="kbd">↵</kbd>next<kbd class="kbd">⇧</kbd><kbd class="kbd">↵</kbd>previous<kbd class="kbd">esc</kbd>close</span>
</div>

<style>
  /* Under the chat's header: the find field, raised and ringed in butter
     while it has focus, with the keys that move between matches beside it. */
  .findbar {
    position: sticky;
    top: var(--top-h, 52px);
    z-index: 6;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 32px;
    border: 0;
    border-bottom: var(--border-divider);
    border-radius: 0;
    background: var(--surface-page);
  }
  .fb-field {
    flex: 0 1 460px;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 42px;
    padding: 0 6px 0 14px;
    border-radius: var(--radius-6);
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-strong);
    color: var(--text-muted);
  }
  .fb-field:focus-within {
    box-shadow:
      0 0 0 var(--stroke-outline) var(--chip-on-line),
      0 0 0 4px var(--chip-on-bg);
  }
  .fb-field input {
    flex: 1;
    min-width: 0;
    min-height: 36px;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--text-strong);
    font-size: 14px;
  }
  .findn {
    flex: none;
    font: 12px var(--type-mono);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .fb-b {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-4);
    color: var(--text-muted);
  }
  .fb-b:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .fb-b:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .fb-sep {
    flex: none;
    width: 1px;
    height: 20px;
    margin: 0 2px;
    background: var(--line-outline);
  }
  .fb-k {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .fb-k .kbd:not(:first-child) {
    margin-left: 6px;
  }
  .fb-k .kbd + .kbd {
    margin-left: 0;
  }
  @media (max-width: 900px) {
    .fb-k {
      display: none;
    }
  }
  @media (max-width: 700px) {
    .findbar {
      padding: 10px 16px;
    }
    .fb-field {
      flex: 1;
    }
  }
</style>
