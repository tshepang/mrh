# mrh - multiple git repo helper

[![Linux build status](https://travis-ci.org/tshepang/mrh.svg?branch=master)](https://travis-ci.org/tshepang/mrh)

This tool crawls current and children directories for git repos,
and checks if there are:
- untracked files
- uncommitted changes
- unpushed commits
- unpulled commits

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


## notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories


## installation

Following is the most easy way to install the tool
(assuming you have the [Rust toolchain installed][install]):

    cargo install mrh


[install]: https://www.rust-lang.org/en-US/install.html
