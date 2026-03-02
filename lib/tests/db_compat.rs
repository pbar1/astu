use astu::db::Db;
use astu::db::DuckDb;
use astu::db::ResultEntry;
use tempfile::tempdir;

#[tokio::test]
async fn load_includes_stderr_only_and_no_output_tasks() {
    let dir = tempdir().expect("tmpdir");
    let db_path = dir.path().join("astu.duckdb");
    let db = DuckDb::try_new(db_path.to_string_lossy().as_ref())
        .await
        .expect("db");

    let job_id = "11111111-1111-4111-8111-111111111111";
    db.save(&ResultEntry {
        job_id: job_id.to_owned(),
        target: "dummy://stderr-only".to_owned(),
        error: None,
        exit_status: Some(0),
        stdout: None,
        stderr: Some(b"only-stderr".to_vec()),
    })
    .await
    .expect("save stderr-only");

    db.save(&ResultEntry {
        job_id: job_id.to_owned(),
        target: "dummy://no-output".to_owned(),
        error: None,
        exit_status: Some(0),
        stdout: None,
        stderr: None,
    })
    .await
    .expect("save no-output");

    let rows = db.load(job_id).await.expect("load");
    assert_eq!(rows.len(), 2, "load dropped tasks");
    assert!(rows.iter().any(|x| x.target == "dummy://stderr-only"));
    assert!(rows.iter().any(|x| x.target == "dummy://no-output"));
}

#[tokio::test]
async fn load_compatibility_non_utf8_is_lossy() {
    let dir = tempdir().expect("tmpdir");
    let db_path = dir.path().join("astu.duckdb");
    let db = DuckDb::try_new(db_path.to_string_lossy().as_ref())
        .await
        .expect("db");

    let job_id = "22222222-2222-4222-8222-222222222222";
    let raw = vec![0xff, 0xfe, b'A'];
    db.save(&ResultEntry {
        job_id: job_id.to_owned(),
        target: "dummy://non-utf8".to_owned(),
        error: None,
        exit_status: Some(0),
        stdout: Some(raw.clone()),
        stderr: None,
    })
    .await
    .expect("save");

    let rows = db.load(job_id).await.expect("load");
    assert_eq!(rows.len(), 1);
    let loaded = rows[0].stdout.clone().expect("stdout");
    let expected = String::from_utf8_lossy(&raw).to_string().into_bytes();
    assert_eq!(loaded, expected, "compat stream contract changed");
}
