pub fn dbg_info() -> String {
    std::fs::read_to_string("target/build_data").unwrap()
}
