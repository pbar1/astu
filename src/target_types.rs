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

#[derive(PartialEq, Eq, Debug)]
pub enum TargetType {
    Kubernetes,
    Docker,
    Ssh,
    Aws,
}

/// Makes an educated guess about what the target type should be, given a
/// target string.
pub fn infer_target_type(s: &str) -> Option<TargetType> {
    // FIXME: No unwrap
    let re_pod = regex::Regex::new(r"-[a-z0-9]{10}-[a-z0-9]{5}$").unwrap();

    if s.starts_with("arn:") {
        Some(TargetType::Aws)
    } else if s.starts_with("i-") {
        Some(TargetType::Aws)
    } else if re_pod.is_match(s) {
        Some(TargetType::Kubernetes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "arn:aws:ec2:us-west-2:012345678901:instance/i-0b22a22eec53b9321",
        Some(TargetType::Aws)
    )]
    #[case("i-0b22a22eec53b9321", Some(TargetType::Aws))]
    #[case("coredns-59b4f5bbd5-24gpc", Some(TargetType::Kubernetes))]
    fn test_infer_target_type(#[case] input: &str, #[case] expected: Option<TargetType>) {
        assert_eq!(expected, infer_target_type(input));
    }
}
