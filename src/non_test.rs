use std::{ffi::OsStr, process};

type Result<T = (), E = std::io::Error> = std::result::Result<T, E>;

#[cfg(not(test))]
pub struct Command(process::Command);

#[cfg(not(test))]
impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self(process::Command::new(program))
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.0.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.0.args(args);
        self
    }

    pub fn current_dir<P: AsRef<OsStr>>(&mut self, dir: P) -> &mut Self {
        self.0.current_dir(dir.as_ref());
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.0.env(key, val);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.0.env_clear();
        self
    }

    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.0.env_remove(key);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.0.envs(vars);
        self
    }

    pub fn get_args(&self) -> std::process::CommandArgs<'_> {
        self.0.get_args()
    }

    pub fn get_current_dir(&self) -> Option<&std::path::Path> {
        self.0.get_current_dir()
    }

    pub fn get_envs(&self) -> std::process::CommandEnvs<'_> {
        self.0.get_envs()
    }

    pub fn get_program(&self) -> &OsStr {
        self.0.get_program()
    }

    pub fn output(&mut self) -> Result<std::process::Output> {
        self.0.output()
    }

    pub fn spawn(&mut self) -> Result<std::process::Child> {
        self.0.spawn()
    }

    pub fn status(&mut self) -> Result<std::process::ExitStatus> {
        self.0.status()
    }

    pub fn stderr<T: Into<std::process::Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.0.stderr(cfg);
        self
    }

    pub fn stdin<T: Into<std::process::Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.0.stdin(cfg);
        self
    }

    pub fn stdout<T: Into<std::process::Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.0.stdout(cfg);
        self
    }
}
