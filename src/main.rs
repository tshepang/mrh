extern crate git2;
extern crate walkdir;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::path::{self, Path, PathBuf};
use std::env;
use std::process;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(
        long = "changed",
        help = "Only show repos with uncommited changes",
    )]
    changed: bool,
}

fn main() {
    fn valid(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|string| !string.starts_with(".git"))
            .unwrap_or(false)
    }

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(why) => {
            println!("{}", why);
            process::exit(1);
        }
    };

    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_entry(|entry| valid(entry))
        .filter_map(|entry| entry.ok()) // ignore stuff we can't read
        .filter(|entry| entry.file_type().is_dir()) // ignore non-dirs
    {
        // XXX Does not handle symlinks proper
        let path = entry.path();
        if let Ok(repo) = Repository::open(path) {
            repo_ops(&repo, &current_dir);
        }
    }
}

// XXX This will break if current_dir is RootDir
fn make_relative(path: &Path, current_dir: &Path) -> PathBuf {
    if path.is_relative() {
        return path.into();
    }
    let mut result = PathBuf::new();
    let mut path_before_current_dir = PathBuf::new();
    let mut after_current_dir = false;
    for component in path.components() {
        if component == path::Component::RootDir {
            path_before_current_dir.push(component.as_os_str());
            continue;
        }
        if after_current_dir {
            result.push(component.as_os_str());
        } else {
            path_before_current_dir.push(component.as_os_str());
        }
        if path_before_current_dir == current_dir {
            after_current_dir = true;
        }
    }
    if result.to_string_lossy().is_empty() {
        ".".into()
    } else {
        result
    }
}

fn repo_ops(repo: &Repository, current_dir: &Path) {
    let cli = Opt::from_args();
    if let Some(path) = repo.workdir() {
        let path = make_relative(path, current_dir);
        let mut opts = git2::StatusOptions::new();
        opts.include_ignored(false).include_untracked(true);
        match repo.statuses(Some(&mut opts)) {
            Ok(statuses) => {
                let local_ref = match repo.head() {
                    Ok(head) => head,
                    Err(why) => {
                        println!("{}", why);
                        process::exit(1);
                    }
                };
                let mut changes = Vec::new();
                if !statuses.is_empty() {
                    changes.push(format!("{} changes", statuses.len()));
                }
                let branch = git2::Branch::wrap(local_ref);
                if let Ok(upstream_branch) = branch.upstream() {
                    let remote_ref = upstream_branch.into_reference();
                    let local_oid = branch.get().target().unwrap();
                    let remote_oid = remote_ref.target().unwrap();
                    if local_oid != remote_oid {
                        changes.push("unpushed commits".into());
                    };
                }
                if !changes.is_empty() {
                    println!("{} ({})", path.display(), changes.join(", "));
                } else if !cli.changed {
                    println!("{}", path.display());
                }
            }
            Err(why) => {
                println!("{}", why);
                process::exit(1);
            }
        }
    }
}
