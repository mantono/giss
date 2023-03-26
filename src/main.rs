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
mod github_resources;
mod issue;
mod list;
mod logger;
mod project;
mod search;
mod sort;
mod target;
mod ui;
mod user;

use crate::structopt::StructOpt;
use cfg::Config;
use issue::Issue;
use list::FilterConfig;
use logger::setup_logging;
use target::Target;
use tokio::runtime::Runtime;
use ui::DisplayConfig;
use user::Username;

#[tokio::main]
async fn main() -> Result<(), AppErr> {
    let cfg: Config = Config::from_args();

    if cfg.print_debug() {
        println!("{}", include!("../target/build_data"));
        return Ok(());
    }

    setup_logging(cfg.verbosity());

    let token: String = cfg.token()?;
    let targets: Vec<Target> = cfg.target()?;
    let user: Option<Username> = cfg.username();
    log::debug!("Config: {:?}", &cfg);

    let filter: FilterConfig = (&cfg).into();
    let display: DisplayConfig = (&cfg).into();

    let bounds: usize = cfg.limit() as usize * 2;
    let (send, recv) = std::sync::mpsc::sync_channel::<Issue>(bounds);

    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        match list::list_issues(send, &user, &targets, &token, &filter).await {
            Ok(_) => log::debug!("API requests completed"),
            Err(e) => log::error!("{:?}", e),
        }
    });

    ui::display(recv, display)?;
    rt.shutdown_background();

    Ok(())
}

#[derive(Debug)]
pub enum AppErr {
    MissingToken,
    TokenWriteError,
    NoTarget,
    InvalidTarget(String),
    ApiError,
    Timeout,
    ChannelError,
    RateLimited,
}
