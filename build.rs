// build.rs
// Locates and links the WebUI static library.
//
// Search order for the library file:
//   1. The directory set by the WEBUI_DIR environment variable
//   2. The crate root directory (next to Cargo.toml)
//   3. Standard system library search paths (fallback)
//
// Feature flags:
//   --features tls  →  links webui-2-secure-static instead of webui-2-static

use std::env;
use std::path::PathBuf;

fn main() {
    // -----------------------------------------------------------------------
    // Choose library name based on features
    // -----------------------------------------------------------------------
    let lib_name = if env::var("CARGO_FEATURE_TLS").is_ok() {
        "webui-2-secure-static"
    } else {
        "webui-2-static"
    };

    // -----------------------------------------------------------------------
    // Tell Cargo where to find the library
    // -----------------------------------------------------------------------

    // 1. WEBUI_DIR environment variable (highest priority)
    if let Ok(dir) = env::var("WEBUI_DIR") {
        println!("cargo:rustc-link-search=native={}", dir);
    }

    // 2. Crate root (where Cargo.toml lives)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    println!("cargo:rustc-link-search=native={}", manifest_dir);

    // 3. A `lib/` subdirectory of the crate root (common convention)
    let lib_dir = PathBuf::from(&manifest_dir).join("lib");
    if lib_dir.exists() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    // -----------------------------------------------------------------------
    // Link the WebUI library
    // -----------------------------------------------------------------------
    println!("cargo:rustc-link-lib=static={}", lib_name);

    // -----------------------------------------------------------------------
    // Platform-specific system dependencies required by WebUI
    // -----------------------------------------------------------------------
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    match target_os.as_str() {
        "windows" => {
            // WebUI needs these Windows system libraries
            for lib in &["ws2_32", "user32", "ole32", "shell32", "advapi32"] {
                println!("cargo:rustc-link-lib={}", lib);
            }
            // MSVC also needs the C++ runtime for WebUI internals
            if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc" {
                println!("cargo:rustc-link-lib=vcruntime");
            }
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
            println!("cargo:rustc-link-lib=framework=WebKit");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=dl");
        }
        _ => {}
    }

    // -----------------------------------------------------------------------
    // Re-run if any of these change
    // -----------------------------------------------------------------------
    println!("cargo:rerun-if-env-changed=WEBUI_DIR");
    println!("cargo:rerun-if-changed=build.rs");
}
