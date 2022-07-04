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

For cases where JSON output is desired, use `--output-json` flag.


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

JSON output format is behind a feature flag:

    cargo install mrh --features json

NOTE: minimum required rustc is v1.60,
due to using `dep:` syntax in Cargo.toml,
to avoid implicit features names.

For library usage, check them [API docs][docs].

[percent-encoding]: https://crates.io/crates/percent-encoding
[install]: https://www.rust-lang.org/en-US/install.html
[docs]: https://docs.rs/mrh
[due to indexmap]: https://github.com/bluss/indexmap/commit/8a571c6d68cb38c283d563ff6972613e0eea4111


#### License

<sup>
Licensed under either of
<a href="LICENSE-APACHE">Apache License, Version 2.0</a>
or
<a href="LICENSE-MIT">MIT license</a>
at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
