fn main() {
    // Fetch git tag/commit with `git describe --all --always --long --broken --dirty`
    for (key, value) in std::env::vars().into_iter().filter(|(k, _)| k.starts_with("CARGO")) {
        println!("{key}: {value}");
    }

    std::fs::write("target/build_data", "foo");
}
