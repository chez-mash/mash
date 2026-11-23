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
import { useExec } from "@raycast/utils";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { useMemo, useState } from "react";
import {
  type Hit,
  MashError,
  agentLogo,
  agentName,
  chatLink,
  findMash,
  literal,
  liveSearchArguments,
  parseSearch,
  parseView,
  rendered,
  searchLink,
  viewArguments,
} from "./mash";

interface Preferences {
  readonly mashPath?: string;
}

/** The desktop app is what opens mash:// links. */
const desktopApp = "/Applications/Mash.app";

type Run = { stdout: string; exitCode: number | null; error?: Error };

// Mash prints a JSON answer even when it fails, so read it before the exit
// code: its message names the fix (start the search service, run mash init).
function read<T>(parse: (stdout: string) => T) {
  return ({ stdout, exitCode, error }: Run): T => {
    if (stdout.trim()) return parse(stdout);
    throw error ?? new MashError(`Mash stopped without an answer (exit ${exitCode}).`);
  };
}

export default function Command() {
  const { mashPath } = getPreferenceValues<Preferences>();
  const mash = useMemo(() => findMash(mashPath, homedir(), existsSync), [mashPath]);
  const hasApp = useMemo(() => existsSync(desktopApp), []);
  const [typed, setTyped] = useState("");
  const query = typed.trim();
  const searching = mash !== undefined && query.length >= 2;
  const { data, isLoading, error } = useExec(mash ?? "mash", liveSearchArguments(query), {
    execute: searching,
    keepPreviousData: true,
    parseOutput: read(parseSearch),
    failureToastOptions: { title: "Mash couldn’t search" },
  });

  if (!mash) {
    return (
      <List>
        <List.EmptyView
          icon={Icon.MagnifyingGlass}
          title={mashPath ? "Mash isn’t at the location you set" : "Mash isn’t installed"}
          description="Install Mash, or set where it is in this command’s preferences."
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

  const hits = searching ? (data ?? []) : [];
  return (
    <List
      isLoading={searching && isLoading}
      onSearchTextChange={setTyped}
      throttle
      searchBarPlaceholder="Search your coding-agent history"
      isShowingDetail={hits.length > 0}
    >
      {query.length < 2 ? (
        <List.EmptyView icon={Icon.MagnifyingGlass} title="Type to search what your agents and you said" />
      ) : error && hits.length === 0 ? (
        <List.EmptyView icon={Icon.Warning} title="Mash couldn’t search" description={error.message} />
      ) : (
        <List.EmptyView
          icon={Icon.MagnifyingGlass}
          title={`No chats on this machine mention “${query}”`}
          description={hasApp ? "Press ⌘↵ to search every machine and by meaning in Mash." : undefined}
          actions={hasApp ? <ActionPanel>{fullSearch(query)}</ActionPanel> : undefined}
        />
      )}
      {hits.map((hit) => (
        <List.Item
          key={hit.chatId}
          icon={agentIcon(hit.agent)}
          title={hit.title}
          accessories={hit.updatedAt ? [{ date: hit.updatedAt }] : []}
          detail={<HitDetail hit={hit} />}
          actions={<HitActions hit={hit} mash={mash} query={query} hasApp={hasApp} />}
        />
      ))}
    </List>
  );
}

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

function HitActions({ hit, mash, query, hasApp }: { hit: Hit; mash: string; query: string; hasApp: boolean }) {
  const read = <Action.Push title="Read Messages" icon={Icon.Text} target={<Messages hit={hit} mash={mash} />} />;
  const open = <Action.Open title="Open in Mash" icon={Icon.AppWindow} target={chatLink(hit)} />;
  return (
    <ActionPanel>
      {hasApp ? open : read}
      {hasApp ? read : null}
      <Action.CopyToClipboard title="Copy Link" content={chatLink(hit)} shortcut={Keyboard.Shortcut.Common.Copy} />
      {hit.passage ? (
        <Action.CopyToClipboard
          title="Copy Passage"
          content={hit.passage}
          shortcut={Keyboard.Shortcut.Common.CopyName}
        />
      ) : null}
      {hasApp ? fullSearch(query) : null}
    </ActionPanel>
  );
}

function fullSearch(query: string) {
  return (
    <Action.Open
      title="Search All of Mash"
      icon={Icon.MagnifyingGlass}
      target={searchLink(query)}
      shortcut={{ modifiers: ["cmd"], key: "return" }}
    />
  );
}

/** The matching message with a few on either side, read with `mash view`. */
function Messages({ hit, mash }: { hit: Hit; mash: string }) {
  const { data, isLoading, error } = useExec(mash, viewArguments(hit.messageId), {
    parseOutput: read(parseView),
    failureToastOptions: { title: "Mash couldn’t read the chat" },
  });
  const markdown = error
    ? `**Mash couldn’t read this chat.**\n\n${literal(error.message)}`
    : (data ?? [])
        .map((message) => {
          const heading = message.id === hit.messageId ? `### ${message.role} · the match` : `### ${message.role}`;
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
          <Action.CopyToClipboard title="Copy Link" content={chatLink(hit)} />
        </ActionPanel>
      }
    />
  );
}
