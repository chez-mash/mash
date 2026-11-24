"""How often memory surfaced decisions and preferences, by month and tool."""

import sqlite3
import sys
from pathlib import Path

log = (
    Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / ".mash" / "usage.sqlite3"
)
db = sqlite3.connect(f"file:{log}?mode=ro", uri=True)
rows = db.execute(
    """
    SELECT substr(calls.at, 1, 7) AS month, calls.tool, call_results.record_kind,
           COUNT(*) AS times
    FROM call_results JOIN calls ON calls.id = call_results.call_id
    WHERE call_results.kind = 'record' AND call_results.record_kind IS NOT NULL
    GROUP BY month, calls.tool, call_results.record_kind
    ORDER BY month, calls.tool, call_results.record_kind
    """
).fetchall()
for month, tool, kind, times in rows:
    print(f"{month}  {tool:<20} {kind:<11} {times}")
