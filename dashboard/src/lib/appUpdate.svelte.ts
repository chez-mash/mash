/**
 * The desktop app's update banner.
 *
 * The Mash desktop app knows whether a newer version exists and how this
 * copy gets it; the dashboard only draws that in its main window and passes
 * on the two choices, Update and Later. The app answers three commands for
 * it (`update_banner`, `update_install`, `update_later`; see
 * desktop/src-tauri/src/update/banner.rs). A browser has no app to ask, so
 * there the banner never shows and nothing is read.
 */
import { inDesktopApp } from "./shareLink";

/** How the person gets the version on offer. */
type UpdateWay = "install" | "download" | "apt";
type UpdateStage = "available" | "downloading" | "installing" | "failed";

/** What the app has to offer, as it says it. */
export interface UpdateOffer {
  /** The running version, and the one on offer. */
  readonly current: string;
  readonly version: string;
  readonly way: UpdateWay;
  readonly stage: UpdateStage;
  /** The part of the download that has arrived, from 0 to 1, when known. */
  readonly progress: number | null;
  /** Why the update did not install. */
  readonly reason: string | null;
  /** The command that updates a copy apt installed. */
  readonly command: string | null;
  /** Where a person downloads Mash by hand. */
  readonly downloads: string;
}

const ways: readonly string[] = ["install", "download", "apt"] satisfies UpdateWay[];
const stages: readonly string[] = [
  "available",
  "downloading",
  "installing",
  "failed",
] satisfies UpdateStage[];

const text = (value: unknown): string | null => (typeof value === "string" && value ? value : null);

/** The app's answer as an offer, or null when it has none or says something else. */
export function readOffer(answer: unknown): UpdateOffer | null {
  if (typeof answer !== "object" || answer === null) return null;
  const said = answer as Record<string, unknown>;
  const version = text(said.version);
  const downloads = text(said.downloads);
  if (!version || !downloads || !/^https:\/\//.test(downloads)) return null;
  if (typeof said.way !== "string" || !ways.includes(said.way)) return null;
  if (typeof said.stage !== "string" || !stages.includes(said.stage)) return null;
  const progress = said.progress;
  return {
    current: text(said.current) ?? "",
    version,
    way: said.way as UpdateWay,
    stage: said.stage as UpdateStage,
    progress:
      typeof progress === "number" && Number.isFinite(progress)
        ? Math.min(1, Math.max(0, progress))
        : null,
    reason: text(said.reason),
    command: text(said.command),
    downloads,
  };
}

/** The banner's one button. */
type UpdateAction =
  | { readonly kind: "install"; readonly label: string }
  | { readonly kind: "link"; readonly label: string; readonly href: string }
  | { readonly kind: "copy"; readonly label: string; readonly text: string };

/** What the banner says for an offer. */
export interface UpdateView {
  readonly tone: "info" | "run" | "warn";
  readonly title: string;
  readonly detail: string;
  /** The command shown under the words, for a copy apt installed. */
  readonly command: string | null;
  /** Nothing to choose while the app is at work. */
  readonly action: UpdateAction | null;
  /** Whether Later is offered. */
  readonly later: boolean;
  /** At work: a whole-number percentage when the size is known, else "busy". */
  readonly progress: number | "busy" | null;
}

export function updateView(offer: UpdateOffer): UpdateView {
  const { version } = offer;
  const have = offer.current ? `You have ${offer.current}.` : "";
  if (offer.stage === "downloading" || offer.stage === "installing") {
    const downloading = offer.stage === "downloading";
    return {
      tone: "run",
      title: downloading ? `Downloading Mash ${version}…` : `Installing Mash ${version}…`,
      detail: downloading
        ? "Mash restarts once the download has been checked."
        : "Mash restarts in a moment.",
      command: null,
      action: null,
      later: false,
      progress: downloading && offer.progress !== null ? Math.round(offer.progress * 100) : "busy",
    };
  }
  if (offer.stage === "failed")
    return {
      tone: "warn",
      title: `Mash couldn’t update to ${version}`,
      detail: `${sentence(offer.reason ?? "The update did not install")} Mash is unchanged.`,
      command: null,
      action: { kind: "link", label: "Download instead", href: offer.downloads },
      later: true,
      progress: null,
    };
  const title = `Mash ${version} is available`;
  if (offer.way === "apt" && offer.command)
    return {
      tone: "info",
      title,
      detail:
        `${have} This copy was installed with apt, which updates the app and the mash command together.`.trim(),
      command: offer.command,
      action: { kind: "copy", label: "Copy command", text: offer.command },
      later: true,
      progress: null,
    };
  return {
    tone: "info",
    title,
    detail: have,
    command: null,
    action:
      offer.way === "install"
        ? { kind: "install", label: "Update and restart" }
        : { kind: "link", label: "Download update", href: offer.downloads },
    later: true,
    progress: null,
  };
}

/** A reason as a sentence: capitalised, with its full stop. */
function sentence(reason: string): string {
  const words = reason.trim();
  const closed = /[.!?…]$/.test(words) ? words : `${words}.`;
  return closed.charAt(0).toUpperCase() + closed.slice(1);
}

type Invoke = (command: string, args: object) => Promise<unknown>;

/** The app's commands, inside the desktop app and nowhere else. */
function appCommands(scope: object): Invoke | undefined {
  if (!inDesktopApp(scope)) return undefined;
  const invoke = (scope as { __TAURI_INTERNALS__?: { invoke?: Invoke } }).__TAURI_INTERNALS__
    ?.invoke;
  return typeof invoke === "function" ? invoke : undefined;
}

/** How soon the app is asked again: often while it is at work, seldom otherwise. */
export function nextLook(offer: UpdateOffer | null): number {
  return offer?.stage === "downloading" || offer?.stage === "installing" ? 1_000 : 20_000;
}

export class AppUpdate {
  #offer = $state<UpdateOffer | null>(null);
  #invoke: Invoke | undefined;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #watching = false;

  /** What the app has to offer now. */
  get offer(): UpdateOffer | null {
    return this.#offer;
  }

  /**
   * Ask the app what it has to offer, now and from time to time. Returns how
   * to stop. Outside the desktop app this does nothing.
   */
  watch(scope: object = globalThis): () => void {
    this.#invoke = appCommands(scope);
    if (!this.#invoke) return () => {};
    this.#watching = true;
    void this.#look();
    return () => {
      this.#watching = false;
      clearTimeout(this.#timer);
    };
  }

  /** Update and restart. Nothing is installed until this is chosen. */
  async install(): Promise<void> {
    await this.#ask("update_install");
  }

  /** Put the banner away for this version until the app starts again. */
  async later(): Promise<void> {
    this.#offer = null;
    await this.#ask("update_later");
  }

  async #ask(command: string): Promise<void> {
    try {
      await this.#invoke?.(command, {});
    } catch {
      // The app says where things stand when it is asked next.
    }
    if (this.#watching) await this.#look();
  }

  async #look(): Promise<void> {
    clearTimeout(this.#timer);
    try {
      this.#offer = readOffer(await this.#invoke?.("update_banner", {}));
    } catch {
      // An app that cannot answer has nothing to offer.
      this.#offer = null;
    }
    if (!this.#watching) return;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.#look(), nextLook(this.#offer));
  }
}

export const appUpdate = new AppUpdate();
