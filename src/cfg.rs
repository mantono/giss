use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use crate::{args::read_repo_from_file, target::Target};
use structopt::StructOpt;

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

    #[structopt(short = "n", long, default_value = "10")]
    limit: usize,

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

    /// Enable colors
    ///
    /// Enable output with colors
    #[structopt(short = "C", long)]
    colors: bool,

    /// Set verbosity level, 0 - 5
    ///
    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long)]
    verbosity: Verbosity,

    /// Prind debug information
    ///
    /// Print debug information about current build for binary, useful for when an issue is
    /// encountered and reported
    #[structopt(short = "D", long)]
    debug: bool,
}

pub enum StateFilter {
    Open,
    Closed,
    All,
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
    pub fn token(&self) -> Option<String> {
        self.token
    }

    pub fn target(&self) -> Vec<Target> {
        if self.target.is_empty() {
            match read_repo_from_file() {
                Some(repo) => match repo.parse::<Target>() {
                    Ok(target) => vec![target],
                    Err(e) => panic!("Unable to convert target"),
                },
                None => panic!("No target found"),
            }
        } else {
            self.target
        }
    }

    pub fn limit(&self) -> usize {
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

    fn all(&self) -> bool {
        !self.issues && !self.pull_requests && !self.pull_requests
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

    pub fn print_debug(&self) -> bool {
        self.print_debug()
    }
}
