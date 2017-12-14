//! A Git repo can be in a number of states where some pending actions may
//! need to be taken:
//!
//! - uncommitted changes
//! - untracked files (can be disabled flag)
//! - unpushed commits
//! - unpulled commits
//! - added files
//! - deleted files
//! - renamed files
//! - untagged HEAD (optional)
//!
//! This library is meant to inspect those states, given a root path as
//! starting point.
//!
//! For a usage example, see `main.rs`, which is the command-line tool
//! exercising the library.
extern crate git2;
extern crate ordermap;
extern crate walkdir;

use std::path::{Path, PathBuf};

use ordermap::set::OrderSet as Set;
use walkdir::{DirEntry, WalkDir};
use git2::{Branch, Delta, Error, Repository, StatusOptions};

/// Represents Crawler output. There are 3 possible scenarios:
///
/// - There are no pending states, so only `path` (to the repo) has a
///   value
/// - There are no pending states, and there is some error preventing the
///   repo to be inspected properly... the `path` and `error` variant will
///   have values
/// - There are pending states... `path` and `pending` will have values
pub struct Output {
    pub path: Option<PathBuf>,
    pub pending: Option<Set<&'static str>>,
    pub error: Option<Error>,
}

/// Crawls the filesystem, given a starting point, looking for Git repos.
pub struct Crawler<'a> {
    pending: bool,
    ignore_untracked: bool,
    absolute_paths: bool,
    untagged_heads: bool,
    root_path: &'a Path,
}

impl<'a> Crawler<'a> {
    /// `path` is where crawling for Git repos begin, the starting point
    pub fn new(path: &'a Path) -> Self {
        Crawler {
            pending: false,
            ignore_untracked: false,
            absolute_paths: false,
            untagged_heads: false,
            root_path: path,
        }
    }

    /// Decide if you only want matches that are in pending state
    pub fn pending(mut self, answer: bool) -> Self {
        self.pending = answer;
        self
    }

    /// Decide if you want to exclude matches that have untracked files
    pub fn ignore_untracked(mut self, answer: bool) -> Self {
        self.ignore_untracked = answer;
        self
    }

    /// Display absolute paths (instead of relative ones)
    pub fn absolute_paths(mut self, answer: bool) -> Self {
        self.absolute_paths = answer;
        self
    }

    /// Decide if you want matches whose HEADS are not tagged
    ///
    /// A use-case is where related repositories (e.g. those comprising
    /// a single system), need to be tagged before, say, a release
    pub fn untagged_heads(mut self, answer: bool) -> Self {
        self.untagged_heads = answer;
        self
    }

    /// Return the results as an iterator
    pub fn iter(&self) -> RepoIter {
        let iter = WalkDir::new(&self.root_path)
            .into_iter()
            .filter_entry(|entry| is_git_dir(entry))
            .filter_map(|entry| entry.ok()) // ignore stuff we can't read
            .filter(|entry| entry.file_type().is_dir()); // ignore non-dirs
        RepoIter::new(self,iter)
    }

    fn repo_ops(&self, repo: &Repository) -> Option<Output> {
        if let Some(path) = repo.workdir() {
            let mut path = path.to_path_buf();
            if !self.absolute_paths {
                path = self.make_relative(&path);
            }
            let mut opts = StatusOptions::new();
            opts.include_ignored(false)
                .include_untracked(true)
                .renames_head_to_index(true)
                .renames_index_to_workdir(true);
            match repo.statuses(Some(&mut opts)) {
                Ok(statuses) => {
                    let mut pending = Set::new();
                    for status in statuses.iter() {
                        if let Some(diff_delta) = status.index_to_workdir() {
                            match diff_delta.status() {
                                Delta::Untracked => {
                                    if !self.ignore_untracked {
                                        pending.insert("untracked files");
                                    }
                                }
                                Delta::Modified => {
                                    pending.insert("uncommitted changes");
                                }
                                Delta::Deleted => {
                                    pending.insert("deleted files");
                                }
                                Delta::Renamed => {
                                    pending.insert("renamed files");
                                }
                                _ => (),
                            }
                        }
                        if let Some(diff_delta) = status.head_to_index() {
                            match diff_delta.status() {
                                Delta::Added => {
                                    pending.insert("added files");
                                }
                                Delta::Modified => {
                                    pending.insert("uncommitted changes");
                                }
                                Delta::Deleted => {
                                    pending.insert("deleted files");
                                }
                                Delta::Renamed => {
                                    pending.insert("renamed files");
                                }
                                _ => (),
                            }
                        };
                    }
                    let local_ref = match repo.head() {
                        Ok(head) => head,
                        Err(why) => {
                            return Some(Output {
                                path: Some(path),
                                pending: None,
                                error: Some(why),
                            });
                        }
                    };
                    if self.untagged_heads {
                        if let Ok(tags) = repo.tag_names(None) {
                            let mut untagged = true;
                            for tag in tags.iter() {
                                if let Some(tag) = tag {
                                    let tag = format!("refs/tags/{}", tag);
                                    if let Ok(reference) = repo.find_reference(&tag) {
                                        if reference == local_ref {
                                            untagged = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            if untagged {
                                pending.insert("untagged HEAD");
                            }
                        }
                    }
                    let branch = Branch::wrap(local_ref);
                    if let Ok(upstream_branch) = branch.upstream() {
                        let remote_ref = upstream_branch.into_reference();
                        let local_oid = branch.get().target().unwrap();
                        let remote_oid = remote_ref.target().unwrap();
                        if local_oid != remote_oid {
                            if let Ok((ahead, behind)) =
                                repo.graph_ahead_behind(local_oid, remote_oid)
                            {
                                if ahead > 0 {
                                    pending.insert("unpushed commits");
                                }
                                if behind > 0 {
                                    pending.insert("unpulled commits");
                                }
                            }
                        }
                    }
                    if !pending.is_empty() {
                        Some(Output {
                            path: Some(path),
                            pending: Some(pending),
                            error: None,
                        })
                    } else if !self.pending {
                        Some(Output {
                            path: Some(path),
                            pending: None,
                            error: None,
                        })
                    } else {
                        None
                    }
                }
                Err(why) => Some(Output {
                    path: Some(path),
                    pending: None,
                    error: Some(why),
                }),
            }
        } else {
            None
        }
    }

    fn make_relative(&self, target_dir: &Path) -> PathBuf {
        if let Ok(path) = target_dir.strip_prefix(self.root_path) {
            if path.to_string_lossy().is_empty() {
                ".".into()
            } else {
                path.into()
            }
        } else {
            target_dir.into()
        }
    }
}

pub struct RepoIter<'a,'b> where 'b: 'a {
    crawler: &'a Crawler<'b>,
    iter: Box<Iterator<Item=DirEntry>>,
}

impl <'a,'b>RepoIter<'a,'b> {
    fn new <I>(crawler: &'a Crawler<'b>, iter: I) -> RepoIter<'a,'b>
    where I: Iterator<Item=DirEntry> + 'static {
        RepoIter {crawler: crawler, iter: Box::new(iter)}
    }
}

impl <'a,'b>Iterator for RepoIter<'a,'b> {
    type Item = Output;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(entry) => {
                    let path = entry.path();
                    if let Ok(repo) = Repository::open(path) {
                        if let Some(output) = self.crawler.repo_ops(&repo) {
                            return Some(output);
                        }
                    }
                }
            }
        }
    }
}

fn is_git_dir(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|string| !string.starts_with(".git"))
        .unwrap_or(false)
}
