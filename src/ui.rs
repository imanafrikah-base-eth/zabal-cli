//! Terminal output and the on-disk project tracker written by `zabal init`.

use crate::generator::totals;
use crate::parser::CommitInfo;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Name of the tracker file `zabal init` drops in the project root.
pub const TRACKER_FILE: &str = ".zabal.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub username: String,
    pub project: String,
    pub created_at: String,
}

pub fn tracker_path(root: &Path) -> PathBuf {
    root.join(TRACKER_FILE)
}

/// Read the tracker if one exists. A missing or malformed file is not an
/// error; the caller simply proceeds without a handle.
pub fn load_tracker(root: &Path) -> Option<Tracker> {
    let raw = fs::read_to_string(tracker_path(root)).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Create (or overwrite) the tracker for this project.
pub fn init_tracker(root: &Path, username: &str) -> Result<(Tracker, PathBuf), Box<dyn Error>> {
    let username = username.trim();
    if username.is_empty() {
        return Err("username cannot be empty".into());
    }

    let tracker = Tracker {
        username: if username.starts_with('@') {
            username.to_string()
        } else {
            format!("@{}", username)
        },
        project: project_name(root),
        created_at: Utc::now().to_rfc3339(),
    };

    let path = tracker_path(root);
    fs::write(&path, serde_json::to_string_pretty(&tracker)?)?;
    Ok((tracker, path))
}

/// Best guess at the project name: the directory the tracker lives in.
pub fn project_name(root: &Path) -> String {
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

/// Human readable summary of a scan, for `zabal preview`.
pub fn render_preview(commits: &[CommitInfo], days: u64, handle: Option<&str>) -> String {
    let t = totals(commits);
    let mut out = String::new();

    let _ = writeln!(
        out,
        "ZABAL preview: last {} day{}",
        days,
        if days == 1 { "" } else { "s" }
    );
    if let Some(handle) = handle {
        let _ = writeln!(out, "Handle: {}", handle);
    }
    let _ = writeln!(out, "{}", "-".repeat(60));

    if commits.is_empty() {
        let _ = writeln!(out, "No commits found in this window.");
        let _ = writeln!(
            out,
            "Try a wider window, for example: zabal preview --days 30"
        );
        return out;
    }

    let _ = writeln!(
        out,
        "{} commits, {} files touched, +{} / -{} lines, across {} active day{}",
        t.commits,
        t.files_changed,
        t.insertions,
        t.deletions,
        t.active_days,
        if t.active_days == 1 { "" } else { "s" }
    );
    let _ = writeln!(out, "{}", "-".repeat(60));

    let mut current_day = "";
    for c in commits {
        if c.date != current_day {
            current_day = c.date.as_str();
            let _ = writeln!(out, "\n{}", current_day);
        }
        let _ = writeln!(
            out,
            "  {}  {:<52} +{}/-{}",
            c.short_id,
            truncate(&c.summary, 52),
            c.insertions,
            c.deletions
        );
    }

    let _ = writeln!(out, "\n{}", "-".repeat(60));
    let _ = writeln!(
        out,
        "Nothing was written to disk. Run `zabal submit` to generate the document."
    );
    out
}

/// Shorten to `width`, leaving room for an ellipsis. Operates on chars so a
/// multi byte commit summary cannot be split mid character.
fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let keep = width.saturating_sub(3);
    let mut out: String = s.chars().take(keep).collect();
    out.push_str("...");
    out
}
