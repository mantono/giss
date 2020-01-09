# giss
*giss* is command line client to list GitHub issues and pull requests.

## Usage
All commands requires a valid GitHub token. The application will automatically read the environment variable
`GITHUB_TOKEN`, but it can also be given when invoking the application with the `-t` or `--token` flag. 
### List Issues & Pull Requests
By default, simply invoking the name of the binary,`giss`, will list issues in the current repo. If the command is not
invoked from a Git repository, an explicit repository will have to given as an argument.

- `giss` - List open issues in current repo
- `giss list mantono/giss` - List open issues in repository _giss_ that belongs to user/organization _mantono_
- `giss list apple` - List open issues in any repository in organization _apple_
- `giss list apple microsoft google` - List open issues in any repository in organizations _apple_, _microsoft_ and _google_
- `giss -c` - List only closed issues in current repo
- `giss -A` - List both open and closed issues in current repo
- `giss -a` - List only open issues assigned to user (owner of token) in current repo
- `giss -p` - List only open pull requests in current repo
- `giss list kotlin -ipa` - List all open issues and pull requests assigned to user in any repository in orgranization _kotlin_

### Unimplemented Features
The `--review-requests` flag has currently not been implemented

See `giss --help` for documentation of all available flags and commands.
## Building
The application is built with cargo. Simply run the following command in the project directory.
```bash
cargo build --release
```
A binary will be created and put in directory `target/release`. 