// build.rs
// Compiles the WebUI C library from source (via the `webui-src` git submodule)
// so users never need to manually provide a prebuilt binary.
//
// Prerequisites for users:
//   - A C compiler (gcc/clang/MSVC — already required by most Rust projects)
//   - `git submodule update --init --recursive` after cloning this repo
//
// Features:
//   --features tls  →  also links OpenSSL and enables WEBUI_TLS

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let webui_src = manifest_dir.join("webui-src");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let tls = env::var("CARGO_FEATURE_TLS").is_ok();

    // -----------------------------------------------------------------------
    // Sanity check: make sure the submodule was initialised
    // -----------------------------------------------------------------------
    let webui_h = webui_src.join("include").join("webui.h");
    if !webui_h.exists() {
        panic!(
            "\n\nwebui.h not found at {}.\n\
             Did you forget to initialise the submodule?\n\
             Run: git submodule update --init --recursive\n",
            webui_h.display()
        );
    }

    // -----------------------------------------------------------------------
    // Gather C source files
    // -----------------------------------------------------------------------
    let src_dir = webui_src.join("src");

    let mut build = cc::Build::new();
    build
        .file(src_dir.join("webui.c"))
        .file(src_dir.join("civetweb").join("civetweb.c"))  // WebUI's bundled HTTP server
        .include(webui_src.join("include"))
        // civetweb is bundled inside webui's src/
        .include(src_dir.join("civetweb"))
        .define("NO_SSL", None)           // disable civetweb's own TLS; we use WEBUI_TLS
        .define("USE_WEBSOCKET", None)
        .define("NDEBUG", None)
        .warnings(false)                  // suppress warnings from C code we don't own
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
    // TLS support (feature-gated)
    // -----------------------------------------------------------------------
    if tls {
        build.define("WEBUI_TLS", None).define("NO_SSL", None); // override the no-ssl define

        // Find OpenSSL via pkg-config (users need libssl-dev / openssl installed)
        // We use pkg-config output directly to avoid adding another build-dep.
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
    }

    // -----------------------------------------------------------------------
    // Compile to a static library in OUT_DIR
    // -----------------------------------------------------------------------
    build.out_dir(&out_dir).compile("webui-2-static");

    // -----------------------------------------------------------------------
    // Linker flags
    // -----------------------------------------------------------------------
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=webui-2-static");

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
    println!("cargo:rerun-if-changed=webui-src/src/webui.c");
    println!("cargo:rerun-if-changed=webui-src/src/civetweb/civetweb.c");
    println!("cargo:rerun-if-changed=webui-src/include/webui.h");
}