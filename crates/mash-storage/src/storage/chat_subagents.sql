-- Chats an agent marked as subagents it started. Local only: it is read from
-- the agent's own files, never published, and replayable by reading them again.
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
