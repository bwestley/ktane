use std::process::Command;
fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    let a = String::from_utf8(output.stdout).unwrap();
    let git_hash = a.trim_end();
    if git_hash.len() != 7 || git_hash.contains(|c: char| !c.is_ascii_hexdigit()) {
        println!("cargo:warning=Invalid git hash \"{}\"", git_hash);
        println!("cargo:rustc-env=GIT_HASH=");
    } else {
        println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    }
}
