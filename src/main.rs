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
    pub path: Option<String>,
    pub pending: Option<Vec<String>>,
    pub error: Option<String>,
}

fn main() {
    let cli = Opt::from_args();
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(why) => {
            println!(
                "{}: Could not read current directory: {}",
                BRIGHT_RED.paint("error"),
                why.to_string(),
            );
            std::process::exit(1)
        }
    };
    let crawler = Crawler::new(&current_dir)
        .pending(cli.pending)
        .ignore_untracked(cli.ignore_untracked)
        .absolute_paths(cli.absolute_paths)
        .untagged_heads(cli.untagged_heads);
    for output in crawler {
        if cli.output_json || cli.output_yaml {
            display(output, &cli);
        } else {
            display_human(output);
        }
    }
}

fn display_human(result: mrh::Output) {
    if let Some(path) = result.path {
        print!("{}", path.display());
    }
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
fn display(result: mrh::Output, cli: &Opt) {
    let path = match result.path {
        Some(path) => Some(path.to_string_lossy().to_string()),
        None => None,
    };
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
    let output = Output {
        path: path,
        pending: pending,
        error: error,
    };
    if cli.output_json {
        display_json(&output);
    } else if cli.output_yaml {
        display_yaml(&output);
    } else {
        unreachable!();
    }
}
#[cfg(not(any(feature = "yaml", feature = "json")))]
fn display(_: mrh::Output, cli: &Opt) {
    let format = if cli.output_json { "JSON" } else { "YAML" };
    eprintln!("Support for {} output format not compiled in", format);
}

#[cfg(feature = "json")]
fn display_json(output: &Output) {
    if let Err(why) = serde_json::to_writer(std::io::stdout(), &output) {
        eprintln!("{}", why);
    } else {
        println!();
    }
}
#[cfg(not(feature = "json"))]
#[cfg(feature = "yaml")]
fn display_json(_: &Output) {
    eprintln!("Support for YAML output format not compiled in");
}

#[cfg(feature = "yaml")]
fn display_yaml(output: &Output) {
    if let Err(why) = serde_yaml::to_writer(std::io::stdout(), &output) {
        eprintln!("{}", why);
    } else {
        println!();
    }
}
#[cfg(not(feature = "yaml"))]
#[cfg(feature = "json")]
fn display_yaml(_: &Output) {
    eprintln!("Support for JSON output format not compiled in");
}
