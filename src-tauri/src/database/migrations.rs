use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
  conn
    .execute_batch(
      "
    CREATE TABLE IF NOT EXISTS files (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        source_root     TEXT NOT NULL,
        relative_path   TEXT NOT NULL,
        dest_path       TEXT NOT NULL DEFAULT '',
        file_name       TEXT NOT NULL,
        file_size       INTEGER NOT NULL,
        modified_at     INTEGER NOT NULL,
        status          TEXT NOT NULL DEFAULT 'pending',
        backed_up_at    TEXT,
        verified_at     TEXT,
        error_message   TEXT,
        UNIQUE(source_root, relative_path)
    );

    CREATE INDEX IF NOT EXISTS idx_files_source_root ON files(source_root);
    CREATE INDEX IF NOT EXISTS idx_files_status ON files(status);
    CREATE INDEX IF NOT EXISTS idx_files_modified_at ON files(modified_at);

    CREATE TABLE IF NOT EXISTS backup_tasks (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        source_root     TEXT NOT NULL,
        dest_path       TEXT NOT NULL,
        total_files     INTEGER,
        total_bytes     INTEGER,
        backed_up_files INTEGER DEFAULT 0,
        backed_up_bytes INTEGER DEFAULT 0,
        failed_files    INTEGER DEFAULT 0,
        status          TEXT NOT NULL DEFAULT 'running',
        started_at      TEXT,
        completed_at    TEXT,
        avg_speed       REAL
    );

    CREATE TABLE IF NOT EXISTS delete_history (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        file_id         INTEGER REFERENCES files(id),
        source_root     TEXT NOT NULL,
        source_path     TEXT NOT NULL,
        deleted_at      TEXT NOT NULL,
        delete_method   TEXT NOT NULL,
        verified        BOOLEAN NOT NULL DEFAULT 1
    );

    CREATE TABLE IF NOT EXISTS settings (
        key             TEXT PRIMARY KEY,
        value           TEXT NOT NULL
    );
    ",
    )
    .map_err(|e| e.to_string())?;
  Ok(())
}
