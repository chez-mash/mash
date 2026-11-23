import type { QueryClient, QueryKey } from "@tanstack/svelte-query";

// The dashboard has one "Check for updates" control, at the foot of the rail.
// It reads again everything the current screen shows: the chat catalogue and
// every query the page has on screen. A page whose data needs a gentler
// reread than a plain refetch, such as an open chat that must keep the
// reader's place, offers its own way here and names the queries it covers.

/** A page's own way of reading its data again. */
export interface PageCheck {
  /** Reads the page's data again. */
  check: () => Promise<unknown>;
  /** The queries `check` already covers, which the general reread leaves alone. */
  owns: (key: QueryKey) => boolean;
}

class Updates {
  #page = $state<PageCheck | undefined>();
  #running = $state(false);

  /** A check is in progress. */
  get checking(): boolean {
    return this.#running;
  }

  /** Lets the page being set up offer its own check while it is on screen. */
  offer(page: PageCheck): void {
    $effect(() => {
      this.#page = page;
      return () => {
        if (this.#page === page) this.#page = undefined;
      };
    });
  }

  /** Reads again what the screen shows. */
  async check(client: QueryClient): Promise<void> {
    if (this.#running) return;
    this.#running = true;
    const page = this.#page;
    try {
      await Promise.all([
        // Only queries on screen are read again now; the ones kept for other
        // screens are just marked out of date.
        client.invalidateQueries(
          { predicate: (query) => !page?.owns(query.queryKey) },
          { cancelRefetch: false },
        ),
        page?.check(),
      ]);
    } catch {
      // Each page shows its own load errors.
    } finally {
      this.#running = false;
    }
  }
}

export const updates = new Updates();
