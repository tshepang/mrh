# mrh - multiple git repo helper

This tool crawls current and children directories for git repos,
and check if there are changes that are not committed.

Show all repos:

    $ mrh
    foo (2 changes)
    bar
    baz (1 changes)

    $ mrh --changed
    foo (2 changes)
    baz (1 changes)

Show all repos that have uncommitted changes:

    mrh --changed

Maybe in future we'll have more informative output:

    $ mrh
    foo <latest tag> (checkout not matching tag, untracked files)
    bar untagged (uncommitted changes, unpushed commits)
    baz v3.0.0

## notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories
