//! mrh - Multi-(git)Repo Helper
//!
//! A Git repo can be in a number of states where some pending actions may
//! need to be taken:
//!
//! - uncommitted changes
//! - unpushed commits
//! - outdated branch
//! - added files
//! - deleted files
//! - renamed files
//! - untracked files (can be disabled)
//! - untagged HEAD (optional)
//! - unpushed tags (optional)
//! - unpulled tags (optional)
//! - unpulled commits (optional)
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
use walkdir::WalkDir;
use git2::{Branch, Delta, Error, Repository, StatusOptions};

/// Represents Crawler output
///
/// There are 3 possible scenarios:
///
/// - There are no pending states, so only `path` (to the repo) has a
///   value
/// - There are no pending states, and there is some error preventing the
///   repo to be inspected properly... the `path` and `error` variant will
///   have values
/// - There are pending states... `path` and `pending` will have values
pub struct Output {
    /// Repository path
    pub path: PathBuf,
    /// A list of pending actions
    pub pending: Option<Set<&'static str>>,
    /// Git-related error
    pub error: Option<Error>,
}

/// Crawls the filesystem, looking for Git repos
pub struct Crawler<'a> {
    pending: bool,
    ignore_untracked: bool,
    absolute_paths: bool,
    untagged_heads: bool,
    access_remote: bool,
    root_path: &'a Path,
    iter: Box<Iterator<Item = Repository>>,
}

impl<'a> Crawler<'a> {
    /// `root` is where crawling for Git repos begin
    pub fn new(root: &'a Path) -> Self {
        Crawler {
            pending: false,
            ignore_untracked: false,
            absolute_paths: false,
            untagged_heads: false,
            access_remote: false,
            root_path: root,
            iter: Box::new(
                WalkDir::new(root)
                    .into_iter()
                    .filter_map(|entry| entry.ok()) // ignore stuff we can't read
                    .filter(|entry| entry.file_type().is_dir()) // ignore non-dirs
                    .filter(|entry| entry.file_name() != ".git") // avoid double-hits
                    .filter_map(|entry| Repository::open(entry.path()).ok()),
            ),
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

    /// Allow access to the remote of the repo
    ///
    /// This allows checking if the repo is in sync with upstream,
    /// so will be relatively slow if remote is behind a network
    /// (which is the most likely scenario).
    pub fn access_remote(mut self, answer: bool) -> Self {
        self.access_remote = answer;
        self
    }

    fn repo_ops(&self, repo: &Repository) -> Option<Output> {
        if let Some(path) = repo.workdir() {
            // ignore libgit2-sys test repos
            if git2::Repository::discover(path).is_err() {
                return None;
            }
            let mut pending = Set::new();
            let mut path = path.to_path_buf();
            if !self.absolute_paths {
                path = self.make_relative(&path);
            }
            let mut opts = StatusOptions::new();
            opts.include_ignored(false)
                .include_untracked(true)
                .renames_head_to_index(true)
                .renames_index_to_workdir(true);
            let local_ref = match repo.head() {
                Ok(head) => head,
                Err(why) => {
                    return Some(Output {
                        path: path,
                        pending: None,
                        error: Some(why),
                    });
                }
            };
            let branch = Branch::wrap(local_ref);
            match repo.statuses(Some(&mut opts)) {
                Ok(statuses) => {
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
                    if self.untagged_heads {
                        let local_ref = branch.get();
                        if let Ok(tags) = repo.tag_names(None) {
                            let mut untagged = true;
                            for tag in tags.iter() {
                                if let Some(tag) = tag {
                                    let tag = format!("refs/tags/{}", tag);
                                    if let Ok(reference) = repo.find_reference(&tag) {
                                        if &reference == local_ref {
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
                    if let Ok(upstream_branch) = branch.upstream() {
                        let remote_ref = upstream_branch.into_reference();
                        let remote_oid = remote_ref.target().unwrap();
                        let local_oid = branch.get().target().unwrap();
                        if local_oid != remote_oid {
                            if let Ok((ahead, behind)) =
                                repo.graph_ahead_behind(local_oid, remote_oid)
                            {
                                if ahead > 0 {
                                    pending.insert("unpushed commits");
                                }
                                if behind > 0 {
                                    pending.insert("outdated branch");
                                }
                            }
                        }
                    }
                    if self.access_remote {
                        if let Ok(remote) = repo.find_remote("origin") {
                            let config = git2::Config::open_default().unwrap();
                            let url = remote.url().unwrap();
                            let mut callbacks = git2::RemoteCallbacks::new();
                            if url.starts_with("http") {
                                callbacks.credentials(|_, _, _| {
                                    git2::Cred::credential_helper(&config, url, None)
                                });
                            } else if url.starts_with("git") {
                                for file_name in &["id_rsa", "id_dsa"] {
                                    if let Some(home_dir) = std::env::home_dir() {
                                        let private_key = home_dir.join(".ssh").join(file_name);
                                        if private_key.exists() {
                                            callbacks.credentials(move |_, _, _| {
                                                git2::Cred::ssh_key("git", None, &private_key, None)
                                            });
                                            break;
                                        }
                                    }
                                }
                            }
                            // avoid "cannot borrow immutable local variable `remote` as mutable"
                            let mut remote = remote.clone();
                            if let Err(why) =
                                remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)
                            {
                                return Some(Output {
                                    path: path,
                                    pending: None,
                                    error: Some(why),
                                });
                            }
                            let local_head_oid = branch.get().target().unwrap();
                            let mut remote_tags = Set::new();
                            if let Ok(remote_list) = remote.list() {
                                for item in remote_list {
                                    let name = item.name();
                                    if name.starts_with("refs/tags/") {
                                        // This weirdness of a postfix appears on some remote tags
                                        if !name.ends_with("^{}") {
                                            remote_tags
                                                .insert((item.name().to_string(), item.oid()));
                                        }
                                    } else if name == "HEAD" &&
                                    // XXX This can be better!
                                        item.oid() != local_head_oid
                                        && !pending.contains("unpushed commits")
                                        && !pending.contains("outdated branch")
                                    {
                                        pending.insert("unpulled commits");
                                    }
                                }
                                let mut local_tags = Set::new();
                                if let Ok(tags) = repo.tag_names(None) {
                                    for tag in tags.iter() {
                                        if let Some(tag) = tag {
                                            let tag = format!("refs/tags/{}", tag);
                                            if let Ok(reference) = repo.find_reference(&tag) {
                                                if let Some(oid) = reference.target() {
                                                    local_tags.insert((tag, oid));
                                                }
                                            }
                                        }
                                    }
                                }
                                if !local_tags.is_subset(&remote_tags) {
                                    pending.insert("unpushed tags");
                                }
                                if !remote_tags.is_subset(&local_tags) {
                                    pending.insert("unpulled tags");
                                }
                            }
                        }
                    }
                    if !pending.is_empty() {
                        Some(Output {
                            path: path,
                            pending: Some(pending),
                            error: None,
                        })
                    } else if !self.pending {
                        Some(Output {
                            path: path,
                            pending: None,
                            error: None,
                        })
                    } else {
                        None
                    }
                }
                Err(why) => Some(Output {
                    path: path,
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

impl<'a> Iterator for Crawler<'a> {
    type Item = Output;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(repo) => {
                    if let Some(output) = self.repo_ops(&repo) {
                        return Some(output);
                    }
                }
            }
        }
    }
}
