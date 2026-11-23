<script lang="ts">
// The viewing scope, said under a page's title so a page never hides whose
// history it shows. It is changed in the corner of the sidebar.
import Globe from "@lucide/svelte/icons/globe";
import Laptop from "@lucide/svelte/icons/laptop";
import Radio from "@lucide/svelte/icons/radio";
import type { Scoped } from "../lib/queries";
import { scopeName, scopeSentence } from "../lib/scope.svelte";

let {
  scoped,
  fallback = undefined,
}: { scoped: Scoped | undefined; fallback?: string | undefined } = $props();
const machines = $derived(new Set(scoped?.all.map((chat) => chat.machine) ?? []).size);
const why = $derived(fallback ?? scoped?.fallback);
</script>

{#if scoped}
  <p class="scope-n" data-scope={scoped.scope.kind}>
    <span class="pill plain">
      {#if scoped.scope.kind === "all"}<Globe size={13} aria-hidden="true" />{:else if scoped.live}<Radio size={13} aria-hidden="true" />{:else}<Laptop size={13} aria-hidden="true" />{/if}
      {scopeName(scoped.scope, { ...scoped.reach, live: scoped.live ? scoped.reach.live : [] }, machines)}
    </span>
    <span class="meta">{scopeSentence(scoped.scope, scoped.reach, scoped.live)}</span>
  </p>
  {#if why}<p class="card tight warn meta" role="status">{why}</p>{/if}
{/if}

<style>
  .scope-n {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 10px;
    margin: 8px 0 0;
  }
  .scope-n .meta {
    margin: 0;
  }
  /* A phone keeps the pill; the top bar names the scope too, and the
     sentence would push the page's content below the fold. */
  @media (max-width: 800px) {
    .scope-n .meta {
      display: none;
    }
  }
</style>
