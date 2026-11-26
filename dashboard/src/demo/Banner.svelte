<script lang="ts">
import { chatHref, recordHref, router, searchHref } from "../lib/route.svelte";
import { localMachine } from "./history";
</script>

<aside class="demo" aria-label="About this demo">
  <span>Demo with sample chats</span>
  <nav aria-label="Demo examples">
    <a href={searchHref({ q: "invoice exports queue" })}>Try a search</a>
    {#if router.current.page === "chat" && router.current.id === "sign-in-tokens"}
      <a href={chatHref("mobile-keychain-follow-up", "workstation", "mobile-keychain-follow-up-1", true)}>See the Codex follow-up</a>
    {:else}
      <a href={recordHref("record-sign-in-tokens", localMachine)}>See a decision</a>
    {/if}
    <a href="https://getmash.io/quick-start/">Install Mash</a>
  </nav>
</aside>

<style>
  .demo {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 0 18px;
    padding: 4px 14px;
    border-bottom: var(--border-divider);
    background: var(--banner-bg);
    color: var(--banner-text);
    font-family: var(--type-sans);
    font-size: 14px;
    text-align: center;
  }
  .demo nav {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0 12px;
  }
  .demo a {
    display: inline-flex;
    align-items: center;
    min-height: 44px;
    color: inherit;
    font-weight: var(--weight-strong);
    text-decoration: underline;
    text-underline-offset: 3px;
  }
</style>
