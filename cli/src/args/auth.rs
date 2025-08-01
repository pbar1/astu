use camino::Utf8PathBuf;
use clap::Args;

const HEADING: Option<&str> = Some("Authentication Options");

/// Arguments for action execution.
#[derive(Debug, Args, Clone)]
pub struct AuthArgs {
    /// Remote user to authenticate as.
    #[arg(short = 'u', long, default_value = "root", help_heading = HEADING)]
    pub user: String,

    /// Path to password file.
    #[arg(long, help_heading = HEADING)]
    pub password_file: Option<Utf8PathBuf>,

    /// Path to SSH agent socket.
    #[arg(long, env = "SSH_AUTH_SOCK", help_heading = HEADING)]
    pub ssh_agent: Option<Utf8PathBuf>,

    /// Path to SSH credential file.
    #[arg(long,  help_heading = HEADING)]
    pub ssh_key: Option<Utf8PathBuf>,

    /// Path to kubeconfig file.
    #[arg(long, env = "KUBECONFIG", default_value = "~/.kube/config", help_heading = HEADING)]
    pub kubeconfig: Option<Utf8PathBuf>,
}

impl AuthArgs {}
