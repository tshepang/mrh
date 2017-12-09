extern crate git2;
extern crate walkdir;
extern crate ordermap;

use std::path::{Path, PathBuf};
use std::{env, io};

use ordermap::set::OrderSet as Set;
use walkdir::{DirEntry, WalkDir};
use git2::{Repository, StatusOptions, Delta, Branch, Error};

pub struct Output<'a> {
    pub path: Option<PathBuf>,
    pub pending: Option<Set<&'a str>>,
    pub error: Option<Error>,
}

#[derive(Default)]
pub struct Crawler {
    pending: bool,
    ignore_untracked: bool,
    absolute_paths: bool,
    untagged_heads: bool,
}

impl Crawler {
    pub fn new() -> Self {
        Crawler {
            pending: false,
            ignore_untracked: false,
            absolute_paths: false,
            untagged_heads: false,
        }
    }

    pub fn pending(mut self, answer: bool) -> Self {
        self.pending = answer;
        self
    }

    pub fn ignore_untracked(mut self, answer: bool) -> Self {
        self.ignore_untracked = answer;
        self
    }

    pub fn absolute_paths(mut self, answer: bool) -> Self {
        self.absolute_paths = answer;
        self
    }

    pub fn untagged_heads(mut self, answer: bool) -> Self {
        self.untagged_heads = answer;
        self
    }

    pub fn run(&self) -> io::Result<Vec<Output>> {
        let current_dir = env::current_dir()?;

        fn is_git_dir(entry: &DirEntry) -> bool {
            entry
                .file_name()
                .to_str()
                .map(|string| !string.starts_with(".git"))
                .unwrap_or(false)
        }

        let mut results = Vec::new();
        for entry in WalkDir::new(".")
            .into_iter()
            .filter_entry(|entry| is_git_dir(entry))
            .filter_map(|entry| entry.ok()) // ignore stuff we can't read
            .filter(|entry| entry.file_type().is_dir()) // ignore non-dirs
        {
            let path = entry.path();
            if let Ok(repo) = Repository::open(path) {
                if let Some(output) = self.repo_ops(&repo, &current_dir) {
                    results.push(output);
                }
            }
        }
        Ok(results)
    }

    fn repo_ops(&self, repo: &Repository, current_dir: &Path) -> Option<Output> {
        if let Some(path) = repo.workdir() {
            let mut path = path.to_path_buf();
            if !self.absolute_paths {
                path = make_relative(&path, current_dir);
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
                Err(why) => {
                    Some(Output {
                        path: Some(path),
                        pending: None,
                        error: Some(why),
                    })
                }
            }
        } else {
            None
        }
    }
}

fn make_relative(target_dir: &Path, current_dir: &Path) -> PathBuf {
    if let Ok(path) = target_dir.strip_prefix(current_dir) {
        if path.to_string_lossy().is_empty() {
            ".".into()
        } else {
            path.into()
        }
    } else {
        target_dir.into()
    }
}
