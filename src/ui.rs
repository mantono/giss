use std::{io::Write, sync::mpsc::RecvTimeoutError};
use std::{sync::mpsc::Receiver, time::Duration};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{
    cfg::Config,
    issue::{Assignee, Issue, Label},
    AppErr,
};

pub struct DisplayConfig {
    colors: ColorChoice,
    limit: u32,
}

impl From<&Config> for DisplayConfig {
    fn from(cfg: &Config) -> Self {
        DisplayConfig {
            colors: cfg.colors(),
            limit: cfg.limit(),
        }
    }
}

pub fn display(channel: Receiver<Issue>, cfg: DisplayConfig) -> Result<(), AppErr> {
    let mut limit: u32 = cfg.limit;
    loop {
        if limit == 0 {
            return Ok(());
        }
        match channel.recv_timeout(Duration::from_secs(20)) {
            Ok(issue) => {
                print_issue(issue, true, &cfg);
                limit -= 1;
            }
            Err(e) => {
                return match e {
                    RecvTimeoutError::Timeout => Err(AppErr::Timeout),
                    RecvTimeoutError::Disconnected => Ok(()),
                }
            }
        };
    }
}

fn print_issue(issue: Issue, print_repo: bool, cfg: &DisplayConfig) {
    let use_colors: ColorChoice = cfg.colors;
    let title: String = truncate(issue.title.clone(), 50);
    let assignees: String = issue
        .assignees
        .nodes
        .iter()
        .map(|a: &Assignee| &a.login)
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

    if print_repo {
        write!(&mut stdout, "#{} {}", issue.number, repo).unwrap();
    } else {
        write!(&mut stdout, "#{}", issue.number).unwrap();
    }

    write(&mut stdout, " | ", Some(Color::Green));
    write(&mut stdout, &title, None);

    if !assignees.is_empty() {
        write(&mut stdout, " | ", Some(Color::Green));
        write(&mut stdout, &assignees, Some(Color::Cyan));
    }

    if !labels.is_empty() {
        write(&mut stdout, " | ", Some(Color::Green));
        write(&mut stdout, &labels, Some(Color::Magenta));
    }

    write(&mut stdout, "\n", None);
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
