# `mockcmd`

An ergonomic drop-in replacement for `std::process::Command` with added mocking capabilities for tests.

> ⚠️ **Warning**: This crate is still in early development. The API will change significantly in future releases.

## Key Features

- Direct replacement for `std::process::Command` with identical API
- Mock command execution with specific arguments
- Set custom exit codes, stdout, and stderr
- Verify command execution happened with specific arguments
- Automatically disabled outside of test mode (zero overhead in production)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mockcmd = "*"

[dev-dependencies]
mockcmd = { version = "*", features = ["test"] }
```

## Usage Example

```rust
use mockcmd::{Command, mock, was_command_executed};

let path = std::env!("CARGO_MANIFEST_DIR");

// Setup a mock for the "git" command
mock("git")
    .current_dir(path)
    .with_arg("status")
    .with_stdout("On branch main\nNothing to commit")
    .register();

// Use the Command just like std::process::Command
let output = Command::new("git").current_dir(path).arg("status").output().unwrap();

// The mock is used instead of executing the real command
assert_eq!(String::from_utf8_lossy(&output.stdout), "On branch main\nNothing to commit");

// Verify that the command was executed with the correct arguments
assert!(was_command_executed(&["git", "status"], Some(path)));
assert!(!was_command_executed(&["git", "push"], None));
```

## How It Works

The library uses conditional compilation to provide different implementations:

- In test mode (`#[cfg(feature = "test")]`), commands are intercepted and mocked responses are returned
- In normal mode, commands pass through to the standard library's process module with zero overhead

This means your production code can use the same `Command` interface without any behavior changes or performance impact.

## Setting Up Mocks

Mocks are defined using a builder pattern, which allows for a fluent API:

```rust
use mockcmd::mock;

// Create a simple mock
mock("program")
    .with_arg("arg1")
    .with_arg("arg2")
    .with_stdout("Success output")
    .with_stderr("Error message")
    .with_status(0)  // Exit code
    .register();
```

## Migrating from std::process::Command

Migration is as simple as changing your import statement:

```diff
- use std::process::Command;
+ use mockcmd::Command;
```

Your existing code will continue to work exactly as before, but now you can add mocks in your tests.