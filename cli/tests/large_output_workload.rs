use assert_cmd::Command;
use tempfile::tempdir;

#[test]
#[ignore = "workload smoke: opt-in"]
fn large_output_workload_smoke() {
    let dir = tempdir().expect("tmpdir");
    let script = "awk 'BEGIN { for (i=1; i<=20000; ++i) print \"line-\" i }'";

    Command::new(assert_cmd::cargo::cargo_bin!("astu"))
        .env("ASTU_DATA_DIR", dir.path())
        .args(["run", "-T", "local:", "--confirm=1", script])
        .assert()
        .success();

    Command::new(assert_cmd::cargo::cargo_bin!("astu"))
        .env("ASTU_DATA_DIR", dir.path())
        .args(["freq", "stdout", "--contains", "line-19999"])
        .assert()
        .success();
}
