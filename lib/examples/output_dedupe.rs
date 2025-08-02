use std::collections::HashMap;
use std::ffi::OsStr;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use regex::Regex;
use tabled::Table;
use tabled::Tabled;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;

#[derive(Default)]
struct TokenizerContext {
    pub hostname: Option<String>,
    pub pid: Option<String>,
    pub user: Option<String>,
    pub cwd: Option<String>,
    pub custom: HashMap<String, String>,
}

impl TokenizerContext {
    fn new() -> Self {
        Self::default()
    }

    fn with_hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }

    fn with_pid(mut self, pid: String) -> Self {
        self.pid = Some(pid);
        self
    }

    fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    fn with_cwd(mut self, cwd: String) -> Self {
        self.cwd = Some(cwd);
        self
    }

    fn with_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }
}

struct OutputTokenizer {
    ipv4_regex: Regex,
    ipv6_regex: Regex,
    mac_regex: Regex,
    hostname_regex: Regex,
    timestamp_regex: Regex,
    uuid_regex: Regex,
    number_regex: Regex,
}

impl OutputTokenizer {
    fn new() -> Result<Self> {
        Ok(Self {
            // IPv4 addresses
            ipv4_regex: Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b")?,
            // IPv6 addresses (simplified)
            ipv6_regex: Regex::new(
                r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b|\b(?:[0-9a-fA-F]{1,4}:)*::[0-9a-fA-F]{1,4}\b",
            )?,
            // MAC addresses (colon-delimited only: xx:xx:xx:xx:xx:xx)
            mac_regex: Regex::new(r"\b(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}\b")?,
            // Hostnames and FQDNs
            hostname_regex: Regex::new(
                r"\b[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.[a-zA-Z]{2,}\b",
            )?,
            // Timestamps (various formats)
            timestamp_regex: Regex::new(
                r"\b\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?\b|\b\d{2}:\d{2}:\d{2}\b|\b\d{4}/\d{2}/\d{2}\b",
            )?,
            // UUIDs
            uuid_regex: Regex::new(
                r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
            )?,
            // Generic numbers (ports, IDs, etc.)
            number_regex: Regex::new(r"\b\d{5,}\b")?,
        })
    }

    fn tokenize(&self, text: &str, context: Option<&TokenizerContext>) -> String {
        let mut result = text.to_string();

        // First, apply exact string matches from context (highest priority)
        if let Some(ctx) = context {
            if let Some(hostname) = &ctx.hostname {
                result = result.replace(hostname, "<HOSTNAME>");
            }
            if let Some(pid) = &ctx.pid {
                result = result.replace(pid, "<PID>");
            }
            if let Some(user) = &ctx.user {
                result = result.replace(user, "<USER>");
            }
            if let Some(cwd) = &ctx.cwd {
                result = result.replace(cwd, "<CWD>");
            }
            // Apply custom mappings
            for (key, value) in &ctx.custom {
                result = result.replace(value, &format!("<{}>", key.to_uppercase()));
            }
        }

        // Then apply regex patterns for unknown values
        result = self.uuid_regex.replace_all(&result, "<UUID>").to_string();
        result = self.ipv6_regex.replace_all(&result, "<IPV6>").to_string();
        result = self.ipv4_regex.replace_all(&result, "<IPV4>").to_string();
        result = self.mac_regex.replace_all(&result, "<MAC>").to_string();
        result = self
            .hostname_regex
            .replace_all(&result, "<HOSTNAME>")
            .to_string();
        result = self
            .timestamp_regex
            .replace_all(&result, "<TIMESTAMP>")
            .to_string();
        result = self
            .number_regex
            .replace_all(&result, "<NUMBER>")
            .to_string();

        result
    }
}

#[derive(Tabled)]
struct CommandOutput {
    stdout: String,
    stderr: String,
    count: usize,
}

async fn run_command<S, I>(
    program: S,
    args: I,
    tokenizer: Arc<OutputTokenizer>,
    frequency_map: Arc<Mutex<HashMap<(String, String), usize>>>,
) -> Result<()>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
{
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id().context("Failed to get PID")?;

    // Create tokenizer context with known values
    let mut context = TokenizerContext::new().with_pid(pid.to_string());

    // Get other context values if available
    if let Ok(hostname_output) = std::process::Command::new("hostname").output() {
        if let Ok(hostname_str) = String::from_utf8(hostname_output.stdout) {
            context = context.with_hostname(hostname_str.trim().to_string());
        }
    }

    if let Ok(user) = std::env::var("USER") {
        context = context.with_user(user);
    }

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(cwd_str) = cwd.to_str() {
            context = context.with_cwd(cwd_str.to_string());
        }
    }

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut normalized_stdout = Vec::new();
    let mut normalized_stderr = Vec::new();

    while !stdout_done || !stderr_done {
        tokio::select! {
            line = stdout_lines.next_line(), if !stdout_done => {
                match line? {
                    Some(text) => {
                        let normalized = tokenizer.tokenize(&text, Some(&context));
                        println!("stdout[{pid}]: {text} -> {normalized}");
                        normalized_stdout.push(normalized);
                    }
                    None => stdout_done = true,
                }
            }
            line = stderr_lines.next_line(), if !stderr_done => {
                match line? {
                    Some(text) => {
                        let normalized = tokenizer.tokenize(&text, Some(&context));
                        eprintln!("stderr[{pid}]: {text} -> {normalized}");
                        normalized_stderr.push(normalized);
                    }
                    None => stderr_done = true,
                }
            }
        }
    }

    let status = child.wait().await?;
    println!("status[{pid}]: {status}");

    // Aggregate normalized output and update frequency map
    let stdout_aggregated = normalized_stdout.join("\n");
    let stderr_aggregated = normalized_stderr.join("\n");

    let mut freq_map = frequency_map.lock().unwrap();
    *freq_map
        .entry((stdout_aggregated, stderr_aggregated))
        .or_insert(0) += 1;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let tokenizer = Arc::new(OutputTokenizer::new()?);
    let frequency_map = Arc::new(Mutex::new(HashMap::<(String, String), usize>::new()));

    let cmd1 = tokio::spawn(run_command(
        "bash",
        [
            "-c",
            "echo $$; hostname; whoami; pwd; echo 192.168.1.1; echo aa:bb:cc:dd:ee:ff",
        ],
        tokenizer.clone(),
        frequency_map.clone(),
    ));
    let cmd2 = tokio::spawn(run_command(
        "bash",
        [
            "-c",
            "echo $$; hostname; whoami; pwd; echo 10.0.0.1; echo 11:22:33:44:55:66",
        ],
        tokenizer.clone(),
        frequency_map.clone(),
    ));

    let (r1, r2) = tokio::join!(cmd1, cmd2);
    r1??;
    r2??;

    // Display frequency table
    println!("\n=== Command Output Frequency ===");
    let freq_map = frequency_map.lock().unwrap();
    let mut outputs: Vec<CommandOutput> = freq_map
        .iter()
        .map(|((stdout, stderr), count)| CommandOutput {
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            count: *count,
        })
        .collect();

    // Sort by count descending
    outputs.sort_by(|a, b| b.count.cmp(&a.count));

    println!("{}", Table::new(outputs));

    Ok(())
}
