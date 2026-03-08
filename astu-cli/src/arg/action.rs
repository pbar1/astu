use clap::Args;
use clap::ValueEnum;

#[derive(Debug, Clone, Default, Args)]
pub struct ActionFlags {
    /// Target URI
    ///
    /// If not passed, defaults to `local:`. May be passed multiple times.
    #[arg(
        short = 'T',
        long,
        value_name = "TARGET",
        help_heading = "Action Flags"
    )]
    pub target: Vec<String>,

    /// Path to a file to read target URIs from.
    ///
    /// Can use `-` to read from stdin. If this is set, then `--stdin` is
    /// assumed to be `target`.
    #[arg(short = 'f', long, value_name = "PATH", help_heading = "Action Flags")]
    pub target_file: Vec<String>,

    /// How to interpret stdin.
    ///
    /// Automatically detected if not explicitly set:
    /// - If `--target-file` is stdin: `target`
    /// - If `{param}` in command template: `param`
    /// - Else: `pipe`
    #[arg(
        long,
        value_name = "MODE",
        verbatim_doc_comment,
        help_heading = "Action Flags"
    )]
    pub stdin: Option<StdinMode>,

    /// Per-task timeout value in humantime. `0` indicates no timeout.
    #[arg(
        long,
        default_value = "30s",
        value_name = "DURATION",
        help_heading = "Action Flags"
    )]
    pub timeout: String,

    /// Auto-accept the plan if passed target count is correct.
    #[arg(long, value_name = "COUNT", help_heading = "Action Flags")]
    pub confirm: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum StdinMode {
    /// Allow `--target-file` to use stdin.
    Target,

    /// Split stdin into tokens based on whitespace (like `xargs`).
    Param,

    /// Multiplex stdin to tasks.
    #[default]
    Pipe,
}
