extern crate ansi_term;
extern crate mrh;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
#[cfg(feature = "json")]
extern crate serde_json;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
#[cfg(any(feature = "yaml", feature = "json"))]
#[macro_use]
extern crate serde_derive;

use std::process;

use structopt::StructOpt;
use ansi_term::Color;
use mrh::Crawler;

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
        long = "untagged-heads",
        help = "Check if HEAD is untagged",
    )]
    untagged_heads: bool,
    #[structopt(
        long = "access-remote",
        help = "Compare against remote repo, most likely over the network",
    )]
    access_remote: bool,
    #[structopt(
        long = "output-yaml",
        help = "Display output in YAML format",
        conflicts_with = "output_json",
    )]
    output_yaml: bool,
    #[structopt(
        long = "output-json",
        help = "Display output in JSON format",
    )]
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
        .access_remote(cli.access_remote)
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
        path: path,
        pending: pending,
        error: error,
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
