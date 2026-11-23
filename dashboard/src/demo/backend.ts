// Answers dashboard requests in the browser from demo history. The tests run
// the responses through the same parsers used by `mash dash`.
import type { Answer, ReadPath } from "../lib/routes";
import type { Transport } from "../lib/transport";
import { chats, type DemoChat, type DemoMessage, localMachine } from "./history";
import { record, recordList } from "./memory";
import { records } from "./records";
import { messageMatcher, searchSnippet, terms } from "./search";
import tools from "./tools.json";

export { terms };

const DAY = 86_400_000;
const MINUTE = 60_000;

/** What the demo sends back: a status and a JSON body. */
export interface Reply {
  readonly status: number;
  readonly body: unknown;
}
/** A request the demo turns down, with the reason shown to the reader. */
class Refusal {
  constructor(
    readonly status: number,
    readonly message: string,
  ) {}
}
const refuse = (status: number, message: string): Refusal => new Refusal(status, message);
const notInDemo = (what: string): Refusal =>
  refuse(403, `${what} is switched off in the demo. Install Mash to use it on your own history.`);

function updatedAt(chat: DemoChat, now: number): number {
  return Math.round(now - chat.daysAgo * DAY);
}
function createdAt(chat: DemoChat, now: number): number {
  return updatedAt(chat, now) - chat.messages.length * 2 * MINUTE;
}
const callId = (chat: DemoChat, position: number): string => `${chat.id}-call-${position}`;
function row(chat: DemoChat, now: number): Answer<"/api/chats">["chats"][number] {
  const marker = chat.subagent;
  const parent = marker && chats.find((candidate) => candidate.id === marker.parent);
  const subagents = chats.filter((candidate) => candidate.subagent?.parent === chat.id).length;
  return {
    chat_id: chat.id,
    provider: chat.agent,
    node_id: chat.machine,
    title: chat.title,
    project: chat.project,
    repository: chat.repository,
    created_at: createdAt(chat, now),
    updated_at: updatedAt(chat, now),
    ...(subagents ? { subagents } : {}),
    ...(marker
      ? {
          subagent: {
            parent_chat_id: marker.parent,
            ...(parent ? { parent_title: parent.title } : {}),
            label: marker.label,
            role: marker.role,
            ...(marker.finished === undefined ? {} : { finished: marker.finished }),
            ...(parent && marker.spawnedAt !== undefined
              ? { spawn_call_id: callId(parent, marker.spawnedAt) }
              : {}),
          },
        }
      : {}),
  };
}
const messageId = (chat: DemoChat, position: number): string => `${chat.id}-${position}`;
function message(
  chat: DemoChat,
  entry: DemoMessage,
  position: number,
  now: number,
): Answer<"/api/chat">["messages"][number] {
  return {
    id: messageId(chat, position),
    ordinal: position,
    role: entry.role,
    content: entry.content,
    created_at: createdAt(chat, now) + position * 2 * MINUTE,
    ...(entry.call
      ? { tools: { tool_calls: [{ ...entry.call, call_id: callId(chat, position) }] } }
      : {}),
    ...(entry.answers === undefined
      ? {}
      : { tools: { tool_results: [{ call_id: callId(chat, entry.answers), is_error: false }] } }),
  };
}
function messages(chat: DemoChat, now: number) {
  return chat.messages.map((entry, position) => message(chat, entry, position, now));
}

function list(params: URLSearchParams, key: string): string[] {
  return (params.get(key) ?? "").split(",").filter(Boolean);
}

function search(params: URLSearchParams, now: number): Answer<"/api/search"> | Refusal {
  if (params.get("intent") === "related-discussion") return notInDemo("Search by meaning");
  const query = params.get("q") ?? "";
  const match = messageMatcher(query);
  const agents = list(params, "provider");
  const machines = list(params, "machine");
  const projects = list(params, "project");
  const days = params.get("days");
  const since = days === null ? 0 : now - Number(days) * DAY;
  const results = chats
    .filter(
      (chat) =>
        (!agents.length || agents.includes(chat.agent)) &&
        (!machines.length || machines.includes(chat.machine)) &&
        (!projects.length || projects.includes(chat.project)) &&
        updatedAt(chat, now) >= since,
    )
    .flatMap((chat) => {
      const matches = chat.messages.flatMap((entry, position) => {
        if (entry.role === "tool") return [];
        const at = match(entry.content);
        return at === undefined ? [] : [{ entry, position, at }];
      });
      const shown = matches[0];
      if (!shown) return [];
      return [
        {
          score: matches.length - chat.daysAgo / 100,
          result: {
            ...row(chat, now),
            score: 0,
            matches: [
              {
                message_id: messageId(chat, shown.position),
                node_id: chat.machine,
                role: shown.entry.role,
                created_at: createdAt(chat, now) + shown.position * 2 * MINUTE,
                copies: [{ message_id: messageId(chat, shown.position), node_id: chat.machine }],
                fusion_score: 0,
                rerank_score: null,
                projection_span: { start_char: 0, end_char: shown.entry.content.length },
                snippet: searchSnippet(shown.entry.content, shown.at),
              },
            ],
          },
        },
      ];
    })
    .sort((a, b) => b.score - a.score)
    .map((scored) => scored.result);
  return {
    query,
    intent: "exact-mention",
    realized_mode: "lexical",
    unit: "chats",
    shared: true,
    results,
    next_offset: null,
    notice:
      "Demo search matches exact words and phrases. Installed Mash also supports prefixes, small typos, and optional search by meaning.",
  };
}

function chatOf(id: string | null): DemoChat | undefined {
  return chats.find((chat) => chat.id === id);
}
function transcript(params: URLSearchParams, now: number): Answer<"/api/chat"> | Refusal {
  const chat = chatOf(params.get("id"));
  if (!chat) return refuse(404, "That chat is not in the demo's history.");
  const offset = Number(params.get("offset") ?? 0);
  const limit = Number(params.get("limit") ?? 100);
  return { id: chat.id, messages: messages(chat, now).slice(offset, offset + limit) };
}
function view(params: URLSearchParams, now: number): Answer<"/api/view"> | Refusal {
  const id = params.get("id") ?? "";
  const chat = chats.find((candidate) => id.startsWith(`${candidate.id}-`));
  if (!chat) return refuse(404, "That message is not in the demo's history.");
  return { id, messages: messages(chat, now) };
}

const roots = {
  "claude-code": ["/home/sam/.claude/projects"],
  codex: ["/home/sam/.codex/sessions"],
  cursor: ["/home/sam/.config/Cursor/User/workspaceStorage"],
  copilot: ["/home/sam/.copilot/session-state"],
  opencode: ["/home/sam/.local/share/opencode"],
};
type Rooted = keyof typeof roots;
function perAgent<T>(entry: (paths: string[]) => T): Record<Rooted, T> {
  return {
    "claude-code": entry(roots["claude-code"]),
    codex: entry(roots.codex),
    cursor: entry(roots.cursor),
    copilot: entry(roots.copilot),
    opencode: entry(roots.opencode),
  };
}
function status(): Answer<"/api/status"> {
  const local = chats.filter((chat) => chat.machine === localMachine);
  const count = local.reduce((sum, chat) => sum + chat.messages.length, 0);
  return {
    ready: true,
    local_qdrant: { url: "http://127.0.0.1:6333", state: "reachable" },
    chats: local.length,
    messages: count,
    searchable_messages: count,
    models_installed: true,
    realized_mode: "hybrid",
    database_path: "/home/sam/.mash/mash.sqlite",
    recommended_action: null,
    embeddings: count,
    stored_embeddings: count,
    query_models: "available",
    semantic_support: true,
    installation: { attention: 0, unverified: 0, unconfigured: 0, clients: [] },
    local_artifacts: { metadata: "ready", lexical: "ready", dense: "ready" },
    lexical: null,
    views: null,
    configuration: {
      path: "/home/sam/.mash/config.toml",
      loaded: true,
      providers: perAgent((paths) => ({ roots: paths })),
      attachments: { share_files: false, allowed_roots: [] },
      index: { since_days: 90, destination: "both" },
      embeddings: { backend: "local" },
      reranking: { backend: "local" },
      qdrant: {
        url: "http://127.0.0.1:6333",
        grpc_url: "http://127.0.0.1:6334",
        api_key_env: null,
        memory: "durable",
        vector_search: { algorithm: "hnsw", ef_runtime: 200 },
      },
      views: {
        build: { embedding_text: "dialogue" },
        query: {
          freshness_warnings: true,
          intent: "related-discussion",
          unit: "chats",
          limit: 20,
          fusion: { rank_constant: 60, lexical_weight: 1, semantic_weight: 1, recency_weight: 50 },
          ranking: "contextual-rerank",
        },
      },
      records: { enabled: true, backfill_since: null },
      redaction: { output: true, publication: true },
      shared: {
        group_id: "harbor",
        node_id: localMachine,
        api_url: "https://mash.harbor.example",
        token_env: "MASH_NODE_TOKEN",
      },
    },
  };
}
function settings(): Answer<"/api/services/settings"> {
  return {
    path: "/home/sam/.mash/config.toml",
    revision: "demo",
    editable: false,
    agents: perAgent((paths) => ({ indexed: true, roots: paths })),
    since_days: 90,
    destination: "both",
    shared: true,
    memory_capture: true,
    hide_credentials: { output: true, publication: true },
  };
}
/** What a dashboard with no service management reports: this machine's index, nothing to control. */
function firstRun(): Answer<"/api/services/setup"> {
  return {
    path: "/home/sam/.mash/config.toml",
    revision: "demo",
    configured: true,
    editable: false,
    histories: perAgent((paths) => ({ indexed: true, roots: paths })),
    since_days: 90,
    shared: true,
    integrations: [
      { agent: "claude-code", installed: true, connected: true },
      { agent: "codex", installed: true, connected: true },
    ],
    qdrant: { url: "http://127.0.0.1:6333", state: "reachable", reachable: true, message: null },
    models_installed: true,
    index: { state: "idle" },
  };
}
function machine(now: number): Answer<"/api/services/local"> {
  const checked = Math.round(now / 1000);
  // A typical install: a watcher indexing, local search, and one agent connected.
  const process = (
    id: string,
    name: string,
    kind: "watcher" | "qdrant" | "mcp",
    summary: string,
  ) => ({
    id,
    name,
    kind,
    state: "healthy" as const,
    summary,
    checked_at: checked,
    pid: null,
    actions: [],
    control_note: "Read by this demo; nothing here can be started or stopped.",
    control_revision: null,
    diagnostics: [],
    compatibility: null,
    logs_available: false,
  });
  return {
    id: localMachine,
    platform: "linux",
    checked_at: checked,
    reachable: true,
    warnings: [],
    services: [
      process("watcher-leadership", "Index watcher", "watcher", "Running · watching this index"),
      process("qdrant", "Local search", "qdrant", "Running · ready"),
      process("runtime-1", "Mash MCP", "mcp", "Connected"),
      {
        id: "sqlite",
        name: "SQLite index",
        kind: "sqlite",
        state: "healthy",
        summary: "Readable · compatible",
        checked_at: checked,
        pid: null,
        actions: [],
        control_note: "SQLite is a file, not a process. Checks do not change or migrate it.",
        control_revision: null,
        diagnostics: [{ label: "Read check", value: "Opened read-only." }],
        compatibility: null,
        logs_available: false,
      },
    ],
  };
}

type Read<P extends ReadPath> = (params: URLSearchParams, now: number) => Answer<P> | Refusal;

// One entry for every route the contract lists, each answering with that
// route's type: a route added to the server does not compile here until the
// demo answers it or declines to.
const reads: { readonly [P in ReadPath]: Read<P> } = {
  "/api/health": () => ({ build: tools.build_revision }),
  "/api/assistant": () => ({ enabled: false, model: null }), // the demo has no model to ask
  "/api/assistant/models": () => refuse(404, "No assistant is configured in the demo."),
  "/api/chats": (_, now) => ({
    local_node: localMachine,
    truncated: false,
    shared_error: null,
    shared_only: false,
    chats: chats.map((chat) => row(chat, now)),
  }),
  "/api/search": search,
  "/api/search/live": search, // the demo's search is already instant
  "/api/chat": transcript,
  "/api/view": view,
  "/api/open-in": () => ({ link: null }), // the demo opens no agent apps
  // The demo's laptop shares nothing, so it has no shared server to read or manage.
  "/api/shared/server": () => ({ admin_key: false, may_change: false }),
  "/api/shared/admin": () => refuse(404, "The demo has no shared server."),
  "/api/records": recordList,
  "/api/record": (params, now) => {
    const entry = records.find((candidate) => candidate.id === params.get("id"));
    return entry
      ? record(entry, now, params.has("node"))
      : refuse(404, "That memory is not in the demo.");
  },
  "/api/status": status,
  "/api/console/tools": () => tools,
  "/api/console/call": () => notInDemo("Running tools"),
  "/api/services/access": () => ({ authentication_required: false, local_node: localMachine }),
  "/api/services": (_, now) => ({ machines: [machine(now)] }),
  "/api/services/local": (_, now) => machine(now),
  "/api/services/settings": settings,
  "/api/services/setup": firstRun,
  "/api/services/fleet": fleet,
  "/api/services/fleet/jobs": fleetJobs,
  "/api/services/operations": () => [],
  "/api/services/log": () => refuse(404, "No configured log source for this service"),
  "/api/services/installation": () => ({
    build: tools.build_revision,
    agents: (["claude-code", "codex"] as const).map((name) => ({
      agent: name,
      expected: true,
      connected: true,
      steps: (["plugin", "skill", "mcp"] as const).map((component) => ({
        component,
        state: "present" as const,
        message: "Installed and readable.",
        remediation: null,
      })),
    })),
  }),
};
// The fleet manager is shown as a controller would see it: every machine in
// the history is a target and a few runs have finished. Starting one is a
// change, which the demo refuses like any other.
const fleetMachines = [...new Set(chats.map((chat) => chat.machine))].sort();
const fleetRevision = "demo";
const fleetActions = [
  "status",
  "preflight",
  "deploy",
  "sync",
  "local-views",
  "watch-start",
  "watch-stop",
  "watch-status",
  "watch-logs",
] as const;
function fleet(): Answer<"/api/services/fleet"> {
  return {
    enabled: true,
    revision: fleetRevision,
    actions: fleetActions.map((action) => ({ action, targets: fleetMachines })),
    services: [],
    dashboard_restarts: [],
  };
}
function fleetJobs(_: URLSearchParams, now: number): Answer<"/api/services/fleet/jobs"> {
  const seconds = Math.round(now / 1000);
  const finished = (
    id: string,
    action: (typeof fleetActions)[number],
    hoursAgo: number,
    took: number,
  ) => ({
    request: { id, action, targets: fleetMachines, revision: fleetRevision },
    requested_at: seconds - hoursAgo * 3600,
    completed_at: seconds - hoursAgo * 3600 + took,
    outcome: "succeeded" as const,
    targets: fleetMachines.map((machine) => ({
      id: machine,
      state: "succeeded" as const,
      detail: null,
    })),
  });
  return [
    finished("demo-status", "status", 3, 9),
    finished("demo-sync", "sync", 26, 148),
    finished("demo-deploy", "deploy", 50, 412),
  ];
}
function isRead(path: string): path is ReadPath {
  return Object.hasOwn(reads, path);
}

/** The answer `mash dash` would give to one request, from the demo's history. */
export function respond(url: string, method: string, now: number): Reply {
  const { pathname, searchParams } = new URL(url, "http://demo.invalid");
  const answer =
    method !== "GET"
      ? notInDemo("Changing anything")
      : isRead(pathname)
        ? reads[pathname](searchParams, now)
        : refuse(404, "This part of Mash is not in the demo.");
  return answer instanceof Refusal
    ? { status: answer.status, body: { error: { message: answer.message } } }
    : { status: 200, body: answer };
}

/** Installed by the demo's entry point in place of requests to `mash dash`. */
export const transport: Transport = (path, init) => {
  const answer = respond(path, init.method ?? "GET", Date.now());
  return Promise.resolve(
    new Response(JSON.stringify(answer.body), {
      status: answer.status,
      headers: { "Content-Type": "application/json" },
    }),
  );
};
