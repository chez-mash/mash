<script lang="ts">
import ArrowLineDownIcon from "phosphor-svelte/lib/ArrowLineDownIcon";
import ArrowLineUpIcon from "phosphor-svelte/lib/ArrowLineUpIcon";
import Icon from "$lib/components/Icon.svelte";

// Scrolling reads the rest of the chat; these float beside it and jump to either end.
let { onstart, onend }: { onstart: () => void; onend: () => void } = $props();
</script>

<nav aria-label="Chat ends" class="ends">
  <button class="btn icon" type="button" onclick={onstart} aria-label="First message" title="First message"><Icon icon={ArrowLineUpIcon} size={15} /></button>
  <button class="btn icon" type="button" onclick={onend} aria-label="Latest message" title="Latest message"><Icon icon={ArrowLineDownIcon} size={15} /></button>
</nav>
