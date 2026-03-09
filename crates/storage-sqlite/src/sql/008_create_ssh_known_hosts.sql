CREATE TABLE IF NOT EXISTS ssh_known_hosts (
    host TEXT NOT NULL,
    port INTEGER NOT NULL CHECK (port >= 1 AND port <= 65535),
    fingerprint TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (host, port)
);

CREATE INDEX IF NOT EXISTS idx_ssh_known_hosts_host ON ssh_known_hosts(host)
