use assert_cmd::Command;
use duckdb::params;
use duckdb::Connection;
use tempfile::tempdir;
use uuid::Uuid;

const TEST_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS jobs (
  job_id BLOB PRIMARY KEY,
  started_at TIMESTAMP,
  finished_at TIMESTAMP,
  command TEXT,
  concurrency BIGINT,
  task_count BIGINT
);
CREATE TABLE IF NOT EXISTS tasks (
  task_id BLOB PRIMARY KEY,
  job_id BLOB,
  started_at TIMESTAMP,
  finished_at TIMESTAMP,
  target_uri TEXT,
  command TEXT,
  status TEXT,
  exit_code BIGINT,
  error TEXT,
  connect_ms BIGINT,
  auth_ms BIGINT,
  exec_ms BIGINT
);
CREATE TABLE IF NOT EXISTS task_vars (
  task_id BLOB,
  key TEXT,
  value TEXT,
  PRIMARY KEY(task_id, key)
);
CREATE TABLE IF NOT EXISTS task_lines (
  task_id BLOB,
  stream TEXT,
  seq BIGINT,
  line_hash BLOB,
  PRIMARY KEY(task_id, stream, seq)
);
CREATE TABLE IF NOT EXISTS line_dict (
  line_hash BLOB PRIMARY KEY,
  line_text TEXT
);
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value BLOB);
"#;

fn init_test_schema(conn: &Connection) {
    conn.execute_batch(TEST_SCHEMA_SQL).expect("schema");
}

fn run_astu(
    data_dir: &std::path::Path,
    args: &[&str],
    stdin: Option<&str>,
) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("astu"));
    cmd.env("ASTU_DATA_DIR", data_dir).args(args);
    if let Some(stdin) = stdin {
        cmd.write_stdin(stdin);
    }
    cmd.assert()
}

#[test]
fn run_local_and_output_stdout() {
    let dir = tempdir().expect("tmpdir");

    run_astu(dir.path(), &["run", "--confirm=1", "echo hello"], None).success();

    run_astu(dir.path(), &["output", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn run_dummy_target() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &[
            "run",
            "-T",
            "dummy://fixture?stdout=mock-ok&exitcode=0",
            "--confirm=1",
            "ignored",
        ],
        None,
    )
    .success();

    run_astu(dir.path(), &["freq", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("mock-ok"));
}

#[test]
fn freq_stdout_uses_normalized_placeholders_for_dedupe() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &[
            "run",
            "-T",
            "dummy://alice@host-a?stdout=user=alice%20host=host-a%20val=42",
            "-T",
            "dummy://bob@host-b?stdout=user=bob%20host=host-b%20val=42",
            "--confirm=2",
            "ignored",
        ],
        None,
    )
    .success();

    run_astu(dir.path(), &["freq", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("user={user} host={host} val=42"))
        .stdout(predicates::str::contains("| 2     |"));
}

#[test]
fn freq_sections_are_separated_by_blank_lines() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &[
            "run",
            "-T",
            "dummy://fixture?stdout=ok&stderr=bad",
            "--confirm=1",
            "ignored",
        ],
        None,
    )
    .success();

    run_astu(dir.path(), &["freq"], None)
        .success()
        .stdout(predicates::str::contains("\n\nstderr\n"));
}

#[test]
fn noninteractive_without_confirm_fails() {
    let dir = tempdir().expect("tmpdir");

    run_astu(dir.path(), &["run", "echo hello"], Some(""))
        .failure()
        .stderr(predicates::str::contains("--confirm"));
}

#[test]
fn stdin_param_mode_works() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &["run", "--stdin", "param", "--confirm=2", "echo {param}"],
        Some("alpha beta"),
    )
    .success();

    run_astu(dir.path(), &["output", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("alpha"))
        .stdout(predicates::str::contains("beta"));
}

#[test]
fn lookup_reads_target_file_from_stdin() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &["lookup", "--target-file", "-"],
        Some("local:\ndummy://fixture\n"),
    )
    .success()
    .stdout(predicates::str::contains("local:"))
    .stdout(predicates::str::contains("dummy://fixture"));
}

#[test]
fn stdin_pipe_mode_works() {
    let dir = tempdir().expect("tmpdir");

    run_astu(
        dir.path(),
        &["run", "--stdin", "pipe", "--confirm=1", "cat"],
        Some("pipe-one\npipe-two\n"),
    )
    .success();

    run_astu(dir.path(), &["output", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("pipe-one"))
        .stdout(predicates::str::contains("pipe-two"));
}

#[test]
fn stdin_pipe_mode_delivers_identical_bytes_to_multiple_tasks() {
    let dir = tempdir().expect("tmpdir");
    let command = "sh -c 'while IFS= read -r line; do echo \"$line\"; sleep 0.01; done'";

    run_astu(
        dir.path(),
        &[
            "run",
            "--stdin",
            "pipe",
            "--concurrency",
            "2",
            "-T",
            "local:",
            "-T",
            "local://worker2",
            "--confirm=2",
            command,
        ],
        Some("pipe-one\npipe-two\n"),
    )
    .success();

    run_astu(dir.path(), &["freq", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("pipe-one"))
        .stdout(predicates::str::contains("pipe-two"));
}

#[test]
fn resume_runs_canceled_task() {
    let dir = tempdir().expect("tmpdir");
    let db_path = dir.path().join("astu.duckdb");
    let conn = Connection::open(&db_path).expect("open duckdb");
    init_test_schema(&conn);

    let job_id = Uuid::now_v7();
    let task_id = Uuid::now_v7();

    conn.execute(
        "INSERT INTO jobs(job_id, started_at, command, concurrency, task_count) VALUES (?, now(), 'echo resumed', 1, 1)",
        params![job_id.as_bytes().to_vec()],
    )
    .expect("insert job");

    conn.execute(
        "INSERT INTO tasks(task_id, job_id, started_at, target_uri, command, status) VALUES (?, ?, now(), 'local:', 'echo resumed', 'canceled')",
        params![task_id.as_bytes().to_vec(), job_id.as_bytes().to_vec()],
    )
    .expect("insert task");

    conn.execute(
        "INSERT INTO meta(key, value) VALUES ('last_job_id', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![job_id.as_bytes().to_vec()],
    )
    .expect("insert meta");
    drop(conn);

    run_astu(
        dir.path(),
        &["resume", "--job", &job_id.to_string(), "--confirm=1"],
        None,
    )
    .success();

    run_astu(dir.path(), &["freq", "stdout"], None)
        .success()
        .stdout(predicates::str::contains("resumed"));
}
