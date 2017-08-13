# mrh - multiple git repo helper

This tool crawls children directories for git repos,
and pokes them for various bits of info.

**This is very unfinished**,
and only displays paths of git repos;
the following are just future plans:

Show all repos:

    $ mrh
    foo <latest tag> (checkout not matching tag, untracked files)
    bar untagged (uncommitted changes, unpushed commits)
    baz v3.0.0

    $ mrh all --quiet/--silent
    foo
    bar
    baz

Show all repos that have uncommitted changes:

    mrh uncommitted

Show all repos that have unpushed commits:

    mrh unpushed

Jump to repo:

    mrh cd <repo name>

## notes

- Ignores unreadable files/directories without warning
- Ignores bare git repositories
