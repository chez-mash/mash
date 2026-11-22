<script lang="ts">
import House from "@lucide/svelte/icons/house";
import EmptyState from "./EmptyState.svelte";
</script>

<EmptyState card title="Page not found" heading="h1">
  This link does not match a page in this dashboard.
  {#snippet action()}<a class="btn p" href="#/"><House size={15} aria-hidden="true" />Go to overview</a>{/snippet}
</EmptyState>
