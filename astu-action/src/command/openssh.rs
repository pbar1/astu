use std::ffi::OsStr;
use std::ops::Deref;
use std::path::Path;
use std::process;

use eyre::Result;
use openssh::OverSsh;
use openssh::Session;

use crate::command;

#[derive(Debug)]
pub struct OpenSshCommand<S> {
    session: S,
    inner: process::Command,
}

impl<S> OpenSshCommand<S>
where
    S: Deref<Target = Session> + Clone,
{
    #[must_use]
    pub fn new(session: S, program: impl AsRef<OsStr>) -> Self {
        Self {
            session,
            inner: process::Command::new(program),
        }
    }
}

impl<S> command::Command for OpenSshCommand<S>
where
    S: Deref<Target = Session> + Clone,
{
    type Child = OpenSshChild<S>;

    fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    fn args<I, A>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    fn env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> &mut Self {
        self.inner.env(key, val);
        self
    }

    fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    fn env_remove(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.inner.env_remove(key);
        self
    }

    fn env_clear(&mut self) -> &mut Self {
        self.inner.env_clear();
        self
    }

    fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    fn stdin(&mut self, cfg: process::Stdio) -> &mut Self {
        self.inner.stdin(cfg);
        self
    }

    fn stdout(&mut self, cfg: process::Stdio) -> &mut Self {
        self.inner.stdout(cfg);
        self
    }

    fn stderr(&mut self, cfg: process::Stdio) -> &mut Self {
        self.inner.stderr(cfg);
        self
    }

    fn get_program(&self) -> &OsStr {
        self.inner.get_program()
    }

    fn get_args(&self) -> impl Iterator<Item = &OsStr> {
        self.inner.get_args()
    }

    fn get_envs(&self) -> impl Iterator<Item = (&OsStr, Option<&OsStr>)> {
        self.inner.get_envs()
    }

    fn get_current_dir(&self) -> Option<&Path> {
        self.inner.get_current_dir()
    }

    async fn spawn(&mut self) -> Result<Self::Child> {
        let mut command = self.inner.over_ssh(self.session.clone())?;
        let child = command.spawn().await?;
        Ok(OpenSshChild { inner: child })
    }
}

#[derive(Debug)]
pub struct OpenSshChild<S> {
    inner: openssh::Child<S>,
}

impl<S> command::Child for OpenSshChild<S> {
    async fn wait(self) -> Result<process::ExitStatus> {
        Ok(self.inner.wait().await?)
    }

    async fn wait_with_output(self) -> Result<process::Output> {
        Ok(self.inner.wait_with_output().await?)
    }
}
