CREATE TABLE IF NOT EXISTS exec_entries (
  job_id TEXT NOT NULL,
  target TEXT NOT NULL,
  error TEXT,
  exit_status INTEGER,
  stdout BLOB,
  stderr BLOB
);