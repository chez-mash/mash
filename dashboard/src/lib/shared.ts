// The shared server as this dashboard sees it, managing its machines with the
// admin key saved on this machine, and joining one with a join string. Join
// strings and admin keys travel only in request bodies and are never kept in
// the page beyond the form that holds them.
import { agoLabel } from "./browse";
import type { components } from "./contract";
import { send } from "./transport";

type Schemas = components["schemas"];
export type ServerView = Schemas["ServerView"];
export type GroupStatus = Schemas["SharedGroupStatus"];
export type Machine = Schemas["SharedMachine"];
export type Overview = Schemas["SharedServerOverview"];
export type JoinView = Schemas["JoinAnswerView"];
export type NewJoin = Schemas["JoinAnswer"];

async function post<T>(path: string, body: unknown): Promise<T> {
  const response = await send(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (!response.ok) {
    let detail = `Mash couldn’t do that (${response.status}).`;
    try {
      const failure = (await response.json()) as { error?: { message?: unknown } };
      if (typeof failure.error?.message === "string") detail = failure.error.message;
    } catch {
      /* A failure without a JSON body keeps the fallback. */
    }
    throw new Error(detail);
  }
  return (await response.json()) as T;
}

/** Check a join string, saving nothing; or join with it when `apply` is set. */
export function joinShared(join: string, apply: boolean): Promise<JoinView> {
  return post<JoinView>("/api/shared/join", { join: join.trim(), apply });
}

/** Save an admin key once the server accepts it, or forget the saved one. */
export function setAdminKey(key: string | null): Promise<{ admin_key: boolean }> {
  return post("/api/shared/admin/key", key === null ? {} : { key: key.trim() });
}

export type Change = "add" | "replace" | "remove" | "delete";

/** Change one machine with the saved admin key; a new join string for add and replace. */
export async function changeMachine(
  change: Change,
  groupId: string,
  nodeId: string,
  confirm?: string,
): Promise<NewJoin | null> {
  const answer = await post<{ join?: NewJoin | null }>("/api/shared/admin/change", {
    change,
    group_id: groupId,
    node_id: nodeId,
    ...(confirm === undefined ? {} : { confirm }),
  });
  return answer.join ?? null;
}

/** Words for how a machine stands with the server. */
export function machineState(machine: Machine, now: number): string {
  switch (machine.state) {
    case "configured":
      return "set in the server's configuration";
    case "waiting": {
      const hours = Math.max(0, Math.round((machine.expires_at - now) / 3600));
      return `waiting to join · its join string works for ${hours} more hour${hours === 1 ? "" : "s"}`;
    }
    case "expired":
      return "never joined · its join string expired";
    case "joined":
      return machine.published ? "joined" : "joined · waiting for first publish";
    case "removed":
      return machine.delete_requested || machine.delete_waiting
        ? "removed · deleting what it published"
        : "removed · what it published stays";
  }
}

/** When a machine last published, in words. */
/** A machine of the group in one short line: its standing only when it is
 * unusual, then what it published and how long ago. The exact time goes in
 * `title`. */
export function machineLine(machine: Machine, now: number): { text: string; title?: string } {
  const at = machine.published?.last_synced_at;
  const usual = machine.state === "configured" || (machine.state === "joined" && at);
  const parts = usual ? [] : [machineState(machine, now)];
  if (!at || !machine.published) return { text: [...parts, lastPublish(machine)].join(" · ") };
  parts.push(
    `${machine.published.chats.toLocaleString()} chats`,
    `published ${agoLabel(Date.parse(at), now * 1000)}`,
  );
  return { text: parts.join(" · "), title: new Date(at).toLocaleString() };
}

export function lastPublish(machine: Machine): string {
  const at = machine.published?.last_synced_at;
  if (!at)
    return machine.receiving
      ? `receiving ${machine.receiving.covered} of ${machine.receiving.total} chats`
      : "nothing published yet";
  return `${machine.published?.chats.toLocaleString()} chats · last published ${new Date(at).toLocaleString()}`;
}
