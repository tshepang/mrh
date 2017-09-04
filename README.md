# mrh - multiple git repo helper

This tool crawls current and children directories for git repos,
and check if there are changes that are not committed, or pushed.

Show all repos:

    $ mrh
    foo (2 changes, unpushed changes)
    bar
    baz (1 changes)
    qux

Show all repos that have uncommitted and/or unpushed changes:

    $ mrh --changed
    foo (2 changes, unpushed changes)
    baz (1 changes)


## notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories


## installation

Following is the most easy way to install the tool
(assuming you have the [Rust toolchain installed][install]):

    cargo install --git https://github.com/tshepang/mrh

[install]: https://www.rust-lang.org/en-US/install.html
