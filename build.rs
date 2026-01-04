use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend");

    // 1. Check if wasm-pack is installed globally
    let is_installed = Command::new("wasm-pack")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_installed {
        println!("cargo:warning=wasm-pack is not installed. Frontend build will be skipped.");
        return;
    }

    // 2. Build using the determined command
    println!("cargo:warning=Building frontend with wasm-pack");

    let status = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "frontend"])
        .status()
        .expect("Failed to run wasm-pack command");

    if !status.success() {
        panic!("wasm-pack build failed");
    }
}
