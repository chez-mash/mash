-- Subagent markers now publish with their chat records. Chats marked before
-- this schema were published without theirs, so journal each one as changed:
-- the next delta amends it with its record and marker and resends none of its
-- messages, and a full snapshot carries the marker anyway. A silver with no
-- markers, such as a shared server's node silver, is left as it was.
UPDATE shared_publication SET revision = revision + 1
 WHERE singleton = 1 AND EXISTS(SELECT 1 FROM chat_subagents);
INSERT INTO shared_changes(chat_id, revision, whole)
SELECT a.chat_id, p.revision, 0
  FROM chat_subagents a CROSS JOIN shared_publication p
 WHERE p.singleton = 1
ON CONFLICT(chat_id) DO UPDATE SET revision = excluded.revision;
