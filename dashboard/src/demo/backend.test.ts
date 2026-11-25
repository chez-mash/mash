import { describe, expect, it } from "vitest";
import { parseCatalog, parseMessages, parseSearch } from "../lib/api";
import { parseConsole } from "../lib/console";
import { messageText } from "../lib/message";
import { parseRecord, parseRecordPage } from "../lib/records";
import type { Answer, ReadPath } from "../lib/routes";
import {
  parseFleetCatalog,
  parseFleetJob,
  parseMachines,
  parseSetup,
} from "../lib/services.svelte";
import { parseSettings } from "../lib/settings";
import { parseFirstRun } from "../lib/setup";
import { parseStatus } from "../lib/status";
import { type Reply, respond, terms } from "./backend";
import { chats, localMachine } from "./history";
import { records } from "./records";

const now = Date.UTC(2026, 10, 12);
function get(path: string): Reply {
  return respond(path, "GET", now);
}
function body<P extends ReadPath>(path: P, query = ""): Answer<P> {
  const answer = get(`${path}${query}`);
  expect(answer.status, path).toBe(200);
  // What the browser would receive: only what survives as JSON.
  return JSON.parse(JSON.stringify(answer.body));
}

describe("the demo answers in the shapes mash dash sends", () => {
  it("lists every chat, on more than one machine and agent", () => {
    const catalog = parseCatalog(body("/api/chats"));
    expect(catalog.localNode).toBe(localMachine);
    expect(catalog.all).toHaveLength(chats.length);
    expect(new Set(catalog.chats.map((chat) => chat.machine)).size).toBeGreaterThan(1);
    expect(new Set(catalog.chats.map((chat) => chat.agent)).size).toBe(6);
    expect(new Set(catalog.all.map((chat) => chat.id)).size).toBe(chats.length);
  });

  it("folds each agent's subagents into the chat that started them", () => {
    const catalog = parseCatalog(body("/api/chats"));
    const subagents = chats.filter((chat) => chat.subagent);
    expect(subagents.map((chat) => chat.agent).sort()).toEqual([
      "claude-code",
      "claude-code",
      "codex",
      "codex",
    ]);
    expect(catalog.folded).toBe(subagents.length);
    expect(catalog.chats).toHaveLength(chats.length - subagents.length);
    const parent = catalog.chats.find((chat) => chat.id === "release-0-2");
    expect(parent?.subagents).toBe(2);
    const child = catalog.all.find((chat) => chat.id === "release-windows-tests");
    // Named for its task, not for the brief it was given.
    expect(child?.title).toBe("Fix the failing Windows test suite");
    expect(child?.subagent?.spawnCallId).toBe("release-0-2-call-2");
  });

  it("serves each transcript, with tool calls joined to their output", () => {
    for (const chat of chats) {
      const page = parseMessages(body("/api/chat", `?id=${chat.id}&offset=0&limit=100`));
      expect(page.messages).toHaveLength(chat.messages.length);
      const calls = page.messages.flatMap((message) =>
        (message.tools?.calls ?? []).map((c) => c.callId),
      );
      const results = page.messages.flatMap((message) =>
        (message.tools?.results ?? []).map((r) => r.callId),
      );
      expect(results).toEqual(calls);
    }
    expect(get("/api/chat?id=missing&offset=0").status).toBe(404);
  });

  it("opens the conversation around a message", () => {
    const page = parseMessages(body("/api/view", "?id=sign-in-tokens-2&context=25"));
    expect(page.messages.map((message) => message.id)).toContain("sign-in-tokens-2");
  });

  it("shows a later Codex continuation while keeping the original decision evidence", () => {
    const catalog = parseCatalog(body("/api/chats"));
    const original = catalog.chats.find((chat) => chat.id === "sign-in-tokens");
    const followUp = catalog.chats.find((chat) => chat.id === "mobile-keychain-follow-up");
    const earlierCart = catalog.chats.find((chat) => chat.id === "offline-cart");
    expect(original).toMatchObject({ agent: "claude-code", machine: "laptop" });
    expect(followUp).toMatchObject({ agent: "codex", machine: "workstation" });
    expect(followUp?.updatedMs).toBeGreaterThan(original?.updatedMs ?? 0);
    expect(earlierCart?.updatedMs).toBeLessThan(original?.updatedMs ?? 0);

    const decision = parseRecord(body("/api/record", "?id=record-sign-in-tokens&node=laptop"));
    const quote = "OK, go with the tokens. Keep the mobile app on the same scheme.";
    expect(decision.revisions[0]?.evidence).toMatchObject([
      { chatId: "sign-in-tokens", messageId: "sign-in-tokens-2", quote, purpose: "endorsement" },
    ]);

    const query = new URLSearchParams({ q: '"device keychain"', provider: "codex" });
    const hits = parseSearch(body("/api/search", `?${query}`), localMachine).hits;
    expect(hits.map((hit) => hit.messageId)).toEqual(["mobile-keychain-follow-up-1"]);
    const followUpPage = parseMessages(
      body("/api/view", "?id=mobile-keychain-follow-up-1&shared=workstation&context=25"),
    );
    const recap = followUpPage.messages.find(
      (message) => message.id === "mobile-keychain-follow-up-1",
    );
    const text = recap ? messageText(recap) : "";
    expect(text).toContain(quote);
    expect(text).toContain("The mobile task is still open");
  });

  it("finds all query words in one message, in any order", () => {
    const page = parseSearch(body("/api/search", "?q=queue+invoice+exports"), localMachine);
    expect(page.hits.map((hit) => hit.messageId)).toEqual(["invoice-export-queue-0"]);
    expect(page.hits[0]?.before).toContain("invoice exports");
    expect(page.warnings.join(" ")).toContain("Demo search matches exact words");
  });

  it("requires exact phrases and preserves punctuation in filenames", () => {
    const phrase = parseSearch(body("/api/search", '?q="invoice+exports"'), localMachine);
    expect(phrase.hits.map((hit) => hit.messageId)).toEqual(["invoice-export-queue-0"]);
    const file = parseSearch(body("/api/search", "?q=0042_export_jobs.sql"), localMachine);
    expect(file.hits.map((hit) => hit.messageId)).toEqual(["invoice-export-queue-5"]);
  });

  it.each([
    ["autenticación", "spanish-sign-in-0"],
    ["corrigé", "french-pagination-0"],
    ["我们更新了身份验证令牌", "chinese-token-update-0"],
    ["認証トークンの更新手順を見直した", "japanese-token-update-0"],
  ])("finds %s in the original language", (q, messageId) => {
    const query = new URLSearchParams({ q, intent: "exact-mention" });
    const results = parseSearch(body("/api/search", `?${query}`), localMachine);
    expect(results.hits.map((hit) => hit.messageId)).toEqual([messageId]);
    expect(results.hits[0]?.before).toContain(q);
  });

  it("skips incomplete queries, terms split across messages, and tool output", () => {
    for (const q of [
      "invoice zebra",
      "missing-zebra",
      "queue dies",
      "choose a queue for invoice exports",
      '"exports invoice"',
      '"shows the order total"',
      "身份验证",
    ]) {
      const results = body("/api/search", `?${new URLSearchParams({ q })}`).results;
      expect(results, q).toEqual([]);
    }
  });

  it("applies the search filters", () => {
    const filtered = parseSearch(
      body(
        "/api/search",
        `?${new URLSearchParams({ q: "queue for exports", machine: "workstation" })}`,
      ),
      localMachine,
    );
    expect(filtered.hits.length).toBeGreaterThan(0);
    expect(filtered.hits.every((hit) => hit.chat.machine === "workstation")).toBe(true);
    const recent = parseSearch(
      body("/api/search", `?${new URLSearchParams({ q: "checkout", days: "7" })}`),
      localMachine,
    );
    expect(recent.hits.map((hit) => hit.chat.id)).toEqual(["flaky-checkout-test"]);
    expect(parseSearch(body("/api/search", "?q=zebra"), localMachine).hits).toEqual([]);
  });

  it("explains that search by meaning needs an installed Mash", () => {
    expect(get("/api/search?q=checkout&intent=related-discussion")).toMatchObject({
      status: 403,
      body: { error: { message: expect.stringContaining("Search by meaning") } },
    });
    expect(body("/api/search", "?q=checkout&intent=exact-mention").intent).toBe("exact-mention");
  });

  it("drops common words and endings from a question", () => {
    expect(terms("What did we decide about the tokens?")).toEqual(["decide", "token"]);
  });

  it("lists memories that quote a message which exists", () => {
    const page = parseRecordPage(body("/api/records", "?shared=true"));
    expect(page.records).toHaveLength(records.length);
    for (const entry of records) {
      const chat = chats.find((candidate) => candidate.id === entry.source.chat);
      expect(chat?.messages[entry.source.message]?.content, entry.id).toContain(entry.source.quote);
      expect(parseRecord(body("/api/record", `?id=${entry.id}&link_offset=0`)).id).toBe(entry.id);
    }
    expect(
      parseRecordPage(body("/api/records", "?kind=decision&shared=true")).records,
    ).toHaveLength(records.filter((entry) => entry.kind === "decision").length);
  });

  it("keeps this machine's memories apart from the shared server's", () => {
    const savedOn = (id: string) =>
      chats.find((chat) => chat.id === records.find((entry) => entry.id === id)?.source.chat)
        ?.machine;
    const local = parseRecordPage(body("/api/records")).records;
    expect(local.length).toBeGreaterThan(0);
    expect(local.length).toBeLessThan(records.length);
    for (const entry of local) {
      expect(entry.node).toBe("");
      expect(savedOn(entry.id)).toBe(localMachine);
    }
    const workstation = parseRecordPage(body("/api/records", "?shared=true&node=workstation"));
    expect(workstation.records.length).toBeGreaterThan(0);
    for (const entry of workstation.records) expect(entry.node).toBe("workstation");
  });

  it("reports status, settings, machines and tools", () => {
    expect(parseStatus(body("/api/status")).ready).toBe(true);
    expect(parseSettings(body("/api/services/settings")).editable).toBe(false);
    expect(parseFirstRun(body("/api/services/setup")).editable).toBe(false);
    expect(parseMachines(body("/api/services")).map((machine) => machine.id)).toEqual([
      localMachine,
    ]);
    expect(parseSetup(body("/api/services/installation", "?machine=laptop")).agents).toHaveLength(
      2,
    );
    const fleet = parseFleetCatalog(body("/api/services/fleet"));
    expect(fleet.enabled).toBe(true);
    expect(fleet.actions.every((item) => item.targets.includes(localMachine))).toBe(true);
    const jobs = body("/api/services/fleet/jobs").map(parseFleetJob);
    expect(jobs.map((job) => job.outcome)).toEqual(["succeeded", "succeeded", "succeeded"]);
    expect(parseConsole(body("/api/console/tools")).tools.length).toBeGreaterThan(0);
  });

  it("refuses to change or run anything", () => {
    expect(respond("/api/services/settings", "POST", now).status).toBe(403);
    expect(respond("/api/services/setup", "POST", now).status).toBe(403);
    expect(respond("/api/services/fleet/jobs", "POST", now).status).toBe(403);
    expect(get("/api/console/call?name=search").status).toBe(403);
    expect(get("/api/elsewhere").status).toBe(404);
  });
});
