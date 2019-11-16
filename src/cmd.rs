pub mod cmd {
    use std::env;

    pub fn cmd_has(key: &str) -> bool {
        env::args().any(|i| i == key)
    }

    pub fn cmd_read(key: &str) -> Option<String> {
        let args: Vec<String> = env::args().collect();
        let value: Option<&String> = args.iter().skip_while(|i| !i.contains(key)).skip(1).next();
        value.cloned()
    }

    pub fn print_help_text() {
        println!("Help text here")
    }

    pub fn print_no_arg() {
        println!("Expected an argument")
    }

    const EXPECTED_OPTS: &str = "list, add, help";

    pub fn print_invalid_arg(arg: &str) {
        println!(
            "Invalid command '{}', expected one of: {}",
            arg, EXPECTED_OPTS
        )
    }

    pub enum Command {
        List,
        Add,
        Help,
        Invalid(String),
    }

    impl Command {
        pub fn from(value: &str) -> Option<Command> {
            match value {
                "list" | "ls" => Some(Command::List),
                "add" => Some(Command::Add),
                "help" => Some(Command::Help),
                _ => Some(Command::Invalid(value.to_string())),
            }
        }

        pub fn parse() -> Option<Command> {
            let args: Vec<String> = env::args().collect();
            let value: Option<&String> = args.iter().skip(1).next();
            match value {
                Some(cmd) => Command::from(cmd),
                None => None,
            }
        }
    }

    pub enum Flag {
        Global,
        Token(String),
        All,
        Mine,
        Repo(String),
    }
}
