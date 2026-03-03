use assert_cmd::Command;
use std::io::Write;
use std::process::Stdio;
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

#[test]
#[ignore = "workload stress: opt-in"]
fn stdin_pipe_auto_mode_stress_streams_without_rss_blowup() {
    let dir = tempdir().expect("tmpdir");
    let total_bytes = std::env::var("ASTU_STDIN_STRESS_BYTES")
        .ok()
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(256 * 1024 * 1024);
    let rss_limit_mb = std::env::var("ASTU_STDIN_STRESS_RSS_MB_MAX")
        .ok()
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(256);
    let targets = std::env::var("ASTU_STDIN_STRESS_TARGETS")
        .ok()
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(3);
    assert!(targets >= 1, "target count must be >= 1");

    let mut run_args = vec!["run".to_owned()];
    for idx in 0..targets {
        run_args.push("-T".to_owned());
        if idx == 0 {
            run_args.push("local:".to_owned());
        } else {
            run_args.push(format!("local://worker{}", idx + 1));
        }
    }
    run_args.push(format!("--confirm={targets}"));
    run_args.push("cat | cksum".to_owned());

    let mut run = std::process::Command::new(assert_cmd::cargo::cargo_bin!("astu"))
        .env("ASTU_DATA_DIR", dir.path())
        .args(run_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn astu run");

    {
        let mut stdin = run.stdin.take().expect("stdin");
        let chunk = vec![b'a'; 1024 * 1024];
        let mut remaining = total_bytes;
        while remaining > 0 {
            let n = remaining.min(chunk.len());
            stdin.write_all(&chunk[..n]).expect("write stdin chunk");
            remaining -= n;
        }
    }

    let mut max_rss_kb = 0usize;
    loop {
        if run.try_wait().expect("try_wait").is_some() {
            break;
        }
        if let Some(rss) = sample_rss_kb(run.id()) {
            max_rss_kb = max_rss_kb.max(rss);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let output = run.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let max_rss_mb = max_rss_kb / 1024;
    assert!(
        max_rss_mb <= rss_limit_mb,
        "RSS exceeded limit: {} MiB > {} MiB",
        max_rss_mb,
        rss_limit_mb
    );

    let freq = std::process::Command::new(assert_cmd::cargo::cargo_bin!("astu"))
        .env("ASTU_DATA_DIR", dir.path())
        .args(["--output=json", "freq", "stdout"])
        .output()
        .expect("run freq");
    assert!(
        freq.status.success(),
        "freq failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&freq.stdout),
        String::from_utf8_lossy(&freq.stderr)
    );

    let value: serde_json::Value = serde_json::from_slice(&freq.stdout).expect("json freq");
    let rows = value
        .get("stdout")
        .and_then(serde_json::Value::as_array)
        .expect("stdout rows");
    let expected_count = i64::try_from(targets).expect("target count");
    let found = rows.iter().any(|row| {
        let count = row
            .get("count")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default();
        let v = row
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        count == expected_count && v.ends_with(&format!(" {total_bytes}"))
    });
    assert!(found, "missing expected cksum row for {} bytes", total_bytes);
}

fn sample_rss_kb(pid: u32) -> Option<usize> {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.trim().parse::<usize>().ok()
}
