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
//! - uncommitted repos (can be disabled)
//! - untagged HEAD (optional)
//! - unpushed tags (optional)
//! - unpulled tags (optional)
//! - unfetched commits (optional)
//!
//! This library is meant to inspect those states, given a root path as
//! starting point.
//!
//! Example:
//!
//! ```
//! # use std::path::Path;
//! # fn main() {
//! mrh::Crawler::new(".")
//!     .pending(true)
//!     .ignore_untracked(true)
//!     .ignore_uncommitted_repos(true)
//!     .for_each(|output| println!("{:?}", output));
//! # }
//! ```

use std::path::{Path, PathBuf};

use git2::{Branch, Delta, Error, Repository, StatusOptions};
use indexmap::set::IndexSet as Set;
use walkdir::WalkDir;

/// Represents Crawler output
///
/// There are 3 possible scenarios:
///
/// - There are no pending states, so only `path` (to the repo) has a
///   value
/// - There are no pending states, and there is some error preventing the
///   repo from being inspected properly... `error` will have `Some` value
/// - There are pending states... `pending` will have `Some` value
#[derive(Debug)]
pub struct Output {
    /// Repository path
    pub path: PathBuf,
    /// A list of pending actions
    pub pending: Option<Set<&'static str>>,
    /// Git-related error
    pub error: Option<Error>,
}

/// Crawls the filesystem, looking for Git repos
pub struct Crawler {
    pending: bool,
    ignore_untracked: bool,
    ignore_uncommitted_repos: bool,
    absolute_paths: bool,
    untagged_heads: bool,
    access_remote: Option<String>,
    root_path: PathBuf,
    iter: Box<dyn Iterator<Item = Repository>>,
}

impl Crawler {
    /// `root` is where crawling for Git repos begin
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Crawler {
            pending: false,
            ignore_untracked: false,
            ignore_uncommitted_repos: false,
            absolute_paths: false,
            untagged_heads: false,
            access_remote: None,
            root_path: root.as_ref().into(),
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

    /// Decide if you want to exclude repos that have no commits
    ///
    /// This will happen when a `git init` is executed,
    /// and one forgets to commit.
    pub fn ignore_uncommitted_repos(mut self, answer: bool) -> Self {
        self.ignore_uncommitted_repos = answer;
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
    /// This allows checking if the repo is in sync with its remote counterpart,
    /// so will be relatively slow if remote is behind a network
    /// (which is the most likely scenario).
    ///
    /// # HTTP protocol remotes
    ///
    /// Uses Git's credentials.helper to determine what authentication
    /// method to use.
    /// If not successful:
    /// > error: an unknown git error occurred
    ///
    /// # Git protocol remotes
    ///
    /// If "ssh-key" is specified, the ssh key will be used for authentication.
    /// If "ssh-agent" is specified, a correctly-set ssh-agent will be assumed.
    /// This is useful for cases where passphrase is set on the ssh key,
    /// else you will get a:
    /// > error authenticating: no auth sock variable
    pub fn access_remote(mut self, ssh_auth_method: Option<String>) -> Self {
        self.access_remote = ssh_auth_method;
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
                    if self.ignore_uncommitted_repos
                        && why.class() == git2::ErrorClass::Reference
                        && why.code() == git2::ErrorCode::UnbornBranch
                    {
                        return None;
                    }
                    return Some(Output {
                        path,
                        pending: None,
                        error: Some(why),
                    });
                }
            };
            let local_branch = Branch::wrap(local_ref);
            let local_head_oid = match local_branch.get().target() {
                Some(oid) => oid,
                None => return None,
            };
            match repo.statuses(Some(&mut opts)) {
                Ok(statuses) => {
                    for status in statuses.iter() {
                        pending = self.diff_ops(&status, pending);
                    }
                    if self.untagged_heads {
                        let local_ref = local_branch.get();
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
                    if let Ok(upstream_branch) = local_branch.upstream() {
                        let upstream_ref = upstream_branch.into_reference();
                        let upstream_head_oid = match upstream_ref.target() {
                            Some(oid) => oid,
                            None => return None,
                        };
                        if local_head_oid != upstream_head_oid {
                            if let Ok((ahead, behind)) =
                                repo.graph_ahead_behind(local_head_oid, upstream_head_oid)
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
                    if self.access_remote.is_some() {
                        pending = match self.remote_ops(repo, pending, local_head_oid) {
                            Ok(pending) => pending,
                            Err(why) => {
                                return Some(Output {
                                    path,
                                    pending: None,
                                    error: Some(why),
                                });
                            }
                        }
                    }
                    if !pending.is_empty() {
                        Some(Output {
                            path,
                            pending: Some(pending),
                            error: None,
                        })
                    } else if !self.pending {
                        Some(Output {
                            path,
                            pending: None,
                            error: None,
                        })
                    } else {
                        None
                    }
                }
                Err(why) => Some(Output {
                    path,
                    pending: None,
                    error: Some(why),
                }),
            }
        } else {
            None
        }
    }

    fn diff_ops<'b>(&self, status: &git2::StatusEntry<'_>, mut pending: Set<&'b str>) -> Set<&'b str> {
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
        pending
    }

    fn remote_ops<'b>(
        &self,
        repo: &Repository,
        mut pending: Set<&'b str>,
        local_head_oid: git2::Oid,
    ) -> Result<Set<&'b str>, Error> {
        if let Ok(remote) = repo.find_remote("origin") {
            // XXX howto avoid the following panic
            let config = git2::Config::open_default().expect("could not get git config");
            let url = match remote.url() {
                Some(url) => url,
                // XXX should not ignore this one, though it seems not a likely one to occur
                None => return Ok(pending),
            };
            let mut callbacks = git2::RemoteCallbacks::new();
            if url.starts_with("http") {
                callbacks.credentials(|_, _, _| git2::Cred::credential_helper(&config, url, None));
            } else if url.starts_with("git") {
                // github, bitbucket, and gitlab use "git" as ssh username
                if let Some(ref method) = self.access_remote {
                    if method == "ssh-key" {
                        for file_name in &["id_rsa", "id_dsa"] {
                            if let Some(home_dir) = dirs::home_dir() {
                                let private_key = home_dir.join(".ssh").join(file_name);
                                if private_key.exists() {
                                    callbacks.credentials(move |_, _, _| {
                                        git2::Cred::ssh_key("git", None, &private_key, None)
                                    });
                                    break;
                                }
                            }
                        }
                    } else if method == "ssh-agent" {
                        callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_agent("git"));
                    }
                }
            }
            // avoid "cannot borrow immutable local variable `remote` as mutable"
            let mut remote = remote.clone();
            remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;
            let mut remote_tags = Set::new();
            if let Ok(remote_list) = remote.list() {
                for item in remote_list {
                    let name = item.name();
                    if name.starts_with("refs/tags/") {
                        // This weirdness of a postfix appears on some remote tags
                        if !name.ends_with("^{}") {
                            remote_tags.insert((item.name().to_string(), item.oid()));
                        }
                    } else if name.starts_with("refs/heads") && item.oid() != local_head_oid {
                        let mut found = false;
                        if let Ok(branches) = repo.branches(None) {
                            for branch in branches {
                                if let Ok(branch) = branch {
                                    if let Some(oid) = branch.0.get().target() {
                                        if oid == item.oid() {
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if !found {
                            pending.insert("unfetched commits");
                        }
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
        Ok(pending)
    }

    fn make_relative(&self, target_dir: &Path) -> PathBuf {
        if let Ok(path) = target_dir.strip_prefix(&self.root_path) {
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

impl Iterator for Crawler {
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
