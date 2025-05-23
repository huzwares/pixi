//! Contains builders for the CLI commands
//! We are using a builder pattern here to make it easier to write tests.
//! And are kinda abusing the `IntoFuture` trait to make it easier to execute as close
//! as we can get to the command line args
//!
//! # Using IntoFuture
//!
//! When `.await` is called on an object that is not a `Future` the compiler will first check if the
//! type implements `IntoFuture`. If it does it will call the `IntoFuture::into_future()` method and
//! await the resulting `Future`. We can abuse this behavior in builder patterns because the
//! `into_future` method can also be used as a `finish` function. This allows you to reduce the
//! required code.
//!
//! ```rust
//! impl IntoFuture for InitBuilder {
//!     type Output = miette::Result<()>;
//!     type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;
//!
//!     fn into_future(self) -> Self::IntoFuture {
//!         Box::pin(init::execute(self.args))
//!     }
//! }
//!
//! ```

use crate::common::IntoMatchSpec;
use pixi::cli::{add, init, task};
use pixi::project::SpecType;
use rattler_conda_types::Platform;
use std::future::{Future, IntoFuture};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use url::Url;

/// Strings from an iterator
pub fn string_from_iter(iter: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    iter.into_iter().map(|s| s.as_ref().to_string()).collect()
}

/// Contains the arguments to pass to `init::execute()`. Call `.await` to call the CLI execute
/// method and await the result at the same time.
pub struct InitBuilder {
    pub args: init::Args,
}

impl InitBuilder {
    pub fn with_channel(mut self, channel: impl ToString) -> Self {
        self.args.channels.push(channel.to_string());
        self
    }

    pub fn with_local_channel(self, channel: impl AsRef<Path>) -> Self {
        self.with_channel(Url::from_directory_path(channel).unwrap())
    }
}

impl IntoFuture for InitBuilder {
    type Output = miette::Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(init::execute(self.args))
    }
}

/// Contains the arguments to pass to `add::execute()`. Call `.await` to call the CLI execute method
/// and await the result at the same time.
pub struct AddBuilder {
    pub args: add::Args,
}

impl AddBuilder {
    pub fn with_spec(mut self, spec: impl IntoMatchSpec) -> Self {
        self.args.specs.push(spec.into());
        self
    }

    /// Set as a host
    pub fn set_type(mut self, t: SpecType) -> Self {
        match t {
            SpecType::Host => {
                self.args.host = true;
                self.args.build = false;
            }
            SpecType::Build => {
                self.args.host = false;
                self.args.build = true;
            }
            SpecType::Run => {
                self.args.host = false;
                self.args.build = false;
            }
        }
        self
    }

    /// Set whether or not to also install the environment. By default the environment is NOT
    /// installed to reduce test times.
    pub fn with_install(mut self, install: bool) -> Self {
        self.args.no_install = !install;
        self
    }

    pub fn set_platforms(mut self, platforms: &[Platform]) -> Self {
        self.args.platform.extend(platforms.iter());
        self
    }
}

impl IntoFuture for AddBuilder {
    type Output = miette::Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(add::execute(self.args))
    }
}

pub struct TaskAddBuilder {
    pub manifest_path: Option<PathBuf>,
    pub args: task::AddArgs,
}

impl TaskAddBuilder {
    /// Execute these commands
    pub fn with_commands(mut self, commands: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.args.commands = string_from_iter(commands);
        self
    }

    /// Depends on these commands
    pub fn with_depends_on(mut self, depends: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.args.depends_on = Some(string_from_iter(depends));
        self
    }

    /// Execute the CLI command
    pub fn execute(self) -> miette::Result<()> {
        task::execute(task::Args {
            operation: task::Operation::Add(self.args),
            manifest_path: self.manifest_path,
        })
    }
}

pub struct TaskAliasBuilder {
    pub manifest_path: Option<PathBuf>,
    pub args: task::AliasArgs,
}

impl TaskAliasBuilder {
    /// Depends on these commands
    pub fn with_depends_on(mut self, depends: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.args.depends_on = string_from_iter(depends);
        self
    }

    /// Execute the CLI command
    pub fn execute(self) -> miette::Result<()> {
        task::execute(task::Args {
            operation: task::Operation::Alias(self.args),
            manifest_path: self.manifest_path,
        })
    }
}
