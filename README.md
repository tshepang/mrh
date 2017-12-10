# mrh - multiple git repo helper

[![Linux build status](https://travis-ci.org/tshepang/mrh.svg?branch=master)](https://travis-ci.org/tshepang/mrh)

This tool crawls current and children directories for git repos,
and checks if there are:
- untracked files
- uncommitted changes
- unpushed commits
- unpulled commits
- added files (git index)
- deleted files
- renamed files
- untagged HEAD (via an optional flag)

Show all repos:

    $ mrh
    foo (untracked files, uncommitted changes, unpushed commits)
    bar
    baz (untracked files)
    qux

Only show those repos that are pending action:

    $ mrh --pending
    foo (untracked files, uncommitted changes, unpushed commits)
    baz (untracked files)

Ignore untracked files in results:

    $ mrh --pending --ignore-untracked
    foo (uncommitted changes, unpushed commits)

Include repos whose HEAD commits are not tagged:

    $ mrh --pending --ignore-untracked --untagged-head
    foo (uncommitted changes, unpushed commits, untagged HEAD)
    bar (untagged HEAD)


## Notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories


## Installation

Following is the most easy way to install the tool
(assuming you have the [Rust toolchain installed][install]):

    cargo install mrh


[install]: https://www.rust-lang.org/en-US/install.html


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
