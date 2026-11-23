import {
  Action,
  ActionPanel,
  Detail,
  Icon,
  type Image,
  Keyboard,
  List,
  getPreferenceValues,
  openExtensionPreferences,
} from "@raycast/api";
import { useExec, usePromise } from "@raycast/utils";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { homedir } from "node:os";
import { basename, join } from "node:path";
import { useMemo, useState } from "react";
import {
  type AgentApp,
  type ReadText,
  claudeOwnedSessions,
  claudeSessionsFolder,
  installedAgentApps,
  isClaudeSessionFile,
  openInAgentApp,
} from "./agentApps";
import {
  type Hit,
  MashError,
  agentLogo,
  agentName,
  appLink,
  chatLink,
  findMash,
  hasDesktopApp,
  failures,
  literal,
  parseChats,
  parseView,
  recentChatsArguments,
  rendered,
  searchCommand,
  searchLink,
  type StreamScope,
  stillComing,
  viewArguments,
} from "./mash";
import { useStreamSearch } from "./stream";

interface Preferences {
  readonly mashPath?: string;
  /** Whether search results include matches in subagent chats; off unless the reader turns it on. */
  readonly subagentMatches?: boolean;
}

type Run = { stdout: string; exitCode: number | null; error?: Error };

const readText: ReadText = (path) => {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return undefined;
  }
};

/** The Claude app's session files: one per session, under account and organisation folders. */
function claudeSessionFiles(home: string): string[] {
  const children = (folder: string): string[] => {
    try {
      return readdirSync(folder).map((name) => join(folder, name));
    } catch {
      return [];
    }
  };
  return children(claudeSessionsFolder(home))
    .flatMap(children)
    .flatMap(children)
    .filter((file) => isClaudeSessionFile(basename(file)));
}

/** Where a chat opens in its agent's own app, as these results need it. */
interface AgentLinks {
  readonly installed: readonly AgentApp[];
  readonly owned: ReadonlySet<string>;
}

// Mash prints a JSON answer even when it fails, so read it before the exit
// code: its message names the fix (start the search service, run mash init).
function read<T>(parse: (stdout: string) => T) {
  return ({ stdout, exitCode, error }: Run): T => {
    if (stdout.trim()) return parse(stdout);
    throw error ?? new MashError(`Mash stopped without an answer (exit ${exitCode}).`);
  };
}

export default function Command() {
  const { mashPath, subagentMatches = false } = getPreferenceValues<Preferences>();
  const mash = useMemo(() => findMash(mashPath, homedir(), existsSync), [mashPath]);
  // Without the desktop app nothing opens mash:// links, so actions that
  // need one are offered only when it is installed.
  const hasApp = useMemo(() => hasDesktopApp(homedir(), existsSync), []);
  const [typed, setTyped] = useState("");
  const query = typed.trim();
  const searching = mash !== undefined && query.length >= 2;
  // Every source at once (this machine by words and by meaning, and your
  // other machines), as one list that fills in as each answers.
  const [scope, setScope] = useState<StreamScope>("all");
  const live = useStreamSearch(mash, query, scope, searching, subagentMatches);
  // Before anything is typed, the latest chats on this machine, as the
  // desktop panel shows. Listing reads metadata only, so it is quick. A
  // failure is shown in the list, not also as a toast on opening.
  const recent = useExec(mash ?? "mash", recentChatsArguments(), {
    execute: mash !== undefined,
    parseOutput: read(parseChats),
    onError: () => undefined,
  });

  if (!mash) {
    return (
      <List>
        <List.EmptyView
          icon={Icon.MagnifyingGlass}
          title={mashPath ? "Mash isn’t at the location you set" : "Mash isn’t installed"}
          description="Install Mash, or set where it is in the Mash extension’s preferences."
          actions={
            <ActionPanel>
              <Action.OpenInBrowser title="How to Install Mash" url="https://getmash.io/installation/" />
              <Action title="Set Mash Location" icon={Icon.Gear} onAction={openExtensionPreferences} />
            </ActionPanel>
          }
        />
      </List>
    );
  }

  // Mash leaves subagent chats out of both unless the reader asks, in the
  // extension's preferences, for search to include them.
  const hits = (searching ? live.hits : recent.data) ?? [];
  const isLoading = searching ? !live.done : recent.isLoading;
  const shownError = searching ? live.error : recent.error;
  const coming = searching ? stillComing(live.sources) : "";
  const failed = searching ? failures(live.sources) : "";
  // Results come from this machine only, so its agents' apps can open them.
  const installed = useMemo(() => installedAgentApps(homedir(), readText), []);
  const claudeChats = installed.some((app) => app.agent === "claude-code")
    ? hits.filter((hit) => hit.agent === "claude-code").map((hit) => hit.chatId)
    : [];
  const owned = usePromise(
    async (ids: string[]) =>
      ids.length ? claudeOwnedSessions(claudeSessionFiles(homedir()), ids, readText) : new Set<string>(),
    [claudeChats],
  );
  const links: AgentLinks = { installed, owned: owned.data ?? new Set() };
  return (
    <List
      isLoading={isLoading}
      onSearchTextChange={setTyped}
      throttle
      onSelectionChange={() => live.interact()}
      searchBarAccessory={
        <List.Dropdown tooltip="Whose history" value={scope} onChange={(value) => setScope(value as StreamScope)}>
          <List.Dropdown.Item title="All machines" value="all" />
          <List.Dropdown.Item title="This machine" value="local" />
        </List.Dropdown>
      }
      searchBarPlaceholder="Search your coding-agent history"
      isShowingDetail={searching && hits.length > 0}
    >
      {shownError && hits.length === 0 ? (
        <List.EmptyView
          icon={Icon.Warning}
          title={searching ? "Mash couldn’t search" : "Mash couldn’t list your chats"}
          description={shownError.message}
          actions={<Trouble retry={searching ? live.retry : recent.revalidate} hasApp={hasApp} />}
        />
      ) : !searching ? (
        <List.EmptyView
          icon={Icon.MagnifyingGlass}
          title="Type to search what your agents and you said"
          description="Your latest chats on this machine appear here once Mash has indexed some."
          actions={hasApp ? <ActionPanel>{openApp}</ActionPanel> : undefined}
        />
      ) : (
        <List.EmptyView
          icon={Icon.MagnifyingGlass}
          title={isLoading ? "Searching…" : `No chats mention “${query}”`}
          description={
            hasApp
              ? "Press ⌘↵ to search every machine and by meaning in Mash."
              : "This searches this machine’s chats by their words. Try other words, or press ⌘↵ to copy a terminal search that also covers your other machines."
          }
          actions={
            <ActionPanel>
              {fullSearch(query, hasApp)}
              <Action.OpenInBrowser
                title="How to Find a Missing Chat"
                icon={Icon.QuestionMarkCircle}
                url="https://getmash.io/find-missing-chat/"
              />
            </ActionPanel>
          }
        />
      )}
      <List.Section title={searching ? "Chats" : "Recent Chats"} subtitle={coming || undefined}>
        {hits.map((hit) => (
          <List.Item
            key={`${hit.machine ?? ""}/${hit.chatId}`}
            icon={agentIcon(hit.agent)}
            title={hit.title}
            subtitle={searching ? undefined : projectName(hit.project)}
            accessories={[
              ...(hit.byMeaning ? [{ tag: "by meaning" }] : []),
              ...(searching && hit.machine ? [{ text: hit.machine }] : []),
              ...(hit.updatedAt ? [{ date: hit.updatedAt }] : []),
            ]}
            detail={<HitDetail hit={hit} />}
            actions={<HitActions hit={hit} mash={mash} query={searching ? query : ""} hasApp={hasApp} links={links} />}
          />
        ))}
        {failed ? (
          <List.Item
            key="failed"
            icon={Icon.Warning}
            title={failed}
            accessories={[{ text: "↵ to retry" }]}
            actions={
              <ActionPanel>
                <Action title="Try Again" icon={Icon.ArrowClockwise} onAction={live.retry} />
              </ActionPanel>
            }
          />
        ) : null}
      </List.Section>
    </List>
  );
}

/** The last folder of a chat's project, short enough for a list row. */
function projectName(project: string): string | undefined {
  return project ? basename(project) : undefined;
}

/** What to do when Mash fails: its message is above, these are the next steps. */
function Trouble({ retry, hasApp }: { retry: () => void; hasApp: boolean }) {
  return (
    <ActionPanel>
      <Action title="Try Again" icon={Icon.ArrowClockwise} onAction={retry} />
      <Action.OpenInBrowser
        title="Open Troubleshooting Guide"
        icon={Icon.QuestionMarkCircle}
        url="https://getmash.io/troubleshooting/"
      />
      {hasApp ? openApp : null}
      <Action.OpenInBrowser title="How to Install Mash" url="https://getmash.io/installation/" />
      <Action title="Set Mash Location" icon={Icon.Gear} onAction={openExtensionPreferences} />
    </ActionPanel>
  );
}

/** Mash's main window, named as the desktop panel names it. */
const openApp = (
  <Action.Open title="Open Mash" icon={Icon.AppWindow} target={appLink} shortcut={Keyboard.Shortcut.Common.Open} />
);

/** The agent's own logo, in its colour; ink logos follow Raycast's theme. */
function agentIcon(agent: string): Image.ImageLike {
  const logo = agentLogo(agent);
  if (!logo) return Icon.Message;
  return {
    source: `agents/${logo.file}`,
    tintColor: logo.tint ?? { light: "#16150f", dark: "#f5f0e4", adjustContrast: false },
  };
}

function HitDetail({ hit }: { hit: Hit }) {
  return (
    <List.Item.Detail
      markdown={hit.passage ? rendered(hit.passage) : "_This chat matched by its title._"}
      metadata={
        <List.Item.Detail.Metadata>
          <List.Item.Detail.Metadata.Label title="Agent" text={agentName(hit.agent)} />
          {hit.project ? <List.Item.Detail.Metadata.Label title="Project" text={hit.project} /> : null}
          {hit.updatedAt ? (
            <List.Item.Detail.Metadata.Label title="Last active" text={hit.updatedAt.toLocaleString()} />
          ) : null}
        </List.Item.Detail.Metadata>
      }
    />
  );
}

function HitActions({
  hit,
  mash,
  query,
  hasApp,
  links,
}: {
  hit: Hit;
  mash: string;
  query: string;
  hasApp: boolean;
  links: AgentLinks;
}) {
  const inAgent = openInAgent(hit, links);
  const read = (
    <Action.Push
      title="Read Messages"
      icon={Icon.Text}
      target={<Messages hit={hit} mash={mash} hasApp={hasApp} inAgent={inAgent} />}
    />
  );
  const open = <Action.Open title="Open Chat in Mash" icon={Icon.AppWindow} target={chatLink(hit)} />;
  return (
    <ActionPanel>
      {hasApp ? open : read}
      {hasApp ? read : null}
      {inAgent}
      <CopyActions hit={hit} hasApp={hasApp} />
      {query ? fullSearch(query, hasApp) : null}
      {hasApp ? openApp : null}
    </ActionPanel>
  );
}

/** The action that opens the chat in its agent's own desktop app, when one is installed that can. */
function openInAgent(hit: Hit, links: AgentLinks) {
  const link = openInAgentApp(hit, links.installed, links.owned);
  return link ? (
    <Action.Open
      title={link.title}
      icon={agentIcon(hit.agent)}
      target={link.url}
      shortcut={Keyboard.Shortcut.Common.OpenWith}
    />
  ) : null;
}

/**
 * What can be copied from a chat. A mash:// link opens only in the desktop
 * app, so without it the message ID is offered instead: `mash view` and an
 * agent's Mash tools read the message by it.
 */
function CopyActions({ hit, hasApp }: { hit: Hit; hasApp: boolean }) {
  return (
    <ActionPanel.Section>
      {hasApp ? (
        <Action.CopyToClipboard title="Copy Link" content={chatLink(hit)} shortcut={Keyboard.Shortcut.Common.Copy} />
      ) : null}
      <Action.CopyToClipboard
        title="Copy Message ID"
        content={hit.messageId}
        shortcut={hasApp ? undefined : Keyboard.Shortcut.Common.Copy}
      />
      {hit.passage ? (
        <Action.CopyToClipboard
          title="Copy Passage"
          content={hit.passage}
          shortcut={Keyboard.Shortcut.Common.CopyName}
        />
      ) : null}
    </ActionPanel.Section>
  );
}

/**
 * Mash's full search for the same words, which also reads other machines: in
 * the desktop app when it is installed, else as a command to run.
 */
function fullSearch(query: string, hasApp: boolean) {
  const shortcut: Keyboard.Shortcut = { modifiers: ["cmd"], key: "return" };
  return hasApp ? (
    <Action.Open title="Search Everywhere" icon={Icon.MagnifyingGlass} target={searchLink(query)} shortcut={shortcut} />
  ) : (
    <Action.CopyToClipboard
      title="Copy Search Command"
      icon={Icon.Terminal}
      content={searchCommand(query)}
      shortcut={shortcut}
    />
  );
}

/** A chat's messages read with `mash view`: around the match, or from the start of a recent chat. */
function Messages({
  hit,
  mash,
  hasApp,
  inAgent,
}: {
  hit: Hit;
  mash: string;
  hasApp: boolean;
  inAgent: ReturnType<typeof openInAgent>;
}) {
  const { data, isLoading, error } = useExec(mash, viewArguments(hit.messageId), {
    parseOutput: read(parseView),
    failureToastOptions: { title: "Mash couldn’t read the chat" },
  });
  const markdown = error
    ? `**Mash couldn’t read this chat.**\n\n${literal(error.message)}`
    : (data ?? [])
        .map((message) => {
          const match = hit.matched && message.id === hit.messageId;
          const heading = match ? `### ${message.role} · the match` : `### ${message.role}`;
          return `${heading}\n\n${rendered(message.content)}`;
        })
        .join("\n\n");
  return (
    <Detail
      isLoading={isLoading}
      navigationTitle={hit.title}
      markdown={markdown}
      actions={
        <ActionPanel>
          {hasApp ? <Action.Open title="Open Chat in Mash" icon={Icon.AppWindow} target={chatLink(hit)} /> : null}
          {inAgent}
          <CopyActions hit={hit} hasApp={hasApp} />
        </ActionPanel>
      }
    />
  );
}
