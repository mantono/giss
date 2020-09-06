use built::Options;
use std::path::{Path, PathBuf};

fn main() {
    let mut default: Options = Options::default();
    let options = default
        .set_compiler(true)
        .set_cfg(true)
        .set_ci(false)
        .set_dependencies(false)
        .set_git(true)
        .set_env(true)
        .set_features(true);

    let src: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let dst: PathBuf = Path::new(&std::env::var("OUT_DIR").unwrap()).join("built.rs");

    built::write_built_file_with_opts(&options, &src, &dst).expect("Failed to acquire build-time information");
}
