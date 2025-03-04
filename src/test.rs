use std::io;
use std::{fmt::Debug, process};

type Result<T = ()> = io::Result<T>;

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

use std::ffi::{OsStr, OsString};
use std::sync::Mutex;

use crate::ExecutedCommand;

/// Represents a command mock definition
#[derive(Debug, Clone)]
pub struct MockDefinition {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub exit_status: Option<i32>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}

pub struct MockDefinitionBuilder {
    program: OsString,
    args: Vec<OsString>,
    exit_status: Option<i32>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl MockDefinitionBuilder {
    /// Creates a new builder with the specified program.
    pub fn new<S: Into<OsString>>(program: S) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            exit_status: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_arg<S>(mut self, arg: S) -> Self
    where
        S: Into<OsString>,
    {
        self.args.push(arg.into());
        self
    }

    /// Sets the expected exit status.
    pub fn with_status(mut self, status: i32) -> Self {
        self.exit_status = Some(status);
        self
    }

    /// Sets the expected stdout.
    pub fn with_stdout<S: Into<Vec<u8>>>(mut self, stdout: S) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    /// Sets the expected stderr.
    pub fn with_stderr<S: Into<Vec<u8>>>(mut self, stderr: S) -> Self {
        self.stderr = Some(stderr.into());
        self
    }

    /// Consumes the builder, returning a `MockDefinition`.
    pub fn build(self) -> MockDefinition {
        MockDefinition {
            program: self.program,
            args: self.args,
            exit_status: self.exit_status,
            stdout: self.stdout,
            stderr: self.stderr,
        }
    }

    pub fn register(self) {
        register_mock(self.build());
    }
}

/// Stores for mock definitions
static MOCK_REGISTRY: Mutex<Vec<MockDefinition>> = Mutex::new(Vec::new());
/// Stores executed commands
static EXECUTED_COMMANDS: Mutex<Vec<ExecutedCommand>> = Mutex::new(Vec::new());

pub fn register_mock(mock: MockDefinition) {
    MOCK_REGISTRY.lock().unwrap().push(mock);
}

pub fn record_executed_command<I, S>(program: &OsStr, args: I)
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

/// Returns a copy of all executed commands.
pub fn get_executed_commands() -> Vec<ExecutedCommand> {
    EXECUTED_COMMANDS.lock().unwrap().clone()
}

pub fn find_mock(program: &OsString, args: &[OsString]) -> Option<MockDefinition> {
    let mocks = MOCK_REGISTRY.lock().unwrap();
    for mock in mocks.iter() {
        if &mock.program == program && mock.args == args {
            return Some(mock.clone());
        }
    }
    None
}

pub fn was_command_executed<P, A>(program: P, args: &[A]) -> bool
where
    P: Into<OsString>,
    A: Into<OsString> + Clone,
{
    let program_os = program.into();
    let args_os: Vec<OsString> = args.iter().cloned().map(Into::into).collect();
    get_executed_commands()
        .iter()
        .any(|cmd| cmd.program == program_os && cmd.args == args_os)
}

mod tests {
    use crate::{register_mock, was_command_executed, Command, MockDefinitionBuilder};

    #[test]
    fn using_test_code_unmocked() {
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        let output = cmd.output().unwrap();
        assert_eq!(output.status.success(), true);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());

        assert!(was_command_executed("echo", &["hello"]));
        assert_eq!(was_command_executed("echo", &["world"]), false);
    }

    #[test]
    fn using_test_code_mocked() {
        MockDefinitionBuilder::new("echo")
            .with_arg("world")
            .with_status(1)
            .with_stdout("WORLD")
            .with_stderr("failed")
            .register();

        let mut cmd = Command::new("echo");
        cmd.arg("world");
        let output = cmd.output().unwrap();
        assert_eq!(output.status.success(), false);
        assert_eq!(output.stdout, b"WORLD");
        assert_eq!(output.stderr, b"failed");

        assert!(was_command_executed("echo", &["world"]));
    }
}
