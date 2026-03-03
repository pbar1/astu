use astu::db::DbTaskStatus;
use astu::db::DuckDb;
use tempfile::tempdir;

#[tokio::test]
async fn task_vars_roundtrip() {
    let dir = tempdir().expect("tmpdir");
    let db_path = dir.path().join("astu.duckdb");
    let db = DuckDb::try_new(db_path.to_string_lossy().as_ref())
        .await
        .expect("db");

    let job_id = "33333333-3333-4333-8333-333333333333";
    let task_id = "44444444-4444-4444-8444-444444444444";

    db.create_job(job_id, "echo {param}", 1, 1)
        .await
        .expect("create job");
    db.create_task(task_id, job_id, "local:", "echo alpha")
        .await
        .expect("create task");
    db.append_task_var(task_id, "{param}", "alpha")
        .await
        .expect("append task var");
    db.finish_task(task_id, DbTaskStatus::Complete, Some(0), None, 0, 0, 1)
        .await
        .expect("finish task");
    db.finish_job(job_id).await.expect("finish job");

    let vars = db.task_vars_for_task(task_id).await.expect("task vars");
    assert_eq!(vars, vec![("{param}".to_owned(), "alpha".to_owned())]);
}
