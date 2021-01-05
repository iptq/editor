use std::env;
use std::path::PathBuf;

fn main() {
    let mut project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    project_dir.push("bass24");
    project_dir.push("c");
    project_dir.push("x64");

    println!("cargo:rustc-link-search={}", project_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=bass");
}
