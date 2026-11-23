"""How often searches run, what they look for, and which filters they use."""

import sqlite3
import sys
from pathlib import Path

log = (
    Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / ".mash" / "usage.sqlite3"
)
db = sqlite3.connect(f"file:{log}?mode=ro", uri=True)
# MCP and the CLI name the search `search`; the dashboard records its routes.
SEARCHES = "tool IN ('search', '/api/search', '/api/search/live', '/api/search/stream')"
# The query is `query` from agents and the CLI, `q` from the dashboard.
QUERY = "COALESCE(json_extract(arguments, '$.query'), json_extract(arguments, '$.q'))"

print("Searches per day")
for day, surface, times in db.execute(
    f"SELECT substr(at, 1, 10), surface, COUNT(*) FROM calls WHERE {SEARCHES} "
    "GROUP BY 1, 2 ORDER BY 1, 2"
):
    print(f"  {day}  {surface:<10} {times}")

print("Most common queries")
for query, times in db.execute(
    f"SELECT {QUERY} AS query, COUNT(*) FROM calls WHERE {SEARCHES} AND query IS NOT NULL "
    "GROUP BY query ORDER BY 2 DESC, query LIMIT 20"
):
    print(f"  {times:>4}  {query}")

print("Filters used")
for name, times in db.execute(
    f"SELECT key, COUNT(*) FROM calls, json_each(calls.arguments) WHERE {SEARCHES} "
    "AND key NOT IN ('query', 'q', 'db', 'config', 'models_dir') "
    "GROUP BY key ORDER BY 2 DESC, key"
):
    print(f"  {times:>4}  {name}")
