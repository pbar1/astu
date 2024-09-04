use std::collections::BTreeSet;
use std::pin::Pin;

use anyhow::Result;
use clap::Args;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use kush_resolve::ForwardResolveChain;
use kush_resolve::Resolve;
use kush_resolve::ReverseResolveChain;
use kush_resolve::Target;

const HEADING: Option<&str> = Some("Target Resolution Options");

/// Arguments for resolving targets.
#[derive(Debug, Args)]
pub struct ResolutionArgs {
    /// Target query. Reads from stdin by default.
    #[arg(short = 'T', long, default_value = "-", help_heading = HEADING)]
    pub targets: Vec<Target>,

    /// Perform reverse resolution instead of forward.
    #[arg(short = 'r', long, help_heading = HEADING)]
    pub reverse: bool,

    /// Do not drop unknown targets.
    #[arg(long, help_heading = HEADING)]
    pub allow_unknown: bool,
}

impl ResolutionArgs {
    fn resolve(self) -> Result<DynTargetStream> {
        // match self.reverse {
        //     false => self.resolve_forward(),
        //     true => self.resolve_reverse(),
        // }
        self.resolve_forward()
    }

    fn resolve_forward(self) -> Result<DynTargetStream> {
        let resolvers = ForwardResolveChain::try_default()?;

        let initial_targets = futures::stream::iter(self.targets);

        Ok(initial_targets.boxed())
    }

    fn resolve_reverse(self) -> Result<DynTargetStream> {
        let resolvers = ReverseResolveChain::try_default()?;

        let initial_targets = futures::stream::iter(self.targets);

        Ok(initial_targets.boxed())
    }
}

pub type DynTargetStream = Pin<Box<dyn Stream<Item = Target>>>;
