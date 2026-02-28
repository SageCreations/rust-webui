// build.rs
// Compiles the WebUI C library from vendored source files.
// The C sources live in vendor/webui/ and are committed directly to this repo,
// so users only need `cargo add` — no submodule init, no manual downloads.
//
// Features:
//   --features tls  →  also links OpenSSL and enables WEBUI_TLS

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_dir = manifest_dir.join("vendor").join("webui");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let tls = env::var("CARGO_FEATURE_TLS").is_ok();

    // -----------------------------------------------------------------------
    // Sanity check: make sure vendor files are present
    // -----------------------------------------------------------------------
    let webui_h = vendor_dir.join("include").join("webui.h");
    if !webui_h.exists() {
        panic!(
            "\n\nwebui.h not found at {}.\n\
             The vendor directory appears to be missing or incomplete.\n\
             Please file an issue at https://github.com/webui-dev/rust-webui\n",
            webui_h.display()
        );
    }

    // -----------------------------------------------------------------------
    // Compile C sources
    // -----------------------------------------------------------------------
    let src_dir = vendor_dir.join("src");

    let mut build = cc::Build::new();
    build
        .file(src_dir.join("webui.c"))
        .file(src_dir.join("civetweb").join("civetweb.c"))
        .include(vendor_dir.join("include"))
        .include(src_dir.join("civetweb"))
        .define("NO_SSL", None)
        .define("USE_WEBSOCKET", None)
        .define("NDEBUG", None)
        .warnings(false)
        .opt_level(2);

    // -----------------------------------------------------------------------
    // Platform-specific C flags
    // -----------------------------------------------------------------------
    match target_os.as_str() {
        "windows" => {
            build
                .define("_WIN32_WINNT", Some("0x0600"))
                .define("WEBUI_STATIC", None);
        }
        "macos" => {
            build.define("__APPLE__", None);
        }
        "linux" => {
            build.define("__linux__", None);
        }
        _ => {}
    }

    // -----------------------------------------------------------------------
    // TLS (optional feature)
    // -----------------------------------------------------------------------
    if tls {
        build.define("WEBUI_TLS", None);
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
    }

    // -----------------------------------------------------------------------
    // Compile and link
    // -----------------------------------------------------------------------
    build.out_dir(&out_dir).compile("webui-2-static");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=webui-2-static");

    // -----------------------------------------------------------------------
    // Platform system libraries
    // -----------------------------------------------------------------------
    match target_os.as_str() {
        "windows" => {
            for lib in &["ws2_32", "user32", "ole32", "shell32", "advapi32"] {
                println!("cargo:rustc-link-lib={}", lib);
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
    // Re-run triggers
    // -----------------------------------------------------------------------
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/webui/src/webui.c");
    println!("cargo:rerun-if-changed=vendor/webui/src/civetweb/civetweb.c");
    println!("cargo:rerun-if-changed=vendor/webui/include/webui.h");
}