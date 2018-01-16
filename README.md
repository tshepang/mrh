# mrh - Multi-(git)Repo Helper

[![Linux build status](https://travis-ci.org/tshepang/mrh.svg?branch=master)](https://travis-ci.org/tshepang/mrh)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/tshepang/mrh?svg=true)](https://ci.appveyor.com/project/tshepang/mrh)

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
- unpulled commits (optional)

I also offers a command line tool with all those features,
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

For cases where JSON or YAML output is desired,
use `--output-json` or `--output-yaml` flags, respectively.


## Notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories


## Installation

Following is the most easy way to install the tool
(assuming you have the [Rust toolchain installed][install]):

    cargo install mrh

JSON and YAML output formats are behind feature flags:

    cargo install mrh --features "yaml json"

NOTE: minimum required rustc is v1.20, due to a transitive dependency, bitflags.

For library usage, check them [API docs][docs].

[install]: https://www.rust-lang.org/en-US/install.html
[docs]: https://docs.rs/crate/mrh


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
