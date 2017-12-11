extern crate ansi_term;
extern crate mrh;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

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
    for result in crawler.run() {
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
}
