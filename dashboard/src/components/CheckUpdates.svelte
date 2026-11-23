<script lang="ts">
import RefreshCw from "@lucide/svelte/icons/refresh-cw";

// The one way a page offers to read its data again: a small quiet button
// with the same words everywhere. Actions that do something else, such as
// retrying a failed load, keep their own words.
let {
  checking,
  disabled,
  check,
}: {
  /** A check is in progress; the button says so and waits. */
  checking: boolean;
  /** Unavailable for another reason, for instance before the first answer. */
  disabled?: boolean;
  check: () => unknown;
} = $props();
</script>

<button class="btn ghost sm" type="button" disabled={disabled ?? checking} onclick={() => check()}><RefreshCw size={14} aria-hidden="true" />{checking ? "Checking…" : "Check for updates"}</button>
