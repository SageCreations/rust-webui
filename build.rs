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
        .file(src_dir.join("civetweb").join("civetweb.c")) // WebUI's bundled HTTP server
        .include(webui_src.join("include"))
        // civetweb is bundled inside webui's src/
        .include(src_dir.join("civetweb"))
        .define("NO_SSL", None) // disable civetweb's own TLS; we use WEBUI_TLS
        .define("USE_WEBSOCKET", None)
        .define("NDEBUG", None)
        .warnings(false) // suppress warnings from C code we don't own
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
    // Bindgen setup
    // -----------------------------------------------------------------------
    //let out_dir = std::env::var("OUT_DIR").unwrap();
    let bindings = bindgen::Builder::default()
        .header("webui-src/include/webui.h")
        // Types List
        .allowlist_type("webui_browser")
        .allowlist_type("webui_runtime")
        .allowlist_type("webui_event")
        .allowlist_type("webui_config")
        .allowlist_type("webui_event_t")
        .allowlist_type("webui_logger_level")
        // Function API List
        .allowlist_function("webui_new_window")
        .allowlist_function("webui_new_window_id")
        .allowlist_function("webui_get_new_window_id")
        .allowlist_function("webui_bind")
        .allowlist_function("webui_set_context")
        .allowlist_function("webui_get_context")
        .allowlist_function("webui_get_best_browser")
        .allowlist_function("webui_show")
        .allowlist_function("webui_show_client")
        .allowlist_function("webui_show_browser")
        .allowlist_function("webui_start_server")
        .allowlist_function("webui_show_wv")
        .allowlist_function("webui_set_kiosk")
        .allowlist_function("webui_focus")
        .allowlist_function("webui_set_custom_parameters")
        .allowlist_function("webui_set_high_contrast")
        .allowlist_function("webui_set_resizable")
        .allowlist_function("webui_is_high_contrast")
        .allowlist_function("webui_browser_exist")
        .allowlist_function("webui_wait")
        .allowlist_function("webui_wait_async")
        .allowlist_function("webui_close")
        .allowlist_function("webui_minimize")
        .allowlist_function("webui_maximize")
        .allowlist_function("webui_close_client")
        .allowlist_function("webui_destroy")
        .allowlist_function("webui_exit")
        .allowlist_function("webui_set_root_folder")
        .allowlist_function("webui_set_browser_folder")
        .allowlist_function("webui_set_default_root_folder")
        .allowlist_function("webui_set_close_handler_wv")
        .allowlist_function("webui_set_file_handler")
        .allowlist_function("webui_set_file_handler_window")
        .allowlist_function("webui_interface_set_response_file_handler")
        .allowlist_function("webui_is_shown")
        .allowlist_function("webui_set_timeout")
        .allowlist_function("webui_set_icon")
        .allowlist_function("webui_encode")
        .allowlist_function("webui_decode")
        .allowlist_function("webui_free")
        .allowlist_function("webui_malloc")
        .allowlist_function("webui_memcpy")
        .allowlist_function("webui_send_raw")
        .allowlist_function("webui_send_raw_client")
        .allowlist_function("webui_set_hide")
        .allowlist_function("webui_set_size")
        .allowlist_function("webui_set_minimum_size")
        .allowlist_function("webui_set_position")
        .allowlist_function("webui_set_center")
        .allowlist_function("webui_set_profile")
        .allowlist_function("webui_set_proxy")
        .allowlist_function("webui_get_url")
        .allowlist_function("webui_open_url")
        .allowlist_function("webui_set_public")
        .allowlist_function("webui_navigate")
        .allowlist_function("webui_navigate_client")
        .allowlist_function("webui_clean")
        .allowlist_function("webui_delete_all_profiles")
        .allowlist_function("webui_delete_profile")
        .allowlist_function("webui_get_parent_process_id")
        .allowlist_function("webui_get_child_process_id")
        .allowlist_function("webui_win32_get_hwnd")
        .allowlist_function("webui_get_hwnd")
        .allowlist_function("webui_get_port")
        .allowlist_function("webui_set_port")
        .allowlist_function("webui_get_free_port")
        .allowlist_function("webui_set_logger")
        .allowlist_function("webui_set_config")
        .allowlist_function("webui_set_event_blocking")
        .allowlist_function("webui_set_frameless")
        .allowlist_function("webui_set_transparent")
        .allowlist_function("webui_get_mime_type")
        .allowlist_function("webui_set_tls_certificate")
        .allowlist_function("webui_run")
        .allowlist_function("webui_run_client")
        .allowlist_function("webui_script")
        .allowlist_function("webui_script_client")
        .allowlist_function("webui_set_runtime")
        .allowlist_function("webui_get_count")
        .allowlist_function("webui_get_int_at")
        .allowlist_function("webui_get_int")
        .allowlist_function("webui_get_float_at")
        .allowlist_function("webui_get_float")
        .allowlist_function("webui_get_string_at")
        .allowlist_function("webui_get_string")
        .allowlist_function("webui_get_bool_at")
        .allowlist_function("webui_get_bool")
        .allowlist_function("webui_get_size_at")
        .allowlist_function("webui_get_size")
        .allowlist_function("webui_return_int")
        .allowlist_function("webui_return_float")
        .allowlist_function("webui_return_string")
        .allowlist_function("webui_return_bool")
        .allowlist_function("webui_get_last_error_number")
        .allowlist_function("webui_get_last_error_message")
        .allowlist_function("webui_interface_bind")
        .allowlist_function("webui_interface_set_response")
        .allowlist_function("webui_interface_is_app_running")
        .allowlist_function("webui_interface_get_window_id")
        .allowlist_function("webui_interface_get_string_at")
        .allowlist_function("webui_interface_get_int_at")
        .allowlist_function("webui_interface_get_float_at")
        .allowlist_function("webui_interface_get_bool_at")
        .allowlist_function("webui_interface_get_size_at")
        .allowlist_function("webui_interface_show_client")
        .allowlist_function("webui_interface_close_client")
        .allowlist_function("webui_interface_send_raw_client")
        .allowlist_function("webui_interface_navigate_client")
        .allowlist_function("webui_interface_run_client")
        .allowlist_function("webui_interface_script_client")
        .generate()
        .expect("failed to generate bindings");
    bindings
        .write_to_file(format!("src/bindings.rs"))
        .expect("failed to write bindings");

    // -----------------------------------------------------------------------
    // Re-run triggers
    // -----------------------------------------------------------------------
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=webui-src/src/webui.c");
    println!("cargo:rerun-if-changed=webui-src/src/civetweb/civetweb.c");
    println!("cargo:rerun-if-changed=webui-src/include/webui.h");
}
