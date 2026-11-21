# Pick up where you left off

Start with a past chat and find the first thing you can do next.

## Follow a sample handoff

1. [Open the saved sign-in decision in the demo](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop).
2. Choose **open the message** to read the original Claude Code conversation.
   The remaining task is to store the mobile refresh token in the device keychain.
3. Choose **See the Codex follow-up** to see how a later sample chat picks up that task.

## Find the next step in your chats

1. Open **chats** in the dashboard. Search for a filename, error, or phrase
   from the work. Use the project filter if several projects have similar chats.
2. Open a result and read the last few messages. Look for what was finished,
   decisions you agreed to, and any test or task left open. Expand any relevant
   tool output to see what happened.
3. Take the next step yourself, or bring a short recap into a new agent session.
   Include the goal, the decision that still applies, and what remains to do.

For help finding the right chat, see [dashboard search](dashboard.md#find-and-read-a-chat)
and [what to check when a chat is missing](troubleshooting.md#search-returns-no-results).

## Reopen the original session

For one of your Codex or Claude Code chats, choose **Copy session ID** in the transcript.
On the computer where the original session is saved, open a terminal in the
project and replace `SESSION_ID` in the command for your agent.

```sh
codex resume SESSION_ID
claude --resume SESSION_ID
```

If the agent cannot open that ID, use its session picker to find the chat by
title, or start a fresh session with a recap.

## Ask an agent for a recap

With the Mash plugin and search tools set up, start a session in the project
you want to continue. Run `/mash:resume` in Claude Code. In Codex or Copilot,
ask “Use the Mash resume skill to tell me where we left off on this project.”
[Set up Mash in your agent](plugin.md#finish-setup-in-the-agent)

The skill tries to refresh the index, then checks for any handoff and reads
recent chats from the project. It summarises what happened, what is still
open, and what to do next. Ask it to show the original message behind any
decision you want to check.

## Leave a handoff for the next session

1. Before ending a session, run `/mash:handoff` in Claude Code or ask your
   agent to use the Mash `handoff` skill. It writes a short brief into the
   current chat. Check that it names the files or branch, settled decisions,
   unfinished work, and the next step.
2. After the agent has finished writing the brief, run `mash index` on the
   computer that has the chat. On the same computer, the resume skill can also
   refresh the index when you return.
3. Open the project in your next session and ask the agent to use `resume`.
   You can also search for `"Mash handoff"` in the dashboard and read the
   brief yourself.

If you switch computers, set up sharing first and index on the computer that
wrote the brief so it can publish the chat. For a manual handoff, copy the
brief into the new session. Share only the details you want the next agent to
read. [Search across your machines](shared-index.md) · [What an agent can see](privacy.md#when-history-can-leave-your-machine)
