pub fn dbg_info() -> String {
    format!(
        "Crate version {}.\nBuilt from commit {} by {} for target {} with profile '{}' and features = {:?}.",
        crate_version!(),
        built_info::GIT_VERSION.unwrap(),
        built_info::RUSTC_VERSION,
        built_info::TARGET,
        built_info::PROFILE,
        built_info::FEATURES
    )
}

#[allow(dead_code)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
