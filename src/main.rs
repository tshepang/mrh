extern crate git2;
extern crate walkdir;
extern crate ansi_term;
extern crate ordermap;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::path::{Path, PathBuf};

use ordermap::set::OrderSet as Set;
use walkdir::{DirEntry, WalkDir};
use git2::Repository;
use structopt::StructOpt;
use ansi_term::Color;

const CYAN: Color = Color::Fixed(6);
const BRIGHT_BLACK: Color = Color::Fixed(8);
const BRIGHT_RED: Color = Color::Fixed(9);

#[derive(StructOpt)]
struct Opt {
    #[structopt(
        long = "pending",
        help = "Only show repos with pending action",
    )]
    pending: bool,
    #[structopt(
        long = "ignore-untracked",
        help = "Do not include untracked files in repos with pending action",
    )]
    ignore_untracked: bool,
    #[structopt(
        long = "absolute-paths",
        help = "Display absolute paths for repos",
    )]
    absolute_paths: bool,
    #[structopt(
        long = "untagged-head",
        help = "Check if HEAD is untagged",
    )]
    untagged_head: bool,
}

fn main() {
    let current_dir = match std::env::current_dir() {
        Ok(path) => path,
        Err(why) => {
            println!(
                "{}: {}",
                BRIGHT_RED.paint("error"),
                BRIGHT_BLACK.paint(why.to_string()),
            );
            std::process::exit(1)
        }
    };

    fn is_git_dir(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|string| !string.starts_with(".git"))
            .unwrap_or(false)
    }

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|entry| is_git_dir(entry))
        .filter_map(|entry| entry.ok()) // ignore stuff we can't read
        .filter(|entry| entry.file_type().is_dir()) // ignore non-dirs
    {
        let path = entry.path();
        if let Ok(repo) = Repository::open(path) {
            repo_ops(&repo, &current_dir);
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

fn repo_ops(repo: &Repository, current_dir: &Path) {
    let cli = Opt::from_args();
    if let Some(path) = repo.workdir() {
        let mut path = path.to_path_buf();
        if !cli.absolute_paths {
            path = make_relative(&path, current_dir);
        }
        let mut opts = git2::StatusOptions::new();
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
                            git2::Delta::Untracked => {
                                if !cli.ignore_untracked {
                                    pending.insert("untracked files");
                                }
                            }
                            git2::Delta::Modified => {
                                pending.insert("uncommitted changes");
                            }
                            git2::Delta::Deleted => {
                                pending.insert("deleted files");
                            }
                            git2::Delta::Renamed => {
                                pending.insert("renamed files");
                            }
                            _ => (),
                        }
                    }
                    if let Some(diff_delta) = status.head_to_index() {
                        match diff_delta.status() {
                            git2::Delta::Added => {
                                pending.insert("added files");
                            }
                            git2::Delta::Modified => {
                                pending.insert("uncommitted changes");
                            }
                            git2::Delta::Deleted => {
                                pending.insert("deleted files");
                            }
                            git2::Delta::Renamed => {
                                pending.insert("renamed files");
                            }
                            _ => (),
                        }
                    };
                }
                let local_ref = match repo.head() {
                    Ok(head) => head,
                    Err(why) => {
                        println!(
                            "{} ({}: {})",
                            path.display(),
                            BRIGHT_RED.paint("error"),
                            BRIGHT_BLACK.paint(why.to_string()),
                        );
                        return;
                    }
                };
                if cli.untagged_head {
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
                let branch = git2::Branch::wrap(local_ref);
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
                    // HashSet, for some reason, does not have join()
                    let pending: Vec<_> = pending.into_iter().collect();
                    println!("{} ({})", path.display(), CYAN.paint(pending.join(", ")));
                } else if !cli.pending {
                    println!("{}", path.display());
                }
            }

            Err(why) => {
                println!(
                    "{} ({}: {})",
                    path.display(),
                    CYAN.paint("error"),
                    BRIGHT_BLACK.paint(why.to_string()),
                );
            }
        }
    }
}
