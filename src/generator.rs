//! Submission rendering. Turns scanned commits into the document that gets
//! handed in, either Markdown or JSON.

use crate::parser::CommitInfo;
use chrono::{TimeZone, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Json,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "markdown" | "md" => Ok(Format::Markdown),
            "json" => Ok(Format::Json),
            other => Err(format!(
                "unknown format '{}'. Expected 'markdown' or 'json'.",
                other
            )),
        }
    }
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Json => "json",
        }
    }
}

/// Aggregate numbers across a set of commits.
#[derive(Debug, Clone, Serialize)]
pub struct Totals {
    pub commits: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub active_days: usize,
    pub contributors: usize,
}

pub fn totals(commits: &[CommitInfo]) -> Totals {
    let days: BTreeSet<&str> = commits.iter().map(|c| c.date.as_str()).collect();
    let people: BTreeSet<&str> = commits.iter().map(|c| c.author_name.as_str()).collect();

    Totals {
        commits: commits.len(),
        files_changed: commits.iter().map(|c| c.files_changed).sum(),
        insertions: commits.iter().map(|c| c.insertions).sum(),
        deletions: commits.iter().map(|c| c.deletions).sum(),
        active_days: days.len(),
        contributors: people.len(),
    }
}

#[derive(Debug, Serialize)]
struct Submission<'a> {
    handle: Option<&'a str>,
    project: Option<&'a str>,
    window_days: u64,
    period_start: String,
    period_end: String,
    generated_at: String,
    totals: Totals,
    commits: &'a [CommitInfo],
}

/// Build the submission document.
pub fn generate_submission(
    commits: &[CommitInfo],
    format: Format,
    handle: Option<&str>,
    project: Option<&str>,
    days: u64,
) -> Result<String, Box<dyn Error>> {
    let now = Utc::now();
    // Mirrors the cutoff arithmetic in `parser::scan_commits` exactly.
    let start = Utc
        .timestamp_opt(now.timestamp() - (days as i64) * 86_400, 0)
        .single()
        .ok_or("the start of the window is not a representable timestamp")?;
    let period_start = start.format("%Y-%m-%d").to_string();
    let period_end = now.format("%Y-%m-%d").to_string();

    match format {
        Format::Json => {
            let submission = Submission {
                handle,
                project,
                window_days: days,
                period_start,
                period_end,
                generated_at: now.to_rfc3339(),
                totals: totals(commits),
                commits,
            };
            Ok(serde_json::to_string_pretty(&submission)?)
        }
        Format::Markdown => Ok(markdown(
            commits,
            handle,
            project,
            days,
            &period_start,
            &period_end,
            &now.to_rfc3339(),
        )),
    }
}

fn markdown(
    commits: &[CommitInfo],
    handle: Option<&str>,
    project: Option<&str>,
    days: u64,
    period_start: &str,
    period_end: &str,
    generated_at: &str,
) -> String {
    let t = totals(commits);
    let mut out = String::new();

    let _ = writeln!(out, "# ZABAL Hackathon Submission\n");
    if let Some(project) = project {
        let _ = writeln!(out, "**Project:** {}  ", project);
    }
    if let Some(handle) = handle {
        let _ = writeln!(out, "**Handle:** {}  ", handle);
    }
    let _ = writeln!(
        out,
        "**Period:** last {} day{} ({} to {})  ",
        days,
        if days == 1 { "" } else { "s" },
        period_start,
        period_end
    );
    let _ = writeln!(out, "**Generated:** {}\n", generated_at);

    if commits.is_empty() {
        let _ = writeln!(
            out,
            "No commits landed in this window. Widen it with `--days` to cover more history."
        );
        return out;
    }

    let _ = writeln!(out, "## Summary\n");
    let _ = writeln!(out, "| Metric | Value |");
    let _ = writeln!(out, "| --- | --- |");
    let _ = writeln!(out, "| Commits | {} |", t.commits);
    let _ = writeln!(out, "| Files changed | {} |", t.files_changed);
    let _ = writeln!(out, "| Lines added | {} |", t.insertions);
    let _ = writeln!(out, "| Lines removed | {} |", t.deletions);
    let _ = writeln!(out, "| Active days | {} |", t.active_days);
    let _ = writeln!(out, "| Contributors | {} |", t.contributors);
    let _ = writeln!(out);

    let _ = writeln!(out, "## What shipped\n");
    for (date, day_commits) in group_by_day(commits) {
        let _ = writeln!(out, "### {}\n", date);
        for c in day_commits {
            let merge = if c.is_merge { " _(merge)_" } else { "" };
            let _ = writeln!(
                out,
                "- `{}` {}{} <br>_{} file{}, +{} / -{}_",
                c.short_id,
                c.summary,
                merge,
                c.files_changed,
                if c.files_changed == 1 { "" } else { "s" },
                c.insertions,
                c.deletions
            );
            if let Some(body) = &c.body {
                for line in body.lines() {
                    let _ = writeln!(out, "  > {}", line);
                }
            }
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Contributors\n");
    for (author, count) in commits_per_author(commits) {
        let _ = writeln!(
            out,
            "- {} ({} commit{})",
            author,
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    out
}

/// Commits bucketed by calendar day, most recent day first.
fn group_by_day(commits: &[CommitInfo]) -> Vec<(String, Vec<&CommitInfo>)> {
    let mut days: BTreeMap<&str, Vec<&CommitInfo>> = BTreeMap::new();
    for c in commits {
        days.entry(c.date.as_str()).or_default().push(c);
    }
    days.into_iter()
        .rev()
        .map(|(date, cs)| (date.to_string(), cs))
        .collect()
}

/// Authors with their commit counts, busiest first.
fn commits_per_author(commits: &[CommitInfo]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in commits {
        *counts.entry(c.author_name.as_str()).or_insert(0) += 1;
    }
    let mut ranked: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(name, n)| (name.to_string(), n))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
}
