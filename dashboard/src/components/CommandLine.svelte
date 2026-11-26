<script lang="ts">
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import Icon from "$lib/components/Icon.svelte";
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

<div class="cmdline"><code>{text}</code><Tip text={copied ? "Copied" : "Copy"}>{#snippet children(tip)}<button {...tip} type="button" aria-label={copied ? "Copied" : "Copy the command"} onclick={copy}>{#if copied}<Icon icon={CheckIcon} size={16} />{:else}<Icon icon={CopyIcon} size={16} />{/if}</button>{/snippet}</Tip></div>
