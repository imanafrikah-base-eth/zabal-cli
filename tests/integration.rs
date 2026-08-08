//! Integration tests for `scan_commits`.
//!
//! Each test builds a throwaway repository with commits at controlled
//! timestamps, so the time window logic can be exercised without depending on
//! whatever history happens to exist locally.

use git2::{Commit, Repository, Signature, Time};
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use zabal_cli::generator::{generate_submission, totals, Format};
use zabal_cli::parser::scan_commits;

const DAY: i64 = 86_400;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn new_repo() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("git init");
    (dir, repo)
}

/// Commit `contents` to `name` with an explicit commit time.
fn commit_file(repo: &Repository, name: &str, contents: &str, message: &str, when: i64) {
    let workdir = repo.workdir().expect("non bare repo").to_path_buf();
    fs::write(workdir.join(name), contents).expect("write file");

    let mut index = repo.index().expect("open index");
    index.add_path(Path::new(name)).expect("stage file");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");

    let sig = Signature::new("Test Dev", "test@example.com", &Time::new(when, 0))
        .expect("build signature");

    let parents: Vec<Commit> = match repo.head().ok().and_then(|h| h.target()) {
        Some(oid) => vec![repo.find_commit(oid).expect("find parent")],
        None => vec![],
    };
    let parent_refs: Vec<&Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .expect("create commit");
}

#[test]
fn empty_repository_yields_no_commits() {
    let (dir, _repo) = new_repo();
    let found = scan_commits(dir.path(), 7).expect("scan should succeed on an empty repo");
    assert!(found.is_empty());
}

// Note: a temp dir that merely lacks a `.git` is not a safe negative case,
// because `Repository::discover` walks upwards to the filesystem root and
// could find an unrelated repository above it. A path that does not exist
// fails deterministically instead.
#[test]
fn missing_path_is_an_error() {
    let dir = TempDir::new().expect("create temp dir");
    let missing = dir.path().join("no").join("such").join("directory");
    assert!(scan_commits(&missing, 7).is_err());
}

#[test]
fn commits_outside_the_window_are_excluded() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "old.txt", "old\n", "ancient work", now() - 30 * DAY);
    commit_file(&repo, "new.txt", "new\n", "recent work", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].summary, "recent work");

    let wider = scan_commits(dir.path(), 60).expect("scan");
    assert_eq!(wider.len(), 2);
}

#[test]
fn commits_are_returned_newest_first() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "a\n", "first", now() - 3 * DAY);
    commit_file(&repo, "b.txt", "b\n", "second", now() - 2 * DAY);
    commit_file(&repo, "c.txt", "c\n", "third", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    let summaries: Vec<&str> = found.iter().map(|c| c.summary.as_str()).collect();
    assert_eq!(summaries, vec!["third", "second", "first"]);
    assert!(found[0].timestamp > found[1].timestamp);
}

#[test]
fn diff_stats_are_populated() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "one\ntwo\nthree\n", "add three lines", now() - 2 * DAY);
    commit_file(&repo, "a.txt", "one\n", "cut back to one line", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    assert_eq!(found.len(), 2);

    // Newest first: the trimming commit removed two lines.
    let trim = &found[0];
    assert_eq!(trim.files_changed, 1);
    assert_eq!(trim.deletions, 2);

    // The root commit counts its whole content as additions.
    let root = &found[1];
    assert_eq!(root.insertions, 3);
    assert_eq!(root.deletions, 0);
    assert!(!root.is_merge);
}

#[test]
fn scanning_works_from_a_subdirectory() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "a\n", "only commit", now() - 1 * DAY);

    let nested = dir.path().join("src").join("deep");
    fs::create_dir_all(&nested).expect("create nested dirs");

    let found = scan_commits(&nested, 7).expect("scan should discover the repo upwards");
    assert_eq!(found.len(), 1);
}

#[test]
fn totals_aggregate_across_commits() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "one\ntwo\n", "add a", now() - 2 * DAY);
    commit_file(&repo, "b.txt", "three\n", "add b", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    let t = totals(&found);

    assert_eq!(t.commits, 2);
    assert_eq!(t.insertions, 3);
    assert_eq!(t.deletions, 0);
    assert_eq!(t.contributors, 1);
}

#[test]
fn markdown_submission_mentions_every_commit() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "a\n", "wire up the parser", now() - 2 * DAY);
    commit_file(&repo, "b.txt", "b\n", "wire up the generator", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    let doc = generate_submission(&found, Format::Markdown, Some("@tester"), Some("demo"), 7)
        .expect("render markdown");

    assert!(doc.contains("# ZABAL Hackathon Submission"));
    assert!(doc.contains("@tester"));
    assert!(doc.contains("wire up the parser"));
    assert!(doc.contains("wire up the generator"));
    assert!(doc.contains("| Commits | 2 |"));
}

#[test]
fn json_submission_is_valid_json() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "a\n", "only commit", now() - 1 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    let doc = generate_submission(&found, Format::Json, None, Some("demo"), 7).expect("render json");

    let parsed: serde_json::Value = serde_json::from_str(&doc).expect("output should parse as JSON");
    assert_eq!(parsed["totals"]["commits"], 1);
    assert_eq!(parsed["window_days"], 7);
    assert_eq!(parsed["commits"][0]["summary"], "only commit");
}

#[test]
fn empty_window_still_renders_a_document() {
    let (dir, repo) = new_repo();
    commit_file(&repo, "a.txt", "a\n", "ancient", now() - 90 * DAY);

    let found = scan_commits(dir.path(), 7).expect("scan");
    assert!(found.is_empty());

    let doc = generate_submission(&found, Format::Markdown, None, Some("demo"), 7).expect("render");
    assert!(doc.contains("No commits landed in this window"));
}

#[test]
fn format_parsing_accepts_aliases_and_rejects_junk() {
    assert_eq!("markdown".parse::<Format>().unwrap(), Format::Markdown);
    assert_eq!("MD".parse::<Format>().unwrap(), Format::Markdown);
    assert_eq!("json".parse::<Format>().unwrap(), Format::Json);
    assert!("yaml".parse::<Format>().is_err());
}
