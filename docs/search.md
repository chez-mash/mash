# Introduction to Mash search

Mash searches your [saved coding-agent conversations](supported-agents.md) on
this machine and on other machines you connect through a [shared index](shared-index.md).
Search covers what you and your agent wrote. Open the transcript to read tool
calls and their output.

Use the [dashboard to find and read a chat](dashboard.md#find-and-read-a-chat),
or [ask a connected agent](agents.md#try-it). The examples below use the
terminal.

## Your first search

After [setting up local search](installation.md#connect-your-histories), try a
distinctive word, filename, or identifier from a chat. This example limits the
output to three results.

```bash
mash search 'authentication token' --limit 3
```

The JSON groups results by chat, best match first. Look for the `title` of the
chat and the `snippet` of the matching message. Copy that message’s
`lookup.view.id` and use it in place of `MESSAGE_ID` to read it in context.

```bash
mash view MESSAGE_ID --context 3
```

<details>
<summary>Example search result</summary>

```json
{
  "results": [
    {
      "chat_id": "01a1a77a-8105-7cf1-849b-37524f39e223",
      "provider": "codex",
      "title": "Add token authentication to the dashboard",
      "project": "/home/you/project",
      "matches": [
        {
          "lookup": { "view": { "id": "709e029507a46e56c1e1cf40" } },
          "role": "assistant",
          "snippet": "It's an authentication gate: the dashboard asks for a token…"
        }
      ]
    }
  ],
  "next_offset": null
}
```

</details>

Search uses the last index. If a conversation you expected is missing,
[check whether it was indexed and how to search for it](find-missing-chat.md).

## Read the original messages

`MESSAGE_ID` is a `lookup.view.id` from a search or `list-chats` result.
`--context 3` adds up to three messages before and after it. An ID that
matches no message fails with `not-found`. A chat ID is not accepted either;
that error names the chat's first message ID to use instead.

For local history, the output gives the message's `chat_id`, the chat's
`message_count`, the `messages` themselves, and how many messages come before
and after them (`omitted_before`, `omitted_after`). When some are left out, a
`notice` says so; raise `--context` to read more.

A result from another machine also has `lookup.view.shared`, the ID of the
node (machine) that published it. Pass it with `--shared`:

```bash
mash view MESSAGE_ID --shared NODE_ID --context 3
```

## Search in other languages

Use word search with the spelling from the original chat. For examples
and tips for Chinese or Japanese, see [searching chats in other languages](search-languages.md).
Search by meaning uses English models and works best for English chat text.

## Find a chat by title

```bash
mash list-chats --title-contains 'authentication' --limit 10
```

This lists chats without searching their messages, so it works without models
or a working search index. A chat's title is its first message, or for a named
Claude Code session, that name. A new name appears the next time the session
is indexed.

Each chat carries `lookup.view.id` for its first message, so you can start
reading from the top:

```bash
mash view FIRST_MESSAGE_ID --context 10
```

To search within one chat, pass its `chat_id` and ask for individual messages
rather than chats:

```bash
mash search 'token' --chat CHAT_ID --unit messages
```

## Narrow the results

| I remember… | Example |
| --- | --- |
| The agent | `mash search 'authentication' --provider codex` |
| Roughly when | `mash search 'authentication' --days 7` |
| A date range | `mash search 'authentication' --after 2026-10-01 --before 2026-11-01` |
| The project directory | `mash search 'authentication' --project /absolute/path/to/project` |
| The repository | `mash search 'authentication' --repository https://github.com/owner/repo.git` |
| Something I said | `mash search 'authentication' --role user` |
| The exact spelling | `mash search 'auth_token' --match-mode strict` |
| The exact phrase | `mash search '"rotate the logs"'` |

Different filters combine: a result must satisfy all of them. Repeat one flag
to accept alternatives, such as `--provider codex --provider claude-code`.

`--project` and `--repository` must match the recorded value exactly. Copy it
from the `project` or `repository` field of a `list-chats` or search result.

`--after` and `--before` accept a date (taken as UTC midnight) or an RFC3339
timestamp. `--after` includes the boundary; `--before` excludes it. On
`list-chats`, the same two flags instead take Unix seconds and filter by chat
activity.

## Search by meaning

Mash searches by word without any models by default. Pass `--intent exact-mention`
to choose word search explicitly if your configuration sets a different
[default](configuration.md#search-defaults).

Once [semantic search is enabled](indexing.md#enable-semantic-search), describe
what you remember in your own words:

```bash
mash search 'why did we choose a separate database' --intent related-discussion
```

Semantic queries currently need English.

Choose the intent by what you remember:

| | `exact-mention` | `related-discussion` |
| --- | --- | --- |
| Use for | Words, identifiers, filenames, error text | A question or idea whose wording may differ |
| Needs models | No | Yes |
| Several words | Each must appear in the message, in any order | Need not appear at all |

Word search matches the words themselves, so “log rotation” won't find “rotate
the application logs”; try “rotate”, or search by meaning. A single term such
as `auth_token` or `voyage-code-4` matches as written, not as separate words.
By default a word also matches as a prefix and tolerates small typos;
`--match-mode strict` requires the exact spelling.

## Search for an exact phrase

Put double quotes around the query, inside the shell's own quotes:

```bash
mash search '"rotate the logs"'
```

A quoted query matches as one phrase with exact spelling, not as independent
words scattered through a message. If it finds nothing, remove the quotes.

## More results and local-only searches

A page holds 20 results unless you set `--limit`. For the next page, repeat the
same command with `--offset` set to the returned `next_offset`; stop when it is
null.

`--sort recent` puts the newest results first within each page of relevant
results. For a chronological list of chats, use `list-chats` instead.

Use `--local` to search only this computer. If you share history, see
[which machines a search covers](search-across-machines.md#search-from-the-terminal)
for options to search published chats or narrow results to one machine.

If results are missing or stale, start with [troubleshooting](troubleshooting.md).
For other commands, see [useful commands](cli.md).
