<script lang="ts">
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import Icon from "$lib/components/Icon.svelte";
import { Tip } from "$lib/components/ui/tooltip";

// A command to run elsewhere, in a muted box that wraps it, with a button to
// copy it. Long commands wrap instead of scrolling, so a narrow column shows
// all of them.
let { text }: { text: string } = $props();
let copied = $state(false);

async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    copied = true;
  } catch {
    copied = false;
  }
}
</script>

<div class="cc"><code>{text}</code><Tip text={copied ? "Copied" : "Copy"}>{#snippet children(tip)}<button {...tip} type="button" aria-label={copied ? "Copied" : `Copy ${text}`} onclick={copy}>{#if copied}<Icon icon={CheckIcon} size={14} />{:else}<Icon icon={CopyIcon} size={14} />{/if}</button>{/snippet}</Tip></div>

<style>
  .cc {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 11px 10px 11px 12px;
    border-radius: var(--radius-5);
    background: var(--surface-muted);
  }
  code {
    flex: 1;
    min-width: 0;
    font-family: var(--type-mono);
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--text-strong);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  button {
    flex: none;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-3);
    color: var(--text-muted);
  }
  button:hover {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
</style>
