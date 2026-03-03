use anyhow::Result;
use astu::util::id::Id;
use clap::Args;

use crate::cmd::Run;
use crate::runtime::Runtime;

/// Resolve targets.
#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[clap(flatten)]
    resolution_args: crate::args::ResolutionArgs,
}

impl Run for ResolveArgs {
    async fn run(&self, _id: Id, runtime: &Runtime) -> Result<()> {
        let targets = self.resolution_args.set_with_default(None).await?;
        if matches!(runtime.output(), crate::args::OutputFormat::Json) {
            let values = targets
                .into_iter()
                .map(|target| target.to_string())
                .collect::<Vec<_>>();
            let rendered = format!("{}\n", serde_json::to_string_pretty(&values)?);
            crate::cmd::render::emit_with_optional_pager(&rendered, true)?;
            return Ok(());
        }
        for target in targets {
            crate::ui::out_line(&target.to_string())?;
        }

        Ok(())
    }
}
