<script lang="ts">
import Check from "@lucide/svelte/icons/check";
import Copy from "@lucide/svelte/icons/copy";
import { Tip } from "$lib/components/ui/tooltip";

// A terminal command on one line with a button that copies it. Without
// clipboard access the command stays selectable on the page.
let { text }: { text: string } = $props();
let copied = $state(false);
async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  } catch {
    copied = false;
  }
}
</script>

<div class="cmdline"><code>{text}</code><Tip text={copied ? "Copied" : "Copy"}>{#snippet children(tip)}<button {...tip} type="button" aria-label={copied ? "Copied" : "Copy the command"} onclick={copy}>{#if copied}<Check size={16} aria-hidden="true" />{:else}<Copy size={16} aria-hidden="true" />{/if}</button>{/snippet}</Tip></div>
