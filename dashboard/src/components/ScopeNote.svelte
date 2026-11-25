<script lang="ts">
// The viewing scope, named quietly at the end of a page's subtitle so a page
// never hides whose history it shows. What the scope includes is said in
// full where it is changed, in the viewing switcher's menu, and here when
// the pointer rests on the name. A page shown from a fallback says so.
import Globe from "@lucide/svelte/icons/globe";
import Laptop from "@lucide/svelte/icons/laptop";
import Radio from "@lucide/svelte/icons/radio";
import type { Scoped } from "../lib/queries";
import { scopeName, scopeSentence } from "../lib/scope.svelte";

let {
  scoped,
  fallback = undefined,
}: { scoped: Scoped | undefined; fallback?: string | undefined } = $props();
const why = $derived(fallback ?? scoped?.fallback);
</script>

{#if scoped}
  <span class="scope-n" data-scope={scoped.scope.kind} title={scopeSentence(scoped.scope, scoped.reach, scoped.live)}>
    {#if scoped.scope.kind === "all"}<Globe size={12} aria-hidden="true" />{:else if scoped.live}<Radio size={12} aria-hidden="true" />{:else}<Laptop size={12} aria-hidden="true" />{/if}
    <span>{scopeName(scoped.scope, { ...scoped.reach, live: scoped.live ? scoped.reach.live : [] })}</span>
  </span>
  {#if why}<span class="scope-why" role="status">{why}</span>{/if}
{/if}

<style>
  .scope-n {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-inline-start: 6px;
    color: var(--text-faint);
    font-size: 12.5px;
    white-space: nowrap;
  }
  .scope-n :global(svg) {
    flex: none;
  }
  .scope-why {
    display: block;
    margin-top: 4px;
    color: var(--status-warn-text);
    font-size: 12.5px;
  }
</style>
