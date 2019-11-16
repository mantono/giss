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
}
