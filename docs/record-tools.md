# MCP tools for saved memory

Use this reference when calling Mash's tools directly. For saving and reviewing
memory in a connected agent or the dashboard, see [Use Mash memory](records.md).
For the current arguments for each tool, see [the MCP tool reference](tools.md#save-and-recall-records)
or ask a connected client to call `help` with `topic: "records"`.

## Recall records

Ask your agent to call `recall_records` with an exact `scope`:

```json
{"scope": {"kind": "repository", "identity": "YOUR_REPOSITORY_ID"}, "query": "search engine"}
```

The optional `query` ranks records by matching words and needs no embedding
model. Established current records come first, then candidates with
`status: "candidate"`.

- Recall returns at most eight records and 6,000 bytes by default. Adjust with
  `limit` (1–20) and `byte_budget` (1,024–32,768).
- `truncated` means a relevant result did not fit. `scan_limited` means the
  scope held more than 256 recent records. Use `list_records` for a complete
  inspection.
- `include_review` adds records labeled `stale_evidence` or `inactive`, and
  records with `conflicted: true`. Default recall and the plugin hooks omit
  them. These labels are not approval.
- `include_personal` adds personal records alongside another scope.
- `file: "src/main.rs"` with a repository or local-project scope matches that
  exact relative path in the record text or quotes.

## Inspect existing records

Call `list_records`. By default it returns established
current records whose evidence still matches the saved messages. Each one is
its current statement, reason, scope and the messages it cites; pass
`detail: "extended"` for quotes, confirmations, the owner and links.

- `include_inactive` adds candidates, older choices and records with changed
  or missing sources.
- `kind`, `state` and `review` filter on the latest revision. `chat` selects
  records citing that chat in any retained revision.
- Follow `next_offset` to continue, keeping the filters unchanged. A page can
  be short or hold no current records without being the last one.

Call `get_record` with a returned ID to read one record and its evidence. Set
`history` to `true` for prior revisions, confirmations and links to
replacement records, following the returned cursors until they are null.
Confirmation dates are when each confirmation was recorded, not when the
original message was sent.

### Shared records

To read published records, call `list_records` with `shared: true`. Results
include `node_id`, and `owner` with `detail: "extended"`; identical statements
from different owners stay separate. Select one node with `node_id`, and pass `next_cursor` back as
`cursor` with the same filters until no cursor remains. Pass a result's
`node_id` alongside its `id` to `get_record` to read that shared copy.

Shared reads use your configured group's authentication and need no models.
They do not fall back to local records when the server is unavailable. Edits
happen on the owning node and reach the shared copy through normal publication.
Records scoped to a local project path stay local.

## Save a conclusion

Read the supporting chat first, then call the `record` MCP
tool with an operation ID and an action. Every field sits at the top level:

```json
{
  "operation_id": "choose-local-qdrant-1",
  "action": "create",
  "scope": {"kind": "chat", "id": "RETURNED_CHAT_ID"},
  "kind": "decision",
  "state": "accepted",
  "statement": "Use Qdrant for local search.",
  "recorded_by": "My coding agent",
  "review": "established",
  "evidence": [{
    "message_id": "RETURNED_USER_MESSAGE_ID",
    "quote": "Yes, let's use Qdrant.",
    "purpose": "endorsement"
  }]
}
```

Replace the IDs and the quote with actual evidence. Reuse the same operation
ID only when retrying the same request after a lost response.

**Scope** is one of:

- `personal`, within this record owner's namespace
- `repository` with an exact `identity`
- `local_project` with an absolute `path`
- `chat` with an `id`

Repository identity and project path must match the cited chat's metadata;
a matching project basename is not enough.

Pass **kind and state** at the top level of the tool call:

| Kind | States | Additional fields |
| --- | --- | --- |
| `decision` | `proposed`, `accepted`, `superseded`, `withdrawn` | optional alternatives and consequences |
| `preference` | `active`, `superseded`, `withdrawn` | optional conditions |

Optional `effective_at` uses Unix milliseconds; omit an unknown date. Choices
not yet in effect are left out of the default list.

**Review** is `candidate` (the default) or `established`. Keep uncertain
interpretations as candidates. An established decision or preference requires
an `endorsement` citation of a user message. A decision with
`"decided_by": "agent"` instead requires a citation of the
agent's own message, and cannot revise or supersede a decision made by the
person. `recorded_by` is caller-supplied attribution, not authenticated
identity.

**Evidence** `purpose` is one of:

- `context` (the default): surrounding material, such as the proposal a short
  approval answers. Cite the proposal as `context` and the approval as
  `endorsement`.
- `endorsement`: the person visibly said it. User-role messages sometimes
  contain automated reports; do not cite those as endorsements.
- `report`: a source's attributed factual claim. It does not assert the
  person's approval.

An agent recording from its own running session can cite it without a message
ID. Mash cites the latest message in that session containing the quote:

```json
{"session_id": "THE_AGENTS_SESSION_ID", "quote": "Yes, let's use Qdrant.", "purpose": "endorsement"}
```

Record writes reject text containing private spans, and normal output applies
your configured credential masking.

### Check for duplicates

Before creating a record, call `find_record_matches` with the proposed `scope`
and `body`. It returns up to eight exact or similar-wording suggestions of the
same kind and scope. Similar wording does not prove agreement, and suggestions
never merge records or add confirmations.

`create` reports an identical live record instead of adding a second one. To
reuse it, set `on_duplicate: "reuse"` on the create call; the receipt
then has `reused: true` and the existing record's ID and revision. Reuse adds
no evidence or confirmation and never revives a retired choice.

## Correct or replace

Edits provide `expected: {"id": "RECORD_ID", "revision": 1}`. If another edit
has changed that revision, reread the record before trying again.

- `revise` corrects a summary or changes its lifecycle state, with supporting
  evidence. New revisions do not inherit confirmations of the old interpretation.
- `supersede` records a changed choice and retires its predecessor atomically.
  Returning to an earlier choice creates another record, preserving the sequence.
- `confirm` adds new supporting source messages to an established current record.
  Reusing the same evidence or retrying a request does not increase confirmations.
- `forget` removes the entire record, including its prior revisions and quotes.

To establish a candidate, `revise` it with a complete `body` that has
`review: "established"` and verified evidence. Reading or confirming a
candidate does not establish it.

Reindexing a changed chat keeps its records but marks changed evidence stale.
**Forgetting a cited chat also removes every dependent record**, including
records that cited that chat only in an older revision. A forgotten record
cannot be recreated by retrying an old request.

## Relate conclusions

To record that two records agree or disagree, use `record` with a new
`operation_id` and the `link` action:

```json
{
  "operation_id": "link-qdrant-choices-1",
  "action": "link",
  "expected": {"id": "SOURCE_ID", "revision": 1},
  "target": {"id": "TARGET_ID", "revision": 1},
  "relation": "contradicts"
}
```

`relation` is `supports` or `contradicts`. Both records must be established,
current, backed by matching evidence, and owned locally in the same scope. Mash
does not infer contradictions from wording or choose a winner.

A correction does not erase a disagreement. Retire or supersede a changed
claim: once either record is inactive, the conflict is no longer active.
Forgetting either record removes the link. Read links in both directions
through `get_record`'s `links`.

## Review an older chat

To review one chat by hand instead of running a backfill:

1. Call `get_extraction_checkpoint` with the chat ID and the scope in which
   you intend to save conclusions. It returns the current
   `source_fingerprint`, the last processed fingerprint (if any) and
   `up_to_date`.
2. Read the chat.
3. Call `extract_records` with that fingerprint, a stable `operation_id`, the
   same scope and chat ID, and up to 64 drafts. For each draft, pass record
   fields and `evidence` as in [Save a conclusion](#save-a-conclusion). Set
   `review: "candidate"` and cite messages from the reviewed chat.
   An empty `records` list records that the review found nothing to save.

If the chat changed during the review, the request is rejected: reread the
chat and submit a new operation. If only the response was lost, retry the
identical request with its original operation ID.

The checkpoint records the agent's claim to have reviewed that version, not
proof that it read every message or interpreted it correctly. Review the
candidates before establishing them.

## Back up records

[Portable `mash export` backups](backup.md) include records with their
revisions, evidence and confirmations. **Backups contain sensitive record text and quoted
evidence, as well as chats.**

A normal import creates a new owner and remaps record IDs and links. An
`import --keep-identity` restore retains IDs and retry receipts; retire the
original database before using the restored copy as its replacement.
