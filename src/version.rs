// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! A string that provides version information about this crate. Based on the
//! crate version, or a version-control identifier if this is a development
//! build.

// https://stackoverflow.com/a/65972328/344467
/// A string that's useful for displaying build information to end users.
pub fn app_version() -> &'static str {
    option_env!("GIT_DESCRIBE")
        .unwrap_or(option_env!("GIT_REV_PARSE").unwrap_or(env!("CARGO_PKG_VERSION")))
}
