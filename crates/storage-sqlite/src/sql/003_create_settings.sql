CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS appearance_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS terminal_themes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    theme_type TEXT NOT NULL DEFAULT 'user',
    background TEXT, foreground TEXT, cursor TEXT, cursor_accent TEXT,
    selection_background TEXT, selection_foreground TEXT, selection_inactive_background TEXT,
    black TEXT, red TEXT, green TEXT, yellow TEXT, blue TEXT, magenta TEXT, cyan TEXT, white TEXT,
    bright_black TEXT, bright_red TEXT, bright_green TEXT, bright_yellow TEXT,
    bright_blue TEXT, bright_magenta TEXT, bright_cyan TEXT, bright_white TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notification_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_type TEXT NOT NULL,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    config TEXT NOT NULL DEFAULT '{}',
    enabled_events TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action_type TEXT NOT NULL,
    details TEXT
);

CREATE TABLE IF NOT EXISTS command_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    session_id TEXT,
    connection_id INTEGER,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS path_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    connection_id INTEGER,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS quick_commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    variables TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS quick_command_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS quick_command_tag_associations (
    quick_command_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (quick_command_id, tag_id),
    FOREIGN KEY (quick_command_id) REFERENCES quick_commands(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES quick_command_tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS favorite_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    connection_id INTEGER,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
)
