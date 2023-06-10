pub mod docker;
pub mod k8s;

use anyhow::bail;
use anyhow::Result;

pub trait InteractiveShell {
    fn interactive_shell(&self) -> Result<()>;
}

pub fn dispatch_interactive_shell(s: &str) -> Result<()> {
    let provider: Box<dyn InteractiveShell> = match s {
        "kubernetes" | "kube" | "k8s" | "k" => Box::<self::k8s::K8sExec>::default(),
        "docker" | "d" => Box::<self::docker::DockerExec>::default(),
        _ => bail!("Unsupported interactive shell type: {s}"),
    };

    provider.interactive_shell()
}
