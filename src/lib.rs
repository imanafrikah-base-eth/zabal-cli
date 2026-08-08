//! LoopHouse CLI. Turns Git commit history into a ZABAL hackathon submission.
//!
//! The work is split into three pieces:
//!
//! - [`parser`] reads the repository and collects commits in a time window.
//! - [`generator`] renders those commits as Markdown or JSON.
//! - [`ui`] handles terminal output and the on-disk project tracker.

pub mod generator;
pub mod parser;
pub mod ui;
