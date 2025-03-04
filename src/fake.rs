use std::io;
use std::process;

use std::ffi::{OsStr, OsString};
use std::sync::Mutex;

use crate::CommandMockBuilder;

type Result<T = ()> = io::Result<T>;

/// Stores for mock definitions
static MOCK_REGISTRY: Mutex<Vec<MockDefinition>> = Mutex::new(Vec::new());
/// Stores executed commands
static EXECUTED_COMMANDS: Mutex<Vec<ExecutedCommand>> = Mutex::new(Vec::new());

pub fn find_mock(program: &OsString, args: &[OsString]) -> Option<MockDefinition> {
    let mocks = MOCK_REGISTRY.lock().unwrap();
    for mock in mocks.iter() {
        if &mock.program == program && mock.args == args {
            return Some(mock.clone());
        }
    }
    None
}

/// Represents a definition for how a command should be mocked.
// Rust sees some fields as unused but aren't
#[derive(Debug, Clone)]
pub struct MockDefinition {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub exit_status: Option<i32>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}

/// A record of an executed command.
#[derive(Debug, Clone)]
pub struct ExecutedCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}

pub struct Command {
    inner: process::Command,
    program: OsString,
    args: Vec<OsString>,
}

#[cfg(test)]
impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let prog = program.as_ref().to_os_string();
        Command {
            inner: process::Command::new(&prog),
            program: prog,
            args: Vec::new(),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        let arg_os = arg.as_ref().to_os_string();
        self.args.push(arg_os.clone());
        self.inner.arg(arg_os);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.arg(a);
        }
        self
    }

    pub fn output(&mut self) -> Result<process::Output> {
        // Record this command invocation
        record_executed_command(&self.program, &self.args);

        let (exit_status, stdout, stderr) = if let Some(mock) = find_mock(&self.program, &self.args)
        {
            let exit_status = mock.exit_status.unwrap_or(0);

            (
                exit_status,
                mock.stdout.unwrap_or_default(),
                mock.stderr.unwrap_or_default(),
            )
        } else {
            (0, "".into(), "".into())
        };

        Ok(process::Output {
            status: exit_code(exit_status),
            stdout,
            stderr,
        })
    }
}

fn exit_code(code: i32) -> process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        process::ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        process::ExitStatus::from_raw(code)
    }
}

fn record_executed_command<I, S>(program: &OsStr, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd = ExecutedCommand {
        program: program.to_owned(),
        args: args
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect(),
    };
    EXECUTED_COMMANDS.lock().unwrap().push(cmd);
}

#[cfg(test)]
/// Returns a copy of all executed commands.
pub fn get_executed_commands() -> Vec<ExecutedCommand> {
    EXECUTED_COMMANDS.lock().unwrap().clone()
}

impl CommandMockBuilder {
    pub fn register(self) {
        MOCK_REGISTRY.lock().unwrap().push(self.build());
    }
}
