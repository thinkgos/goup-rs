const MIN_VERSION: &str = "1.74";

fn main() -> Result<(), shadow_rs::ShadowError> {
    match version_check::is_min_version(MIN_VERSION) {
        Some(true) => {}
        // rustc version too small or can't figure it out
        _ => {
            eprintln!("'goup' requires rustc >= {}", MIN_VERSION);
            std::process::exit(1);
        }
    }
    shadow_rs::new()
}
