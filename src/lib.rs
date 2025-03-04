use std::ffi::OsString;

#[cfg(not(test))]
mod non_test;

#[cfg(not(test))]
pub use non_test::*;

#[cfg(test)]
mod test;

#[cfg(test)]
pub use test::*;

/// A record of an executed command
#[derive(Debug, Clone)]
pub struct ExecutedCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}
