// Reading one chat as the reader scrolls. The chat opens at its end, or
// around a linked message, and the stretches before and after it load as
// they come near the screen. At most ten stay loaded; the far ones are let
// go and read again on the way back. When Mash indexes more of the chat the
// last stretch is read again: text already shown never changes by itself.
// New messages are added at the end only while the reader is at the bottom
// of the chat, and are otherwise offered.
//
// This owns where the reader asked to be and what is loaded around it. It
// reads the page only to keep the reader's place; putting the reader at a
// message once it arrives stays with the view, which knows what else (a find
// match, a remembered position) is waiting on that.
import {
  type CreateInfiniteQueryResult,
  createInfiniteQuery,
  type InfiniteData,
  useQueryClient,
} from "@tanstack/svelte-query";
import { tick, untrack } from "svelte";
import {
  type ChatPlace,
  type ChatStretch,
  chatStretch,
  earlierPlace,
  laterPlace,
  type MessagePage,
  PAGE_SIZE,
} from "./api";
import { stableRead } from "./queries";
import { joinStretches, rereadPlace, rereadUpdate, withLastReplaced } from "./stretches";

/** The chat being read: its ID, the machine it is read from, and whether from the shared index. */
export interface ChatSource {
  readonly id: string;
  readonly machine: string;
  readonly shared: boolean;
}

/** What reading needs to know about the page that shows it. */
export interface ReadingView {
  /** The transcript's element, once it is on the page. */
  pane(): HTMLElement | undefined;
  /** Whether something else, such as the phone's session list, covers the transcript. */
  covered(): boolean;
}

/** Whether the reader is at, or within a short scroll of, the bottom of the page. */
function atBottom(): boolean {
  const page = document.documentElement;
  return window.innerHeight + window.scrollY >= page.scrollHeight - 120;
}

export function toBottom(): void {
  void tick().then(() =>
    window.scrollTo({ top: document.documentElement.scrollHeight, behavior: "instant" }),
  );
}

export class ChatReading {
  /** Where the reader asked to start reading; scrolling reads on from there. */
  place = $state<ChatPlace>({ at: "end" });
  /** Counts the reader's requests to go somewhere, so asking again goes again. */
  arrival = $state(0);
  /**
   * The arrival the reader has been put at. Until it matches `arrival`,
   * nothing loads around the screen, so a stretch arriving above the place
   * being shown cannot push it away.
   */
  placed = -1;
  /** Text already shown changed; the reader is offered the new version. */
  changed = $state(false);
  /** The last stretch read again with new messages, kept back while the reader is elsewhere. */
  held = $state<ChatStretch | undefined>();
  /** New messages not yet shown. */
  added = $state(0);
  checking = $state(false);

  readonly #source: () => ChatSource;
  readonly #view: ReadingView;
  readonly #client = useQueryClient();
  readonly key = $derived.by(() => {
    const { id, machine, shared } = this.#source();
    return ["transcript", id, machine, shared, JSON.stringify(this.place)];
  });
  readonly stretches: CreateInfiniteQueryResult<InfiniteData<ChatStretch, ChatPlace>, Error>;
  /** Everything loaded, read as one page. */
  readonly loaded: MessagePage | undefined;
  readonly total: number | undefined;
  readonly atStart: boolean;
  readonly atEnd: boolean;

  /** Must be made while a component is being set up, since it reads the query client from it. */
  constructor(source: () => ChatSource, start: ChatPlace, view: ReadingView) {
    this.#source = source;
    this.#view = view;
    this.place = start;
    this.stretches = createInfiniteQuery(() => {
      const { id, machine, shared } = this.#source();
      return {
        ...stableRead,
        queryKey: this.key,
        initialPageParam: this.place,
        queryFn: ({ pageParam, signal }: { pageParam: ChatPlace; signal: AbortSignal }) =>
          chatStretch(id, machine, pageParam, signal, shared),
        getPreviousPageParam: earlierPlace,
        getNextPageParam: laterPlace,
        maxPages: 10,
      };
    });
    // Derived here, once the query they read exists.
    this.loaded = $derived(joinStretches(this.stretches.data?.pages ?? []));
    this.total = $derived(this.loaded?.total);
    this.atStart = $derived(this.stretches.data?.pages[0]?.start === 0);
    this.atEnd = $derived(this.stretches.data !== undefined && !this.stretches.hasNextPage);
  }

  /** Starts reading a different chat, or the same one again, from `place`. */
  restart(place: ChatPlace): void {
    this.place = place;
    untrack(() => this.arrival++);
    this.changed = false;
    this.held = undefined;
    this.added = 0;
  }

  /** Read from `next`, or go there again when it is where the reader already started. */
  goTo(next: ChatPlace): void {
    this.place = next;
    this.arrival++;
  }

  toStart(): void {
    if (this.atStart) window.scrollTo({ top: 0, behavior: "instant" });
    else this.goTo({ at: "offset", offset: 0 });
  }

  toEnd(): void {
    if (this.atEnd) toBottom();
    else this.goTo({ at: "end" });
  }

  // Reading on. A stretch that loads above the screen pushes the text down, so
  // the message the reader is looking at is held in place; so is one when a
  // far stretch is let go.
  #onScreen(): { element: HTMLElement; top: number } | undefined {
    const element = Array.from(
      this.#view.pane()?.querySelectorAll<HTMLElement>("[data-message]") ?? [],
    ).find((element) => element.getBoundingClientRect().bottom > 90);
    return element ? { element, top: element.getBoundingClientRect().top } : undefined;
  }

  async #readOn(direction: "earlier" | "later"): Promise<void> {
    const before = this.#onScreen();
    await (direction === "earlier"
      ? this.stretches.fetchPreviousPage()
      : this.stretches.fetchNextPage());
    await tick();
    if (before?.element.isConnected)
      window.scrollBy({
        top: before.element.getBoundingClientRect().top - before.top,
        behavior: "instant",
      });
    this.fill();
  }

  /** Loads the next stretch on either side once it is within a screen or two. */
  fill(): void {
    const pane = this.#view.pane();
    if (this.placed !== this.arrival || this.#view.covered() || !pane) return;
    const stretches = this.stretches;
    if (stretches.isFetching || stretches.isFetchingPreviousPage || stretches.isFetchingNextPage)
      return;
    const reach = window.innerHeight * 1.5;
    const box = pane.getBoundingClientRect();
    if (stretches.hasPreviousPage && box.top > -reach) void this.#readOn("earlier");
    else if (stretches.hasNextPage && box.bottom < window.innerHeight + reach)
      void this.#readOn("later");
  }

  /** Replaces the last loaded stretch with the same stretch read again. */
  #keep(next: ChatStretch): void {
    this.#client.setQueryData<InfiniteData<ChatStretch, ChatPlace>>(this.key, (data) =>
      withLastReplaced(data, next),
    );
  }

  /**
   * Reads the end of what is loaded again, and with `everything`, the rest of
   * it too, as the Check for updates button does.
   */
  async checkForUpdates(everything = false): Promise<void> {
    const pages = this.stretches.data?.pages ?? [];
    const last = pages.at(-1);
    if (!last || this.checking) return;
    this.checking = true;
    const key = this.key;
    const { id, machine, shared } = this.#source();
    const again = (stretch: ChatStretch, limit: number) =>
      chatStretch(id, machine, rereadPlace(stretch, limit), new AbortController().signal, shared);
    try {
      const earlier = everything
        ? pages.slice(0, -1).filter((stretch) => stretch.end > stretch.start)
        : [];
      const [next, ...reread] = await Promise.all([
        // Reading past the old end picks up new messages along with it.
        again(last, Math.max(last.end - last.start, PAGE_SIZE)),
        ...earlier.map((stretch) => again(stretch, stretch.end - stretch.start)),
      ]);
      if (key !== this.key || !next) return;
      const update = rereadUpdate(earlier, reread, last, next);
      if (update.kind === "rewritten") this.changed = true;
      if (update.kind !== "appended") return;
      if (!this.#view.covered() && atBottom()) {
        this.#keep(next);
        if (!update.onPage) await this.stretches.fetchNextPage();
        toBottom();
      } else {
        // Off the bottom, only the length is kept, so text on screen stays put.
        if (!update.onPage) this.#keep(next);
        else this.held = next;
        this.added += update.added;
      }
    } catch {
      // The stream says again when the chat next changes.
    } finally {
      this.checking = false;
    }
  }

  /** Shows the messages that arrived while the reader was elsewhere in the chat. */
  showNew(): void {
    if (this.held) this.#keep(this.held);
    this.held = undefined;
    this.added = 0;
    this.toEnd();
  }

  showChanged(): void {
    this.changed = false;
    this.held = undefined;
    void this.stretches.refetch();
  }
}
