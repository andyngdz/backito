//! Reading typed values out of environment variables.
//!
//! Shared by the environment config source and the environment secret source so
//! both treat a blank value the same way: absent. A variable exported empty is a
//! mistake that should fail at startup, not a value that reaches an API call.

use std::str::FromStr;

use super::ConfigError;

/// Reads `name`, treating blank as absent.
pub fn optional(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Reads `name`, failing when it is unset or blank.
pub fn required(name: &str) -> Result<String, ConfigError> {
    optional(name).ok_or_else(|| ConfigError::MissingEnvVar {
        variable: name.to_owned(),
    })
}

/// Reads `name` and parses it, or returns `default` when it is unset or blank.
pub fn parse_or<T: FromStr>(name: &str, default: T) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    match optional(name) {
        Some(value) => parse(name, &value),
        None => Ok(default),
    }
}

/// Parses an already-read value, naming the variable it came from on failure.
pub fn parse<T: FromStr>(name: &str, value: &str) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    value
        .parse()
        .map_err(|source: T::Err| ConfigError::InvalidEnvValue {
            variable: name.to_owned(),
            reason: source.to_string(),
        })
}
