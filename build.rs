use std::env;

fn main() {
    if env::var("CARGO_FEATURE_LOCAL").is_ok() {
        println!("cargo:rustc-env=BN=0");
    } else {
        println!("cargo:rustc-env=BN=1");
    }
}
