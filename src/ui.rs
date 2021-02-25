use std::{io::Write, sync::mpsc::RecvTimeoutError};
use std::{sync::mpsc::Receiver, time::Duration};

use itertools::Itertools;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use user::Username;

use crate::{
    cfg::Config,
    issue::{Issue, Label, UserFields},
    search::Type,
    sort::Sorting,
    user, AppErr,
};

pub struct DisplayConfig {
    colors: ColorChoice,
    sorting: Sorting,
    user: Option<Username>,
    limit: u32,
    links: bool,
}

impl From<&Config> for DisplayConfig {
    fn from(cfg: &Config) -> Self {
        DisplayConfig {
            colors: cfg.colors(),
            limit: cfg.limit(),
            user: cfg.username(),
            sorting: cfg.sorting(),
            links: cfg.show_links(),
        }
    }
}

pub fn display(channel: Receiver<Issue>, cfg: DisplayConfig) -> Result<(), AppErr> {
    let mut limit: u32 = cfg.limit * 3;
    let mut queue: Vec<Issue> = Vec::with_capacity(limit as usize);
    while limit > 0 {
        match channel.recv_timeout(Duration::from_secs(20)) {
            Ok(issue) => {
                queue.push(issue);
                limit -= 1;
            }
            Err(e) => match e {
                RecvTimeoutError::Timeout => return Err(AppErr::Timeout),
                RecvTimeoutError::Disconnected => limit = 0,
            },
        };
    }
    queue.sort_unstable_by(|i0, i1| cfg.sorting.sort(i0, i1));
    queue
        .into_iter()
        .unique_by(|i| i.id)
        .take(cfg.limit as usize)
        .for_each(|i| print_issue(i, true, &cfg));
    Ok(())
}

fn print_issue(issue: Issue, print_repo: bool, cfg: &DisplayConfig) {
    let use_colors: ColorChoice = cfg.colors;
    let title: String = truncate(issue.title.clone(), 50);
    let assignees: String = issue
        .assignees
        .nodes
        .iter()
        .map(|a: &UserFields| &a.login)
        .map(|s: &String| format!("{}{}", "@", s))
        .collect::<Vec<String>>()
        .join(", ");

    let repo: String = if print_repo {
        issue.repository.name_with_owner.clone()
    } else {
        String::from("")
    };

    let labels: String = issue
        .labels
        .nodes
        .iter()
        .map(|l: &Label| &l.name)
        .map(|s: &String| format!("{}{}", "#", s))
        .collect::<Vec<String>>()
        .join(", ");

    let mut stdout = StandardStream::stdout(use_colors);

    print_type(&mut stdout, &issue, cfg);

    let target: String = if print_repo {
        format!("#{} {}", issue.number, repo)
    } else {
        format!("#{}", issue.number)
    };

    write(&mut stdout, target.as_str(), None);
    delimiter(&mut stdout);
    write(&mut stdout, &title, None);

    if !assignees.is_empty() {
        delimiter(&mut stdout);
        write(&mut stdout, &assignees, Some(Color::Cyan));
    }

    if !labels.is_empty() {
        delimiter(&mut stdout);
        write(&mut stdout, &labels, Some(Color::Magenta));
    }

    if cfg.links {
        delimiter(&mut stdout);
        write(&mut stdout, &issue.link(), Some(Color::Blue));
    }

    write(&mut stdout, "\n", None);
}

fn print_type(stream: &mut StandardStream, issue: &Issue, cfg: &DisplayConfig) {
    let kind: Type = match issue.kind {
        Type::Issue => Type::Issue,
        _ => match &cfg.user {
            Some(user) => match issue.has_review_request(&user.0) {
                true => Type::ReviewRequest,
                false => Type::PullRequest,
            },
            None => Type::PullRequest,
        },
    };

    match kind {
        crate::search::Type::Issue => write(stream, "I ", Some(Color::Blue)),
        crate::search::Type::PullRequest => write(stream, "P ", Some(Color::Magenta)),
        crate::search::Type::ReviewRequest => {
            write(stream, "P", Some(Color::Magenta));
            write(stream, "R", Some(Color::Yellow));
        }
    };
    write(stream, "| ", Some(Color::Green));
}

fn delimiter(stream: &mut StandardStream) {
    write(stream, " | ", Some(Color::Green));
}

fn truncate(string: String, max_length: usize) -> String {
    let new_length: usize = std::cmp::min(string.len(), max_length);
    if new_length < string.len() {
        string[..new_length].to_string()
    } else {
        string
    }
}

fn write(stream: &mut StandardStream, content: &str, color: Option<Color>) {
    stream.set_color(ColorSpec::new().set_fg(color)).unwrap();
    write!(stream, "{}", content).unwrap();
}
