-- Facts are no longer a kind of record: memory keeps decisions and
-- preferences. As with open questions, SQLite cannot change the check on
-- records.kind in place, so these triggers refuse the retired kind instead.
-- Deleting a record removes its revisions, evidence, confirmations and links.
DELETE FROM records WHERE kind = 'fact';
DELETE FROM record_changes WHERE record_id NOT IN (SELECT id FROM records);
CREATE TRIGGER IF NOT EXISTS records_retired_fact_insert BEFORE INSERT ON records
WHEN NEW.kind = 'fact'
BEGIN SELECT RAISE(ABORT, 'facts are no longer a kind of record'); END;
CREATE TRIGGER IF NOT EXISTS records_retired_fact_update BEFORE UPDATE OF kind ON records
WHEN NEW.kind = 'fact'
BEGIN SELECT RAISE(ABORT, 'facts are no longer a kind of record'); END;
