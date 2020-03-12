# mrh - Multi-(git)Repo Helper

[![crates.io](https://img.shields.io/crates/v/mrh.svg)](https://crates.io/crates/mrh)
[![build status](https://github.com/tshepang/mrh/workflows/CI/badge.svg)](https://github.com/tshepang/mrh/actions)

This repo provides a library that allows crawling a directory and its
children for Git repos.
It reports if those repos have:

- uncommitted changes
- unpushed commits
- outdated branch
- added files
- deleted files
- renamed files
- untracked files (can be disabled)
- uncommitted repos (can be disabled)
- untagged HEAD (optional)
- unpushed tags (optional)
- unpulled tags (optional)
- unfetched commits (optional)

It also offers a command line tool with all those features,
one of which is to show all repos:

    $ mrh
    foo (uncommitted changes, untracked files, unpushed commits)
    bar
    baz (untracked files)
    qux

Only show those repos that are pending action:

    $ mrh --pending
    foo (uncommitted changes, untracked files, unpushed commits)
    baz (untracked files)

Ignore untracked files in results:

    $ mrh --pending --ignore-untracked
    foo (uncommitted changes, unpushed commits)

Include repos whose HEAD commits are not tagged:

    $ mrh --pending --ignore-untracked --untagged-head
    foo (uncommitted changes, unpushed commits, untagged HEAD)
    bar (untagged HEAD)

Check which repos have unfetched commits,
a relatively slow operation when the remote is on the network:

    $ mrh --access-remote ssh-key
    qux (unfetched commits)

For cases where JSON or YAML output is desired,
use `--output-json` or `--output-yaml` flags, respectively.


## Notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories


## Installation

You will need to first install a few packages before you can build mrh.
On Debian/Ubuntu, here how you do:

    apt install cmake libssl-dev pkg-config gcc

These are needed by `libssh2-sys` crate,
which itself is ultimately needed by the git2 crate.

Proceed to build and install mrh
(assuming you have the [Rust toolchain installed][install]):

    cargo install mrh

JSON and YAML output formats are behind feature flags:

    cargo install mrh --features "yaml json"

NOTE: minimum required rustc is v1.36.0,
due to use of new libstd features by [smallvec],
a transitive dependency of git2.

For library usage, check them [API docs][docs].

[percent-encoding]: https://crates.io/crates/percent-encoding
[install]: https://www.rust-lang.org/en-US/install.html
[docs]: https://docs.rs/mrh


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
