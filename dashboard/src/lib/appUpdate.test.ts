import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppUpdate, nextLook, readOffer, type UpdateOffer, updateView } from "./appUpdate.svelte";

const said = (changes: object = {}): Record<string, unknown> => ({
  current: "1.4.0",
  version: "1.5.0",
  way: "install",
  stage: "available",
  progress: null,
  reason: null,
  command: null,
  downloads: "https://example.invalid/mash/downloads",
  ...changes,
});
const offer = (changes: object = {}): UpdateOffer => {
  const read = readOffer(said(changes));
  if (!read) throw new Error("the fixture is not an offer");
  return read;
};

describe("reading the app's answer", () => {
  it("takes an offer as the app says it", () => {
    expect(readOffer(said())).toEqual(said());
    expect(readOffer(said({ stage: "downloading", progress: 0.25 }))?.progress).toBe(0.25);
  });

  it("has nothing to show when the app has nothing to offer", () => {
    expect(readOffer(null)).toBeNull();
    expect(readOffer(undefined)).toBeNull();
    expect(readOffer("1.5.0")).toBeNull();
  });

  it("refuses an answer it does not understand", () => {
    expect(readOffer(said({ version: "" }))).toBeNull();
    expect(readOffer(said({ way: "teleport" }))).toBeNull();
    expect(readOffer(said({ stage: "done" }))).toBeNull();
    expect(readOffer(said({ downloads: "javascript:alert(1)" }))).toBeNull();
    expect(readOffer(said({ downloads: null }))).toBeNull();
  });

  it("keeps progress between nothing and all of it", () => {
    expect(readOffer(said({ progress: 7 }))?.progress).toBe(1);
    expect(readOffer(said({ progress: -1 }))?.progress).toBe(0);
    expect(readOffer(said({ progress: Number.NaN }))?.progress).toBeNull();
    expect(readOffer(said({ progress: "half" }))?.progress).toBeNull();
  });
});

describe("what the banner says", () => {
  it("offers to install where the app updates itself", () => {
    const view = updateView(offer());
    expect(view.title).toBe("Mash 1.5.0 is available");
    expect(view.detail).toBe("You have 1.4.0.");
    expect(view.action).toEqual({ kind: "install", label: "Update and restart" });
    expect(view.later).toBe(true);
    expect(view.progress).toBeNull();
  });

  it("points at the downloads page where it does not", () => {
    expect(updateView(offer({ way: "download" })).action).toEqual({
      kind: "link",
      label: "Download update",
      href: "https://example.invalid/mash/downloads",
    });
  });

  it("shows the apt command for a copy apt installed", () => {
    const command = "sudo apt install --only-upgrade mash-desktop";
    const view = updateView(offer({ way: "apt", command }));
    expect(view.command).toBe(command);
    expect(view.action).toEqual({ kind: "copy", label: "Copy command", text: command });
    expect(view.detail).toContain("installed with apt");
    // Without the command there is still the downloads page.
    expect(updateView(offer({ way: "apt" })).action?.kind).toBe("link");
  });

  it("shows progress and nothing to choose while the app is at work", () => {
    const downloading = updateView(offer({ stage: "downloading", progress: 0.416 }));
    expect(downloading.title).toBe("Downloading Mash 1.5.0…");
    expect(downloading.progress).toBe(42);
    expect(downloading.action).toBeNull();
    expect(downloading.later).toBe(false);
    expect(updateView(offer({ stage: "downloading" })).progress).toBe("busy");
    const installing = updateView(offer({ stage: "installing", progress: 0.5 }));
    expect(installing.title).toBe("Installing Mash 1.5.0…");
    expect(installing.progress).toBe("busy");
    expect(installing.action).toBeNull();
  });

  it("gives a failure its reason and the download as the way on", () => {
    const view = updateView(offer({ stage: "failed", reason: "the download is gone" }));
    expect(view.tone).toBe("warn");
    expect(view.title).toBe("Mash couldn’t update to 1.5.0");
    expect(view.detail).toBe("The download is gone. Mash is unchanged.");
    expect(view.action).toEqual({
      kind: "link",
      label: "Download instead",
      href: "https://example.invalid/mash/downloads",
    });
    expect(view.later).toBe(true);
    expect(updateView(offer({ stage: "failed", reason: "no room (disk full)" })).detail).toBe(
      "No room (disk full). Mash is unchanged.",
    );
    expect(updateView(offer({ stage: "failed", reason: "It stopped." })).detail).toBe(
      "It stopped. Mash is unchanged.",
    );
    expect(updateView(offer({ stage: "failed" })).detail).toBe(
      "The update did not install. Mash is unchanged.",
    );
  });

  it("asks again soon only while the app is at work", () => {
    expect(nextLook(null)).toBe(20_000);
    expect(nextLook(offer())).toBe(20_000);
    expect(nextLook(offer({ stage: "failed" }))).toBe(20_000);
    expect(nextLook(offer({ stage: "downloading" }))).toBe(1_000);
    expect(nextLook(offer({ stage: "installing" }))).toBe(1_000);
  });
});

describe("asking the app", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  /** A stand-in for the desktop app: answers with `answer` and notes what it was asked. */
  function app(first: unknown) {
    const state = { answer: first, asked: [] as string[] };
    const invoke = async (command: string): Promise<unknown> => {
      state.asked.push(command);
      return command === "update_banner" ? state.answer : null;
    };
    return { state, scope: { __MASH_DESKTOP__: true, __TAURI_INTERNALS__: { invoke } } };
  }

  it("asks nothing in a browser", async () => {
    const { state, scope } = app(said());
    const update = new AppUpdate();
    update.watch({ __TAURI_INTERNALS__: scope.__TAURI_INTERNALS__ });
    update.watch({ __MASH_DESKTOP__: true });
    await vi.advanceTimersByTimeAsync(60_000);
    await update.install();
    expect(state.asked).toEqual([]);
    expect(update.offer).toBeNull();
  });

  it("shows what the app offers and looks again from time to time", async () => {
    const { state, scope } = app(null);
    const update = new AppUpdate();
    const stop = update.watch(scope);
    await vi.advanceTimersByTimeAsync(0);
    expect(update.offer).toBeNull();
    state.answer = said();
    await vi.advanceTimersByTimeAsync(20_000);
    expect(update.offer?.version).toBe("1.5.0");
    stop();
    state.answer = null;
    await vi.advanceTimersByTimeAsync(60_000);
    expect(update.offer?.version).toBe("1.5.0");
    expect(state.asked).toEqual(["update_banner", "update_banner"]);
  });

  it("looks at once when the app says to", async () => {
    const { state, scope } = app(null);
    const page = Object.assign(new EventTarget(), scope);
    const update = new AppUpdate();
    const stop = update.watch(page);
    await vi.advanceTimersByTimeAsync(0);
    state.answer = said();
    page.dispatchEvent(new Event("mash-update"));
    await vi.advanceTimersByTimeAsync(0);
    expect(update.offer?.version).toBe("1.5.0");
    expect(state.asked).toEqual(["update_banner", "update_banner"]);
    stop();
    state.answer = null;
    page.dispatchEvent(new Event("mash-update"));
    await vi.advanceTimersByTimeAsync(0);
    expect(update.offer?.version).toBe("1.5.0");
  });

  it("installs only when asked, then follows the app's progress", async () => {
    const { state, scope } = app(said());
    const update = new AppUpdate();
    const stop = update.watch(scope);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(state.asked).not.toContain("update_install");
    state.answer = said({ stage: "installing" });
    await update.install();
    expect(state.asked.slice(-2)).toEqual(["update_install", "update_banner"]);
    expect(update.offer?.stage).toBe("installing");
    const before = state.asked.length;
    await vi.advanceTimersByTimeAsync(3_000);
    expect(state.asked.length - before).toBe(3);
    stop();
  });

  it("puts the banner away at once on Later and tells the app", async () => {
    const { state, scope } = app(said());
    const update = new AppUpdate();
    const stop = update.watch(scope);
    await vi.advanceTimersByTimeAsync(0);
    state.answer = null;
    const later = update.later();
    expect(update.offer).toBeNull();
    await later;
    expect(state.asked).toContain("update_later");
    expect(update.offer).toBeNull();
    stop();
  });

  it("shows nothing when the app cannot answer", async () => {
    const invoke = async (): Promise<unknown> => {
      throw new Error("not allowed");
    };
    const update = new AppUpdate();
    const stop = update.watch({ __MASH_DESKTOP__: true, __TAURI_INTERNALS__: { invoke } });
    await vi.advanceTimersByTimeAsync(0);
    await update.install();
    expect(update.offer).toBeNull();
    stop();
  });
});
