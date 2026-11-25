<script lang="ts">
// Placeholder rows shown while a page's first read is in flight.
let { rows = 4 }: { rows?: number } = $props();
</script>

<div role="status" aria-label="Loading">
  {#each { length: rows } as _, index (index)}
    <!-- A title, a longer line and a short one, as a result row reads. -->
    <div class="skrow">
      <span class="sk" style:height="10px" style:width="{50 + ((index * 17) % 25)}%"></span>
      <span class="sk" style:height="8px" style:width="{70 + ((index * 11) % 20)}%"></span>
      <span class="sk" style:height="8px" style:width="{28 + ((index * 13) % 18)}%"></span>
    </div>
  {/each}
</div>
