use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend");

    // 1. Check if wasm-pack is installed globally
    let is_installed = Command::new("wasm-pack")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let wasm_pack_cmd = if is_installed {
        "wasm-pack".to_string()
    } else {
        // 2. Install if missing (Only for Linux x86_64, which Shuttle uses)
        let out_dir = env::var("OUT_DIR").unwrap();
        let install_dir = Path::new(&out_dir).join("wasm-pack-install");
        // Ensure directory exists
        fs::create_dir_all(&install_dir).unwrap();

        let wasm_pack_path = install_dir.join("wasm-pack");

        if !wasm_pack_path.exists() {
            let version = "v0.13.1";
            let url = format!(
                "https://github.com/rustwasm/wasm-pack/releases/download/{}/{}-{}.tar.gz",
                version, "wasm-pack", version
            );
            let tarball = install_dir.join("wasm-pack.tar.gz");

            println!("cargo:warning=Downloading wasm-pack from {}", url);
            let status = Command::new("curl")
                .args(&["-L", "-o", tarball.to_str().unwrap(), &url])
                .status()
                .expect("Failed to run curl to download wasm-pack");

            if !status.success() {
                // If curl fails (e.g. non-linux for some reason, or no network), panic.
                // But for local mac dev (if wasm-pack missing), this linux binary won't work anyway.
                // We assume this branch is primarily for Shuttle/CI on Linux.
                panic!("Failed to download wasm-pack from {}", url);
            }

            // Extract
            let status = Command::new("tar")
                .args(&[
                    "-xzf",
                    tarball.to_str().unwrap(),
                    "-C",
                    install_dir.to_str().unwrap(),
                    "--strip-components=1",
                ])
                .status()
                .expect("Failed to run tar to extract wasm-pack");

            if !status.success() {
                panic!("Failed to extract wasm-pack");
            }
        }

        wasm_pack_path.to_str().unwrap().to_string()
    };

    // 3. Build using the determined command
    println!("cargo:warning=Building frontend with {}", wasm_pack_cmd);

    // We need to ensure the executed binary runs with the current directory set properly
    // wasm-pack build <path> works from anywhere.

    let status = Command::new(wasm_pack_cmd)
        .args(&["build", "--target", "web", "frontend"])
        .status()
        .expect("Failed to run wasm-pack command");

    if !status.success() {
        panic!("wasm-pack build failed");
    }
}
