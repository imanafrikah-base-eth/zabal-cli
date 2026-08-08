//! Commit scanning. Walks the repository history and collects the commits
//! that fall inside a time window, along with their diff statistics.

use chrono::{DateTime, TimeZone, Utc};
use git2::{Commit, Repository, Sort};
use serde::Serialize;
use std::error::Error;
use std::path::Path;

/// One commit, flattened into the fields a submission document needs.
#[derive(Debug, Clone, Serialize)]
pub struct CommitInfo {
    pub short_id: String,
    pub id: String,
    pub summary: String,
    pub body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    /// Unix timestamp of the commit, in seconds.
    pub timestamp: i64,
    /// Calendar day, `YYYY-MM-DD`, used for grouping.
    pub date: String,
    pub datetime: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub is_merge: bool,
}

/// Collect every commit reachable from HEAD that landed within the last
/// `days` days, newest first.
///
/// `repo_path` may be the repository root or any directory inside it; the
/// repository is discovered by walking upwards. A repository with no commits
/// yet yields an empty list rather than an error.
pub fn scan_commits(repo_path: &Path, days: u64) -> Result<Vec<CommitInfo>, Box<dyn Error>> {
    let repo = Repository::discover(repo_path)?;

    if repo.is_empty()? {
        return Ok(Vec::new());
    }

    let cutoff = Utc::now().timestamp() - (days as i64) * 86_400;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        // Walk the whole history rather than stopping at the first commit
        // older than the cutoff: merges and rebases can put an older
        // timestamp ahead of a newer one.
        if commit.time().seconds() < cutoff {
            continue;
        }
        commits.push(describe(&repo, &commit)?);
    }

    commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(commits)
}

fn describe(repo: &Repository, commit: &Commit) -> Result<CommitInfo, Box<dyn Error>> {
    let (files_changed, insertions, deletions) = diff_stats(repo, commit)?;

    let seconds = commit.time().seconds();
    let when: DateTime<Utc> = Utc
        .timestamp_opt(seconds, 0)
        .single()
        .ok_or_else(|| format!("commit {} has an unrepresentable timestamp", commit.id()))?;

    let author = commit.author();
    let id = commit.id().to_string();

    Ok(CommitInfo {
        short_id: id.chars().take(7).collect(),
        id,
        summary: commit.summary().unwrap_or("(no commit message)").to_string(),
        body: commit
            .body()
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty()),
        author_name: author.name().unwrap_or("(unknown)").to_string(),
        author_email: author.email().unwrap_or("").to_string(),
        timestamp: seconds,
        date: when.format("%Y-%m-%d").to_string(),
        datetime: when.to_rfc3339(),
        files_changed,
        insertions,
        deletions,
        is_merge: commit.parent_count() > 1,
    })
}

/// Diff a commit against its first parent. Root commits are diffed against
/// the empty tree, so their whole content counts as additions.
fn diff_stats(repo: &Repository, commit: &Commit) -> Result<(usize, usize, usize), Box<dyn Error>> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let stats = diff.stats()?;

    Ok((stats.files_changed(), stats.insertions(), stats.deletions()))
}
