-- Migration v7: Fix passkeys schema drift (counter -> sign_count, add last_used_at)
-- 由 migrations.rs 程序化控制，仅在检测到 counter 列时运行

CREATE TABLE passkeys_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    credential_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT 0,
    transports TEXT,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 仅复制旧表中确实存在的列（counter → sign_count，last_used_at 置 NULL）
INSERT INTO passkeys_new (id, user_id, credential_id, public_key, sign_count, transports, name, created_at)
    SELECT id, user_id, credential_id, public_key, counter, transports, name, created_at
    FROM passkeys;

DROP TABLE passkeys;

ALTER TABLE passkeys_new RENAME TO passkeys;

CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id);

CREATE INDEX IF NOT EXISTS idx_passkeys_credential_id ON passkeys(credential_id);
