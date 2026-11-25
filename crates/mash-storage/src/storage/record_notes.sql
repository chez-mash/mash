-- Notes join decisions and preferences as a kind of record. SQLite cannot
-- change the check on records.kind in place, and rebuilding the table would
-- cascade its drop into every revision, evidence quote and link. Widening a
-- CHECK is the one change SQLite documents as safe to make by editing the
-- stored table definition: no row changes, and every stored kind still passes.
PRAGMA writable_schema = ON;
UPDATE sqlite_schema
SET sql = replace(sql, '''fact'',''open_question'')', '''fact'',''open_question'',''note'')')
WHERE type = 'table' AND name = 'records';
PRAGMA writable_schema = RESET;
-- Rebuilding the index is an ordinary schema change, so it bumps the schema
-- cookie and other open connections reload the widened check.
DROP INDEX records_scope;
CREATE INDEX records_scope ON records(owner, scope, kind);
