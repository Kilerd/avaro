use chrono::Local;

fn main() {
    let now = Local::now();
    let build_version = std::fs::read_to_string("../.build_version").unwrap_or(env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rerun-if-changed=../.build_version");
    println!("cargo:rustc-env=ZHANG_BUILD_DATE={}", now.format("%Y-%m-%d"));
    println!("cargo:rustc-env=ZHANG_BUILD_VERSION={}", build_version);
}
