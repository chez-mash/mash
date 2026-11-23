-- Chats an agent marked as subagents it started. A node reads it from the
-- agent's own files, replayably, and publishes it with the chat record; the
-- shared server keeps each node's rows in that node's silver.
-- The parent is the chat the agent named, from the same provider and machine;
-- it may not be indexed here, so it is not a foreign key.
CREATE TABLE IF NOT EXISTS chat_subagents (
    chat_id TEXT PRIMARY KEY NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    parent_chat_id TEXT CHECK (parent_chat_id IS NULL OR (parent_chat_id <> '' AND parent_chat_id <> chat_id)),
    label TEXT,
    role TEXT,
    depth INTEGER CHECK (depth IS NULL OR depth >= 0),
    finished INTEGER CHECK (finished IS NULL OR finished IN (0, 1)),
    spawn_call_id TEXT
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS chat_subagents_parent ON chat_subagents(parent_chat_id)
    WHERE parent_chat_id IS NOT NULL;
-- Claude Code chats read from the desktop app's copy of a session it ran over
-- SSH (`projects/ssh-<sessionId>/`). The original lives on the remote machine
-- under the same chat ID; readers prefer it when another machine shares it.
CREATE TABLE IF NOT EXISTS chat_ssh_copies (
    chat_id TEXT PRIMARY KEY NOT NULL REFERENCES chats(id) ON DELETE CASCADE
) WITHOUT ROWID;
