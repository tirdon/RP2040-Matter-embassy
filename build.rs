use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    #[cfg(not(feature = "skip-cyw43-firmware"))]
    download_cyw43_firmware();

    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}

#[cfg(not(feature = "skip-cyw43-firmware"))]
fn download_cyw43_firmware() {
    let download_folder = "cyw43-firmware";
    let url_base = "https://github.com/embassy-rs/embassy/raw/refs/heads/main/cyw43-firmware";
    let file_names = [
        "43439A0.bin",
        "43439A0_btfw.bin",
        "43439A0_clm.bin",
        "nvram_rp2040.bin",
        "LICENSE-permissive-binary-license-1.0.txt",
        "README.md",
    ];

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={download_folder}");
    std::fs::create_dir_all(download_folder).expect("Failed to create download directory");

    for file in file_names {
        let url = format!("{url_base}/{file}");
        if std::path::Path::new(download_folder).join(file).exists() {
            continue;
        }
        match reqwest::blocking::get(&url) {
            Ok(response) => {
                let content = response.bytes().expect("Failed to read file content");
                let file_path = PathBuf::from(download_folder).join(file);
                std::fs::write(file_path, &content).expect("Failed to write file");
            }
            Err(err) => panic!(
                "Failed to download cyw43 firmware from {url}: {err}",
            ),
        }
    }
}
