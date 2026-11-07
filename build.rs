fn main() {
    if std::env::var_os("CARGO_FEATURE_ADMIN").is_none() {
        return;
    }
    println!("cargo:rustc-link-arg-bin=xiuxian=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bin=xiuxian=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
    );
}
