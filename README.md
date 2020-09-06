# giss
*giss* is command line client to list GitHub issues and pull requests.

## Usage
All commands requires a valid GitHub token. The application will automatically read the environment variable
`GITHUB_TOKEN`, but it can also be given when invoking the application with the `-t` or `--token` flag.
### List Issues & Pull Requests
By default, simply invoking the name of the binary, `giss`, will list issues in the current repo. If the command is not
invoked from a Git repository, an explicit repository will have to given as an argument.

- `giss` - List open issues in current repo
- `giss mantono/giss` - List open issues in repository _giss_ that belongs to user/organization _mantono_
- `giss apple` - List open issues in any repository in organization _apple_
- `giss apple microsoft google` - List open issues in any repository in organizations _apple_, _microsoft_ and _google_
- `giss -c` - List only closed issues and pull requests in current repo
- `giss -A` - List both open and closed issues and pull requests in current repo
- `giss -a` - List only open issues and pull requests assigned to user\* in current repo
- `giss -i` - List only open pull requests in current repo
- `giss -p` - List only open pull requests in current repo
- `giss -r` - List review requests for user\*
- `giss -a kotlin` - List all open issues and pull requests assigned to user in any repository in orgranization _kotlin_

\*the user is determined by the owner of the token

```
USAGE:
    giss [FLAGS] [OPTIONS] [target]...

FLAGS:
    -A, --all
            Show all issues and pull requests and do not filter by open or closed state

    -a, --assigned
            Only include issues and pull requests assigned to user

    -c, --closed
            Only show issues and pull requests in state closed or merged

    -h, --help
            Prints help information

    -i, --issues
            Only list issues

    -o, --open
            Only show issues and pull requests in state open. This is enabled by default

    -p, --pull-requests
            Only list pull requests

    -r, --review-requests
            Only show pull requests where the user has been requested to review it

    -V, --version
            Prints version information


OPTIONS:
    -n, --limit <limit>
            Limit how many issues that should be listed [default: 10]

    -t, --token <token>
            GitHub API token [env: GITHUB_TOKEN]

    -v, --verbosity <verbosity>
            Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level
            configured via RUST_LOG overrides this setting. [default: 1]

ARGS:
    <target>...
            Name of the targets for the action. Can be either a single repository or one or multiple organizations or
            owners. Any repository specified must be qualified with the owner or organization name. For example
            'org/repo'. If action is 'create' then only a repository will be accepted. When no target is specified,
            repository in current directory will be used, if possible.
```

## Building
The application is built with cargo. Simply run the following command in the project directory.
```bash
cargo build --release
```
A binary will be created and put in directory `target/release`.