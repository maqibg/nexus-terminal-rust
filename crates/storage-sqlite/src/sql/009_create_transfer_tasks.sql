CREATE TABLE IF NOT EXISTS transfer_tasks (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    file_name TEXT NOT NULL,
    total_bytes INTEGER NOT NULL,
    transferred_bytes INTEGER NOT NULL,
    status TEXT NOT NULL,
    error TEXT,
    session_id TEXT,
    source_path TEXT,
    target_path TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
