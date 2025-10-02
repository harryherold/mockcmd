//! # mockcmd
//!
//! A lightweight mocking library for command execution in Rust.
//!
//! This crate provides a drop-in replacement for `std::process::Command` with added mocking
//! capabilities for tests. It offers a clean, ergonomic API for mocking command execution
//! while maintaining the same interface as the standard library.
//!
//! ## Key Features
//!
//! - Direct replacement for `std::process::Command` with identical API
//! - Mock command execution with specific arguments
//! - Set custom exit codes, stdout, and stderr
//! - Verify command execution happened with specific arguments
//! - Automatically disabled outside of test mode
//!
//! ## Usage Example
//!
//! ```
//! use mockcmd::{Command, mock, was_command_executed};
//!
//! // Use repository path to set current dir
//! let path = std::env!("CARGO_MANIFEST_DIR");
//!
//! // Setup a mock for the "git" command
//! mock("git")
//!     .current_dir(path)
//!     .with_arg("status")
//!     .with_stdout("On branch main\nNothing to commit")
//!     .register();
//!
//! // Use the Command just like std::process::Command
//! let output = Command::new("git").current_dir(path).arg("status").output().unwrap();
//!
//! // The mock is used instead of executing the real command
//! assert_eq!(String::from_utf8_lossy(&output.stdout), "On branch main\nNothing to commit");
//!
//! // Verify that the command was executed with the correct arguments
//! assert!(was_command_executed(&["git", "status"], Some(path)));
//! assert!(!was_command_executed(&["git", "push"], None));
//! ```
//!
//! ## How It Works
//!
//! The library is a drop-in replacement for `std::process::Command` that uses conditional
//! compilation to provide different implementations depending on whether you're in a test context:
//!
//! - In test mode (`#[cfg(feature = "test")]`), commands are intercepted and mocked responses are returned
//! - In normal mode, commands pass through to the standard library's process module with zero overhead
//!
//! This means your production code can use the same `Command` interface without any behavior changes
//! or performance impact.
//!
//! ## Setting Up Mocks
//!
//! Mocks are defined using a builder pattern, which allows for a fluent API:
//!
//! ```
//! use mockcmd::mock;
//!
//! // Create a simple mock
//! mock("program")
//!     .with_arg("arg1")
//!     .with_arg("arg2")
//!     .with_stdout("Success output")
//!     .with_stderr("Error message")
//!     .with_status(0)  // Exit code
//!     .register();
//! ```
//!
//! ## Migrating from std::process::Command
//!
//! Migration is as simple as changing your import statement:
//!
//! ```diff
//! - use std::process::Command;
//! + use mockcmd::Command;
//! ```
//!
//! Your existing code will continue to work exactly as before, but now you can add mocks in your tests.

use std::ffi::OsString;

#[cfg(not(feature = "test"))]
mod real;
#[cfg(not(feature = "test"))]
pub use real::*;

#[cfg(feature = "test")]
mod fake;
#[cfg(feature = "test")]
mod fake_tests;
#[cfg(feature = "test")]
pub use fake::*;

#[cfg(feature = "test")]
#[track_caller]
/// Panics if `pieces` is empty.
pub fn was_command_executed(pieces: &[&str], current_dir: Option<&str>) -> bool {
    use std::ffi::OsStr;

    let (program_os, args_os) = pieces.split_first().unwrap();

    let dir = current_dir.map(|s| OsString::from(s));

    get_executed_commands().iter().any(|cmd| {
        cmd.program == OsStr::new(program_os) && cmd.args == args_os && cmd.current_dir == dir
    })
}

#[cfg(not(feature = "test"))]
#[track_caller]
/// Panics if `pieces` is empty.
pub fn was_command_executed(_pieces: &[&str], _current_dir: Option<&str>) -> bool {
    panic!("called outside of `cfg(test)` context");
}

/// Creates a new command mock builder for the specified program.
///
/// This function is the entry point for defining mocked command behavior in tests.
/// It begins a builder chain that lets you configure the expected command arguments,
/// output, and exit status.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use mockcmd::mock;
///
/// // Mock a simple command with no arguments
/// mock("echo").with_stdout("hello world").register();
///
/// // Mock a command with specific arguments
/// mock("git")
///     .with_arg("status")
///     .with_stdout("On branch main\nNothing to commit")
///     .register();
///
/// // Mock a command with exit status and error output
/// mock("grep")
///     .with_arg("pattern")
///     .with_arg("file.txt")
///     .with_status(1)
///     .with_stderr("file.txt: No such file or directory")
///     .register();
/// ```
///
/// # Note
///
/// This function only has an effect when used within `#[cfg(feature = "test")]`.
/// In non-test code, mock definitions will be ignored.
pub fn mock<S: Into<OsString>>(program: S) -> CommandMockBuilder {
    CommandMockBuilder::new(program)
}

pub struct CommandMockBuilder {
    #[allow(unused)] // actually used when `cfg(test)`
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<OsString>,
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
            current_dir: None,
            exit_status: None,
            stdout: None,
            stderr: None,
        }
    }

    /// Sets the working directory for the command.
    pub fn current_dir<S: Into<OsString>>(mut self, dir: S) -> Self {
        self.current_dir = Some(dir.into());
        self
    }

    /// Sets the command arguments.
    ///
    /// This replaces any existing arguments with the provided ones.
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Adds a single argument to the command.
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

    #[cfg(feature = "test")]
    /// Consumes the builder, returning a `MockDefinition`.
    pub fn build(self) -> MockDefinition {
        MockDefinition {
            program: self.program,
            args: self.args,
            current_dir: self.current_dir,
            exit_status: self.exit_status,
            stdout: self.stdout,
            stderr: self.stderr,
        }
    }

    #[cfg(not(feature = "test"))]
    /// Consumes the builder, returning a `MockDefinition`.
    pub fn build(self) -> MockDefinition {
        MockDefinition {}
    }
}
