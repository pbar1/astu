use clap::Args;
use kush_resolve::ForwardChainResolver;
use kush_resolve::ResolveExt;
use kush_resolve::ReverseChainResolver;
use kush_resolve::Target;
use kush_resolve::TargetStream;

const HEADING: Option<&str> = Some("Target Resolution Options");

/// Arguments for resolving targets.
#[derive(Debug, Args, Clone)]
pub struct ResolutionArgs {
    /// Target query. Reads from stdin by default.
    #[arg(short = 'T', long, default_value = "-", help_heading = HEADING)]
    pub target: Target,

    /// Perform reverse resolution instead of forward.
    #[arg(short = 'r', long, help_heading = HEADING)]
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