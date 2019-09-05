extern crate ansi_term;
extern crate mrh;
extern crate structopt;
#[cfg(feature = "json")]
extern crate serde_json;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
#[cfg(any(feature = "yaml", feature = "json"))]
#[macro_use]
extern crate serde_derive;

use std::process;

use ansi_term::Color;
use structopt::StructOpt;

use mrh::Crawler;

const CYAN: Color = Color::Fixed(6);
const BRIGHT_BLACK: Color = Color::Fixed(8);
const BRIGHT_RED: Color = Color::Fixed(9);

#[derive(StructOpt)]
#[structopt(about)]
struct Opt {
    /// Only show repos with pending action
    #[structopt(long)]
    pending: bool,
    /// Do not include untracked files in output
    #[structopt(long)]
    ignore_untracked: bool,
    /// Do not include repos that have no commits
    #[structopt(long)]
    ignore_uncommitted_repos: bool,
    /// Display absolute paths for repos
    #[structopt(long)]
    absolute_paths: bool,
    /// Check if HEAD is untagged
    #[structopt(long)]
    untagged_heads: bool,
    /// Compare against remote repo, most likely over the network
    #[structopt(long, possible_value = "ssh-key", possible_value = "ssh-agent")]
    ssh_auth_method: Option<String>,
    /// Display output in YAML format
    #[structopt(long, conflicts_with = "output_json")]
    output_yaml: bool,
    /// Display output in JSON format
    #[structopt(long)]
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
    let cli = Opt::from_args();
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
    print!("{}", result.path.display());
    if let Some(pending) = result.pending {
        let pending: Vec<_> = pending.into_iter().collect();
        print!(" ({})", CYAN.paint(pending.join(", ")));
    }
    if let Some(error) = result.error {
        print!(
            " ({}: {})",
            BRIGHT_RED.paint("error"),
            BRIGHT_BLACK.paint(error.to_string()),
        );
    }
    println!();
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
    let error = match result.error {
        Some(error) => Some(error.to_string()),
        None => None,
    };
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
