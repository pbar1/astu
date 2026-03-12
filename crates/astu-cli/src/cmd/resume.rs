use clap::Args;

/// Resume a previously canceled job
#[derive(Debug, Args)]
pub struct Resume {}

impl crate::Run for Resume {
    async fn run(&self) -> eyre::Result<()> {
        eyre::bail!("unimplemented")
    }
}
