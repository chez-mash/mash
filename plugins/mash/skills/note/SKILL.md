---
name: note
description: Save a note to Mash memory. Use when the person runs /mash:note, or says "note that", "add a note that", "make a note that" or "take a note", to keep something for later sessions that is not a decision or a preference.
---

# Save a note to Mash memory

The person wants one thing kept for later sessions as a note: something worth
knowing that is not a choice, such as where a service runs, when a job resets,
or who owns an area. Save it now with Mash's `record` tool; hosts may prefix
the tool names. This is the `remember` skill with the kind already chosen.

## What to save

Whatever follows the command. With nothing after it, save the most recent
thing the person asked you to note in this conversation, and say which one
you chose. If nothing qualifies, ask what to note instead of inventing
something.

Write it as one statement that will make sense alone in a later session. It
does not have to be the person's wording: name the project, file or tool it
concerns and state the point plainly.

## How to save it

1. Call `help` with `tool: "record"` for the current arguments. The loaded
   skill may describe another release; live help is authoritative.
2. Call `find_record_matches` with the same scope and body you are about to
   save. If a record already says it, tell the person and stop. If one says
   something that this replaces, supersede that record instead of adding a
   second one.
3. Call `record` with:
   - the scope and `session_id` from the Mash note at the start of this
     session (repository scope inside a repository, personal for something
     about the person rather than a project);
   - `kind: "note"` and `state: "active"`, with no other kind fields;
   - `evidence`: the person's exact words from this conversation as the
     quote, with purpose `endorsement`. The quote is checked against this
     session's transcript, so copy it exactly, typos included.
   - review `established`. A note exists only because the person asked for
     it, so Mash refuses a candidate note.
4. If the save fails, say so with the error. Retry once on `index-busy`.
   Never report something as saved that was not.

## Afterwards

Confirm in one line: the note as saved and its scope. Do not paste IDs
unless asked. Do not save anything else the person did not ask for.

Never save credentials, tokens or other secrets, even on request: say why and
offer to note where the secret is kept instead.
