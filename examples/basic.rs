extern crate mrh;

fn main() {
    mrh::Crawler::new(".")
        .pending(true)
        .ignore_untracked(true)
        .ignore_uncommitted_repos(true)
        .for_each(|output| println!("{:?}", output));
}
