use std::ffi::OsString;

#[cfg(not(test))]
mod real;
#[cfg(not(test))]
pub use real::*;

#[cfg(test)]
mod fake;
#[cfg(test)]
mod fake_tests;
#[cfg(test)]
pub use fake::*;

#[cfg(test)]
#[track_caller]
/// Panics if `pieces` is empty.
pub fn was_command_executed(pieces: &[&str]) -> bool {
    use std::ffi::OsStr;

    let (program_os, args_os) = pieces.split_first().unwrap();

    get_executed_commands()
        .iter()
        .any(|cmd| cmd.program == OsStr::new(program_os) && cmd.args == args_os)
}

#[cfg(not(test))]
#[track_caller]
/// Panics if `pieces` is empty.
pub fn was_command_executed(_pieces: &[&str]) -> bool {
    panic!("called outside of `cfg(test)` context");
}

pub fn mock<S: Into<OsString>>(program: S) -> CommandMockBuilder {
    CommandMockBuilder::new(program)
}

pub struct CommandMockBuilder {
    #[allow(unused)] // actually used when `cfg(test)`
    program: OsString,
    args: Vec<OsString>,
    exit_status: Option<i32>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl CommandMockBuilder {
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

    #[cfg(test)]
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

    #[cfg(not(test))]
    /// Consumes the builder, returning a `MockDefinition`.
    pub fn build(self) -> MockDefinition {
        MockDefinition {}
    }
}
