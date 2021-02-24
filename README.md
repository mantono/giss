# giss
*giss* is command line client to list GitHub issues and pull requests.

## Usage
All commands requires a valid [GitHub API token](https://github.com/settings/tokens). The application will automatically read the environment variable
`GITHUB_TOKEN`, but it can also be given when invoking the application with the `-t` or `--token` flag. The token does not need any permission for reading public repositories, but for private repositories is the `repo` permission required.

### List Issues & Pull Requests
By default, simply invoking the name of the binary, `giss`, will list tickets that are either
- issues
- pull requets
- review requests

in the current repo.
If the command is not invoked from a Git repository, an explicit repository will have to given as an argument.

- `giss` - List open tickets in current repo
- `giss mantono/giss` - List open tickets in repository _giss_ that belongs to user/organization _mantono_
- `giss apple` - List open tickets in any repository in organization _apple_
- `giss apple microsoft google` - List tickets in any repository in organizations _apple_, _microsoft_ and _google_
- `giss rust-lang/rust apple/swift golang/go` - List open tickets in repositories for rust, swift and go
- `giss -c` - List only closed tickets in current repo
- `giss -oc` - List both open and closed tickets in current repo
- `giss -a` - List only open tickets assigned to user\* in current repo
- `giss -i` - List only open issues in current repo
- `giss -p` - List only open pull requests in current repo
- `giss -r` - List only review requests for user\*
- `giss -a kotlin` - List all open tickets assigned to user in any repository in orgranization _kotlin_

\*the user is determined by the owner of the token, unless overriden with the `--user` flag.

See `giss --help` for all available options.

```
USAGE:
    giss [FLAGS] [OPTIONS] [--] [target]...

FLAGS:
    -a, --assigned
            Assigned only

            Only include issues and pull requests assigned to user
    -c, --closed
            Show closed issues or pull requests

            Include issues, pull request or review requests that are closed or merged
    -D, --debug
            Prind debug information

            Print debug information about current build for binary, useful for when an issue is encountered and reported
    -h, --help
            Prints help information

    -i, --issues
            List issues

    -o, --open
            Show open issues or pull requests

            Include issues, pull request or review requests that are open. If neither this flag nor --closed/-c is
            given, default behavior will be to display open issues or pull requests.
    -p, --pull-requests
            List pull requests

    -r, --review-requests
            List review requests

    -V, --version
            Prints version information


OPTIONS:
        --colors <colors>
            Set use of colors

            Enable or disable output with colors. By default, the application will try to figure out if colors are
            supported by the terminal in the current context, and use it if possible. Possible values are "on", "true",
            "off", "false", "auto". [default: auto]
    -l, --labels <labels>...
            Filter by label

            Only include issues, pull requests or review reuests which has (all) the given label(s).
    -n, --limit <limit>
            Limit the number of issues or pull requests to list [default: 10]

    -O, --order <order>
            Ordering

            Can be either ascending (asc|ascending) or decending (desc|descending)
    -P, --project <project>
            Filter by project

            Only include isses, pull request or review requests which is assoicated with the given project.
    -S, --search <search>
            Search

            Search by a string, which must be present either in the title or the body of an issue or pull request.
    -s, --sort-by <sort-by>
            Sort by

            Sort by any of the following properties; "created", "updated", "comments", "reactions"
    -t, --token <token>
            GitHub API token

            API token that will be used when authenticating towards GitHub's API [env: GITHUB_TOKEN]
    -u, --user <user>
            Username

            Username to use for the query. Will default to the username for the user of the token.
    -v, --verbosity <verbosity>
            Set verbosity level, 0 - 5

            Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level
            configured via RUST_LOG overrides this setting. [default: 1]

ARGS:
    <target>...
            Name of target(s)

            Name of the targets for the action. Can be a combination of one or several repositories, organizations or
            users. Any repository specified must be qualified with the owner or organization name. For example
            'org/repo'. When no target is specified, repository in current directory will be used, if possible.
```

## Building
The application is built with cargo. Simply run the following command in the project directory.
```bash
cargo build --release
```
A binary will be created and put in directory `target/release`.


## Install
Run `cargo install --path .`
