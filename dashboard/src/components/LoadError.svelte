<script lang="ts">
import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
import WarningIcon from "phosphor-svelte/lib/WarningIcon";
import Icon from "$lib/components/Icon.svelte";

let {
  title,
  message,
  retry,
  busy = false,
  retryLabel = "Retry",
}: {
  title: string;
  message: string;
  retry: () => unknown;
  busy?: boolean;
  retryLabel?: string;
} = $props();
const detail = $derived(
  ["Load failed", "Failed to fetch", "NetworkError when attempting to fetch resource."].includes(
    message,
  )
    ? "Could not reach Mash. Check your connection and that the dashboard server is running, then retry."
    : message,
);
</script>

<!-- One line across the space the content would fill: the warning, what
     failed over why, and the retry at the end. -->
<div class="card bad loaderr" role="alert" aria-busy={busy}>
  <Icon icon={WarningIcon} class="loaderr-mark" size={18} state="strong" />
  <div>
    <strong>{title}</strong>
    <p>{detail}</p>
  </div>
  <button type="button" class="btn sm" onclick={retry} disabled={busy}>
    <Icon icon={ArrowClockwiseIcon} size={14} />{busy ? "Trying again…" : retryLabel}
  </button>
</div>
