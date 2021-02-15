#[macro_use]
extern crate clap;
extern crate dirs_next;
extern crate lazy_static;
extern crate log;
extern crate regex;
extern crate structopt;

mod api;
mod args;
mod cfg;
mod dbg;
mod github_resources;
mod issue;
mod list;
mod logger;
mod search;
mod target;
mod user;

use crate::structopt::StructOpt;
use cfg::Config;
use dbg::dbg_info;
use list::FilterConfig;
use logger::setup_logging;
use target::Target;
use termcolor::ColorChoice;
use user::Username;

fn main() -> Result<(), AppErr> {
    let cfg: Config = Config::from_args();

    if cfg.print_debug() {
        println!("{}", dbg_info());
        return Ok(());
    }

    setup_logging(cfg.verbosity());

    let token: String = cfg.token()?;
    let targets: Vec<Target> = cfg.target()?;
    let user: Option<Username> = cfg.username();
    let colors: ColorChoice = cfg.colors();
    log::debug!("Config: {:?}", &cfg);

    let filter: FilterConfig = cfg.into();
    list::list_issues(&user, &targets, &token, &filter, colors);

    Ok(())
}

#[derive(Debug)]
pub enum AppErr {
    MissingToken,
    TokenWriteError,
    NoTarget,
    InvalidTarget(String),
    ApiError,
}
