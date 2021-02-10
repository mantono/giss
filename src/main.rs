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
use crate::user::fetch_username;
use args::{parse_args, read_repo_from_file};
use cfg::Config;
use dbg::dbg_info;
use list::FilterConfig;
use logger::setup_logging;

fn main() {
    let cfg = Config::from_args();

    if cfg.print_debug() {
        println!("{}", dbg_info());
        std::process::exit(0);
    }

    setup_logging(&cfg.verbosity);

    let token: String = args.value_of("token").expect("No token was present").to_string();
    let targets: Vec<&str> = args.values_of("target").expect("Target must be present").collect();
    let targets: Vec<Target> = validate_targets(targets).expect("Must have valid target(s)");
    let user: String = fetch_username(&token);
    let config = FilterConfig::from_args(&args);
    let colors: bool = args.is_present("colors");

    log::debug!("Config: {:?}", config);
    list::list_issues(&user, &targets, &token, &config, colors)
}
