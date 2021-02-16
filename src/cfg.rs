use std::str::FromStr;

use crate::{args::read_repo_from_file, list::StateFilter, target::Target, user::Username, AppErr};
use structopt::StructOpt;
use termcolor::ColorChoice;

#[derive(StructOpt, Debug)]
#[structopt(name = "giss", author, about)]
pub struct Config {
    /// Name of target(s)
    ///
    /// Name of the targets for the action. Can be a combination of one or several repositories,
    /// organizations or users. Any repository specified must be qualified with the owner or
    /// organization name. For example 'org/repo'. When no target is specified, repository in
    /// current directory will be used, if possible.
    target: Vec<Target>,

    /// GitHub API token
    ///
    /// API token that will be used when authenticating towards GitHub's API
    #[structopt(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<String>,

    /// Assigned only
    ///
    /// Only include issues and pull requests assigned to user
    #[structopt(short, long)]
    assigned: bool,

    /// Limit the number of issues or pull requests to list
    #[structopt(short = "n", long, default_value = "10")]
    limit: u32,

    /// Show open issues or pull requests
    ///
    /// Include issues, pull request or review requests that are open. If neither this flag nor
    /// --closed/-c is given, default behavior will be to display open issues or pull requests.
    #[structopt(short, long)]
    open: bool,

    /// Show closed issues or pull requests
    ///
    /// Include issues, pull request or review requests that are closed or merged
    #[structopt(short, long)]
    closed: bool,

    /// List issues
    #[structopt(short, long)]
    issues: bool,

    /// List pull requests
    #[structopt(short, long)]
    pull_requests: bool,

    /// List review requests
    #[structopt(short, long)]
    review_requests: bool,

    /// Username
    ///
    /// Username to use for the query. Will default to the username for the user of the token.
    #[structopt(short, long)]
    user: Option<Username>,

    /// Set use of colors
    ///
    /// Enable or disable output with colors. By default, the application will
    /// try to figure out if colors are supported by the terminal in the current context, and use it
    /// if possible.
    /// Possible values are "on", "true", "off", "false", "auto".
    #[structopt(long = "colors", default_value = "auto")]
    colors: Flag,

    /// Set verbosity level, 0 - 5
    ///
    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long, default_value = "1")]
    verbosity: Verbosity,

    /// Prind debug information
    ///
    /// Print debug information about current build for binary, useful for when an issue is
    /// encountered and reported
    #[structopt(short = "D", long)]
    debug: bool,
}

#[derive(Debug, Copy, Clone)]
enum Flag {
    True,
    False,
    Auto,
}

impl FromStr for Flag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "true" | "on" => Ok(Flag::True),
            "false" | "off" => Ok(Flag::False),
            "auto" => Ok(Flag::Auto),
            _ => Err(format!("Unrecognized option {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct Verbosity(u8);

impl Verbosity {
    pub fn level(&self) -> u8 {
        self.0
    }
}

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>() {
            Ok(n) => match n {
                0..=5 => Ok(Verbosity(n)),
                _ => Err(format!("Unsupported verbosity level '{}'", n)),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Config {
    pub fn token(&self) -> Result<String, AppErr> {
        self.token.clone().ok_or(AppErr::MissingToken)
    }

    pub fn username(&self) -> Option<Username> {
        match &self.user {
            Some(user) => Some(user.clone()),
            None => match self.token.clone() {
                Some(token) => futures::executor::block_on(Username::from_token(&token)).ok(),
                None => None,
            },
        }
    }

    pub fn target(&self) -> Result<Vec<Target>, AppErr> {
        if self.target.is_empty() {
            match read_repo_from_file() {
                Some(repo) => match repo.parse::<Target>() {
                    Ok(target) => Ok(vec![target]),
                    Err(_) => Err(AppErr::InvalidTarget(repo)),
                },
                None => Err(AppErr::NoTarget),
            }
        } else {
            Ok(self.target.clone())
        }
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn state(&self) -> StateFilter {
        if self.open && self.closed {
            StateFilter::All
        } else if self.closed {
            StateFilter::Closed
        } else {
            StateFilter::Open
        }
    }

    pub fn assigned_only(&self) -> bool {
        self.assigned
    }

    fn all(&self) -> bool {
        !self.issues && !self.pull_requests && !self.review_requests
    }

    pub fn issues(&self) -> bool {
        self.issues || self.all()
    }

    pub fn reviews(&self) -> bool {
        self.review_requests || self.all()
    }

    pub fn pulls(&self) -> bool {
        self.pull_requests || self.all()
    }

    pub fn verbosity(&self) -> &Verbosity {
        &self.verbosity
    }

    pub fn colors(&self) -> ColorChoice {
        match self.colors {
            Flag::True => ColorChoice::Always,
            Flag::False => ColorChoice::Never,
            Flag::Auto => ColorChoice::Auto,
        }
    }

    pub fn print_debug(&self) -> bool {
        self.debug
    }
}
