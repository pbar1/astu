CREATE TABLE IF NOT EXISTS ping_entries (
  job_id TEXT NOT NULL,
  target TEXT NOT NULL,
  error TEXT,
  message BLOB
);