extern crate mrh;

use std::path::Path;

fn main() {
    let crawler = mrh::Crawler::new(Path::new("."))
        .pending(true)
        .ignore_untracked(true)
        .ignore_uncommitted_repos(true);
    for output in crawler {
        println!("{:?}", output);
    }
}
