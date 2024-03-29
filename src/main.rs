#[cfg(feature = "json")]
use serde::Serialize;

use std::{
    fmt::Write as _,
    io::Write,
    path::{Path, PathBuf},
    process,
};

use ansi_term::Color;
use anyhow::{bail, ensure, Result};
use clap::Parser;

use mrh::Crawler;

const CYAN: Color = Color::Fixed(6);
const BRIGHT_BLACK: Color = Color::Fixed(8);
const BRIGHT_RED: Color = Color::Fixed(9);

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    /// Only show repos with pending action
    #[arg(long)]
    pending: bool,
    /// Do not include untracked files in output
    #[arg(long)]
    ignore_untracked: bool,
    /// Do not include repos that have no commits
    #[arg(long)]
    ignore_uncommitted_repos: bool,
    /// Display absolute paths for repos
    #[arg(long)]
    absolute_paths: bool,
    /// Check if HEAD is untagged
    #[arg(long)]
    untagged_heads: bool,
    /// Compare against remote repo, most likely over the network
    #[arg(long, value_parser = ["ssh-key", "ssh-agent"])]
    ssh_auth_method: Option<String>,
    /// Display output in JSON format
    #[arg(long)]
    output_json: bool,
    /// Choose a path where to start the crawl
    #[arg(default_value = ".")]
    root_path: PathBuf,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct Output {
    pub path: String,
    pub pending: Option<Vec<String>>,
    pub error: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    ensure!(
        cli.root_path.metadata()?.is_dir(),
        "root path should be a directory",
    );
    let crawler = Crawler::new(&cli.root_path)
        .pending(cli.pending)
        .ignore_untracked(cli.ignore_untracked)
        .ignore_uncommitted_repos(cli.ignore_uncommitted_repos)
        .access_remote(cli.ssh_auth_method)
        .absolute_paths(cli.absolute_paths)
        .untagged_heads(cli.untagged_heads);
    for output in crawler {
        if cli.output_json {
            display_json(output);
        } else {
            display_human(output)?;
        }
    }
    Ok(())
}

fn display_human(result: mrh::Output) -> Result<()> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(why) => {
            bail!(
                "{}: Could not read current directory: {}",
                BRIGHT_RED.paint("error"),
                why,
            );
        }
    };
    let mut output = if let Ok(path) = result.path.strip_prefix(current_dir) {
        if path == Path::new("") {
            ".".into()
        } else {
            String::from(path.to_string_lossy())
        }
    } else {
        String::from(result.path.to_string_lossy())
    };
    if let Some(pending) = result.pending {
        let pending: Vec<_> = pending.into_iter().collect();
        write!(output, " ({})", CYAN.paint(pending.join(", ")))?;
    }
    if let Some(error) = result.error {
        write!(
            output,
            " ({}: {})",
            BRIGHT_RED.paint("error"),
            BRIGHT_BLACK.paint(error.to_string()),
        )?;
    }
    if let Err(why) = writeln!(std::io::stdout(), "{output}") {
        if why.kind() == std::io::ErrorKind::BrokenPipe {
            process::exit(1);
        } else {
            eprintln!("{why}");
        }
    }
    Ok(())
}

#[cfg(feature = "json")]
fn make_serde_digestible(result: mrh::Output) -> Output {
    let path = result.path.to_string_lossy().to_string();
    let pending = match result.pending {
        Some(pending) => {
            let vec: Vec<_> = pending.iter().map(|value| value.to_string()).collect();
            Some(vec)
        }
        None => None,
    };
    let error = result.error.map(|error| error.to_string());
    Output {
        path,
        pending,
        error,
    }
}

#[cfg(feature = "json")]
fn display_json(output: mrh::Output) {
    let output = make_serde_digestible(output);
    if let Err(why) = serde_json::to_writer(std::io::stdout(), &output) {
        eprintln!("{why}");
        process::exit(1);
    }
    println!();
}
#[cfg(not(feature = "json"))]
fn display_json(_: mrh::Output) {
    eprintln!("Support for JSON output format not compiled in");
    process::exit(1);
}
