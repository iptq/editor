use std::env;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
fn load_bass() {
    let mut project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    project_dir.push("linux");
    project_dir.push("bass24");
    project_dir.push("x64");

    println!("cargo:rustc-link-search={}", project_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=bass");
}

#[cfg(target_os = "windows")]
fn load_bass() {
    let mut project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    project_dir.push("win");
    project_dir.push("bass24");
    project_dir.push("c");
    project_dir.push("x64");

    println!("cargo:rustc-link-search={}", project_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=bass");
}

fn main() {
    load_bass();
}
