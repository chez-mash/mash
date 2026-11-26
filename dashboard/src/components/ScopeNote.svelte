<script lang="ts">
// The viewing scope, named quietly at the end of a page's subtitle so a page
// never hides whose history it shows. What the scope includes is said in
// full where it is changed, in the viewing switcher's menu, and here when
// the pointer rests on the name. A page shown from a fallback says so.
import BroadcastIcon from "phosphor-svelte/lib/BroadcastIcon";
import GlobeIcon from "phosphor-svelte/lib/GlobeIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import Icon from "$lib/components/Icon.svelte";
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
    {#if scoped.scope.kind === "all"}<Icon icon={GlobeIcon} size={12} />{:else if scoped.live}<Icon icon={BroadcastIcon} size={12} />{:else}<Icon icon={LaptopIcon} size={12} />{/if}
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
