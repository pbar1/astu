CREATE TABLE IF NOT EXISTS exec_entries (
  job_id BLOB NOT NULL,
  target TEXT NOT NULL,
  exit_status INTEGER NOT NULL,
  stdout BLOB,
  stderr BLOB
);
