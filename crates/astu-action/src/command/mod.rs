use std::ffi::OsStr;
use std::path::Path;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;

use eyre::Result;

pub mod mock;
pub mod openssh;

pub trait Child: Sized {
    async fn wait(self) -> Result<ExitStatus>;
    async fn wait_with_output(self) -> Result<Output>;
}

pub trait Command: Sized {
    type Child: Child;

    fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self;

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> &mut Self;

    fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>;

    fn env_remove(&mut self, key: impl AsRef<OsStr>) -> &mut Self;
    fn env_clear(&mut self) -> &mut Self;
    fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self;
    fn stdin(&mut self, cfg: Stdio) -> &mut Self;
    fn stdout(&mut self, cfg: Stdio) -> &mut Self;
    fn stderr(&mut self, cfg: Stdio) -> &mut Self;

    fn get_program(&self) -> &OsStr;
    fn get_args(&self) -> impl Iterator<Item = &OsStr>;
    fn get_envs(&self) -> impl Iterator<Item = (&OsStr, Option<&OsStr>)>;
    fn get_current_dir(&self) -> Option<&Path>;

    async fn spawn(&mut self) -> Result<Self::Child>;

    async fn status(&mut self) -> Result<ExitStatus> {
        self.spawn().await?.wait().await
    }

    async fn output(&mut self) -> Result<Output> {
        self.spawn().await?.wait_with_output().await
    }
}
