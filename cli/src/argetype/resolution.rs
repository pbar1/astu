use astu_resolve::ForwardChainResolver;
use astu_resolve::ResolveExt;
use astu_resolve::ReverseChainResolver;
use astu_resolve::Target;
use astu_resolve::TargetStream;
use clap::Args;

const HEADING: Option<&str> = Some("Target Resolution Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ResolutionArgs {
    /// Target query. Pass `-` to read from stdin.
    pub target: Target,

    /// Perform reverse resolution instead of forward.
    #[arg(long, help_heading = HEADING)]
    pub reverse: bool,
}

impl ResolutionArgs {
    pub fn resolve(self) -> TargetStream {
        let target = self.target.clone();
        match self.reverse {
            true => ReverseChainResolver::new().resolve_infallible(target),
            false => ForwardChainResolver::new().resolve_infallible(target),
        }
    }
}
