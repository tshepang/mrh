#[cfg(any(feature = "yaml", feature = "json"))]
use serde::Serialize;

use std::{io::Write, process};

use ansi_term::Color;
use clap::Parser;

use mrh::Crawler;

const CYAN: Color = Color::Fixed(6);
const BRIGHT_BLACK: Color = Color::Fixed(8);
const BRIGHT_RED: Color = Color::Fixed(9);

#[derive(Parser)]
#[clap(about, version)]
struct Opt {
    /// Only show repos with pending action
    #[clap(long)]
    pending: bool,
    /// Do not include untracked files in output
    #[clap(long)]
    ignore_untracked: bool,
    /// Do not include repos that have no commits
    #[clap(long)]
    ignore_uncommitted_repos: bool,
    /// Display absolute paths for repos
    #[clap(long)]
    absolute_paths: bool,
    /// Check if HEAD is untagged
    #[clap(long)]
    untagged_heads: bool,
    /// Compare against remote repo, most likely over the network
    #[clap(long, possible_value = "ssh-key", possible_value = "ssh-agent")]
    ssh_auth_method: Option<String>,
    /// Display output in YAML format
    #[clap(long, conflicts_with = "output-json")]
    output_yaml: bool,
    /// Display output in JSON format
    #[clap(long)]
    output_json: bool,
}

#[cfg(any(feature = "yaml", feature = "json"))]
#[derive(Serialize)]
struct Output {
    pub path: String,
    pub pending: Option<Vec<String>>,
    pub error: Option<String>,
}

fn main() {
    let cli = Opt::parse();
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(why) => {
            eprintln!(
                "{}: Could not read current directory: {}",
                BRIGHT_RED.paint("error"),
                why.to_string(),
            );
            process::exit(1)
        }
    };
    let crawler = Crawler::new(&current_dir)
        .pending(cli.pending)
        .ignore_untracked(cli.ignore_untracked)
        .ignore_uncommitted_repos(cli.ignore_uncommitted_repos)
        .access_remote(cli.ssh_auth_method)
        .absolute_paths(cli.absolute_paths)
        .untagged_heads(cli.untagged_heads);
    for output in crawler {
        if cli.output_json {
            display_json(output);
        } else if cli.output_yaml {
            display_yaml(output);
        } else {
            display_human(output);
        }
    }
}

fn display_human(result: mrh::Output) {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();
    let mut output = format!("{}", result.path.display());
    if let Some(pending) = result.pending {
        let pending: Vec<_> = pending.into_iter().collect();
        output.push_str(&format!(" ({})", CYAN.paint(pending.join(", "))));
    }
    if let Some(error) = result.error {
        output.push_str(&format!(
            " ({}: {})",
            BRIGHT_RED.paint("error"),
            BRIGHT_BLACK.paint(error.to_string()),
        ));
    }
    if let Err(why) = writeln!(std::io::stdout(), "{}", output) {
        if why.kind() == std::io::ErrorKind::BrokenPipe {
            process::exit(1);
        } else {
            eprintln!("{}", why);
        }
    }
}

#[cfg(any(feature = "yaml", feature = "json"))]
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
        eprintln!("{}", why);
        process::exit(1);
    }
    println!();
}
#[cfg(not(feature = "json"))]
fn display_json(_: mrh::Output) {
    eprintln!("Support for JSON output format not compiled in");
    process::exit(1);
}

#[cfg(feature = "yaml")]
fn display_yaml(output: mrh::Output) {
    let output = make_serde_digestible(output);
    if let Err(why) = serde_yaml::to_writer(std::io::stdout(), &output) {
        eprintln!("{}", why);
        process::exit(1);
    }
    println!();
}
#[cfg(not(feature = "yaml"))]
fn display_yaml(_: mrh::Output) {
    eprintln!("Support for YAML output format not compiled in");
    process::exit(1);
}
