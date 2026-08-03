//! Progress reporting: the steps a command runs, and how work announces itself.

mod domain;
mod traits;

pub use domain::Step;
#[cfg(test)]
pub use traits::SilentObserver;
pub use traits::{MeteredReader, ProgressObserver};
