<script lang="ts">
import RefreshCw from "@lucide/svelte/icons/refresh-cw";

// The dashboard's one way to read what is on screen again, shown at the foot
// of the rail (see lib/updates.svelte.ts). Pages do not add their own copy;
// actions that do something else, such as retrying a failed load, keep their
// own words.
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
