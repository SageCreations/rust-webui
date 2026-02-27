// ffi.rs
// Raw, unsafe 1:1 bindings to webui.h
// Do not use these directly — use the safe wrappers in window.rs and event.rs instead.
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_longlong, c_uint, c_void};

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Direct mirror of `webui_event_t` in C.
/// Must stay `#[repr(C)]` and field-for-field identical to the C struct.
#[repr(C)]
pub struct RawEvent {
    pub window: usize,
    pub event_type: usize,
    pub element: *mut c_char,
    pub event_number: usize,
    pub bind_id: usize,
    pub client_id: usize,
    pub connection_id: usize,
    pub cookies: *mut c_char,
}

// ---------------------------------------------------------------------------
// Enums passed as size_t in the C API
// (C enums are just ints; we wrap them in Rust enums in types.rs)
// ---------------------------------------------------------------------------

/// Mirror of `webui_config` (typedef enum).
/// Passed by value to `webui_set_config`.
#[repr(C)]
pub enum RawConfig {
    ShowWaitConnection = 0,
    UiEventBlocking = 1,
    FolderMonitor = 2,
    MultiClient = 3,
    UseCookies = 4,
    AsynchronousResponse = 5,
}

// ---------------------------------------------------------------------------
// Extern block
// ---------------------------------------------------------------------------

#[link(name = "webui-2-static")]
extern "C" {
    // -- Window creation ------------------------------------------------

    pub fn webui_new_window() -> usize;
    pub fn webui_new_window_id(window_number: usize) -> usize;
    pub fn webui_get_new_window_id() -> usize;

    // -- Binding --------------------------------------------------------

    pub fn webui_bind(
        window: usize,
        element: *const c_char,
        func: Option<unsafe extern "C" fn(*mut RawEvent)>,
    ) -> usize;

    pub fn webui_set_context(window: usize, element: *const c_char, context: *mut c_void);
    pub fn webui_get_context(e: *mut RawEvent) -> *mut c_void;
    pub fn webui_get_best_browser(window: usize) -> usize;

    // -- Show -----------------------------------------------------------

    pub fn webui_show(window: usize, content: *const c_char) -> bool;
    pub fn webui_show_client(e: *mut RawEvent, content: *const c_char) -> bool;
    pub fn webui_show_browser(window: usize, content: *const c_char, browser: usize) -> bool;
    pub fn webui_start_server(window: usize, content: *const c_char) -> *const c_char;
    pub fn webui_show_wv(window: usize, content: *const c_char) -> bool;

    // -- Window appearance / behavior -----------------------------------

    pub fn webui_set_kiosk(window: usize, status: bool);
    pub fn webui_set_custom_parameters(window: usize, params: *mut c_char);
    pub fn webui_set_high_contrast(window: usize, status: bool);
    pub fn webui_set_resizable(window: usize, status: bool);
    pub fn webui_is_high_contrast() -> bool;
    pub fn webui_browser_exist(browser: usize) -> bool;
    pub fn webui_set_hide(window: usize, status: bool);
    pub fn webui_set_size(window: usize, width: c_uint, height: c_uint);
    pub fn webui_set_minimum_size(window: usize, width: c_uint, height: c_uint);
    pub fn webui_set_position(window: usize, x: c_uint, y: c_uint);
    pub fn webui_set_center(window: usize);
    pub fn webui_set_frameless(window: usize, status: bool);
    pub fn webui_set_transparent(window: usize, status: bool);

    // -- Wait / close ---------------------------------------------------

    pub fn webui_wait();
    pub fn webui_wait_async() -> bool;
    pub fn webui_close(window: usize);
    pub fn webui_minimize(window: usize);
    pub fn webui_maximize(window: usize);
    pub fn webui_close_client(e: *mut RawEvent);
    pub fn webui_destroy(window: usize);
    pub fn webui_exit();

    // -- File server / root folder --------------------------------------

    pub fn webui_set_root_folder(window: usize, path: *const c_char) -> bool;
    pub fn webui_set_browser_folder(path: *const c_char);
    pub fn webui_set_default_root_folder(path: *const c_char) -> bool;

    // -- Handlers -------------------------------------------------------

    pub fn webui_set_close_handler_wv(
        window: usize,
        close_handler: Option<unsafe extern "C" fn(usize) -> bool>,
    );

    pub fn webui_set_file_handler(
        window: usize,
        handler: Option<unsafe extern "C" fn(*const c_char, *mut c_int) -> *const c_void>,
    );

    pub fn webui_set_file_handler_window(
        window: usize,
        handler: Option<
            unsafe extern "C" fn(usize, *const c_char, *mut c_int) -> *const c_void,
        >,
    );

    pub fn webui_interface_set_response_file_handler(
        window: usize,
        response: *const c_void,
        length: c_int,
    );

    // -- State ----------------------------------------------------------

    pub fn webui_is_shown(window: usize) -> bool;
    pub fn webui_set_timeout(second: usize);
    pub fn webui_set_icon(window: usize, icon: *const c_char, icon_type: *const c_char);
    pub fn webui_set_profile(window: usize, name: *const c_char, path: *const c_char);
    pub fn webui_set_proxy(window: usize, proxy_server: *const c_char);
    pub fn webui_get_url(window: usize) -> *const c_char;
    pub fn webui_open_url(url: *const c_char);
    pub fn webui_set_public(window: usize, status: bool);
    pub fn webui_navigate(window: usize, url: *const c_char);
    pub fn webui_navigate_client(e: *mut RawEvent, url: *const c_char);
    pub fn webui_get_mime_type(file: *const c_char) -> *const c_char;

    // -- Network --------------------------------------------------------

    pub fn webui_get_port(window: usize) -> usize;
    pub fn webui_set_port(window: usize, port: usize) -> bool;
    pub fn webui_get_free_port() -> usize;

    // -- Process info ---------------------------------------------------

    pub fn webui_get_parent_process_id(window: usize) -> usize;
    pub fn webui_get_child_process_id(window: usize) -> usize;
    pub fn webui_win32_get_hwnd(window: usize) -> *mut c_void;
    pub fn webui_get_hwnd(window: usize) -> *mut c_void;

    // -- Memory / encoding ----------------------------------------------

    pub fn webui_encode(str: *const c_char) -> *mut c_char;
    pub fn webui_decode(str: *const c_char) -> *mut c_char;
    pub fn webui_free(ptr: *mut c_void);
    pub fn webui_malloc(size: usize) -> *mut c_void;
    pub fn webui_memcpy(dest: *mut c_void, src: *mut c_void, count: usize);

    // -- Raw binary data ------------------------------------------------

    pub fn webui_send_raw(
        window: usize,
        function: *const c_char,
        raw: *const c_void,
        size: usize,
    );

    pub fn webui_send_raw_client(
        e: *mut RawEvent,
        function: *const c_char,
        raw: *const c_void,
        size: usize,
    );

    // -- Logging / global config ----------------------------------------

    pub fn webui_set_logger(
        func: Option<unsafe extern "C" fn(usize, *const c_char, *mut c_void)>,
        user_data: *mut c_void,
    );

    pub fn webui_set_config(option: RawConfig, status: bool);
    pub fn webui_set_event_blocking(window: usize, status: bool);
    pub fn webui_set_runtime(window: usize, runtime: usize);

    // -- TLS ------------------------------------------------------------

    pub fn webui_set_tls_certificate(
        certificate_pem: *const c_char,
        private_key_pem: *const c_char,
    ) -> bool;

    // -- JavaScript -----------------------------------------------------

    pub fn webui_run(window: usize, script: *const c_char);
    pub fn webui_run_client(e: *mut RawEvent, script: *const c_char);

    pub fn webui_script(
        window: usize,
        script: *const c_char,
        timeout: usize,
        buffer: *mut c_char,
        buffer_length: usize,
    ) -> bool;

    pub fn webui_script_client(
        e: *mut RawEvent,
        script: *const c_char,
        timeout: usize,
        buffer: *mut c_char,
        buffer_length: usize,
    ) -> bool;

    // -- Event argument getters -----------------------------------------

    pub fn webui_get_count(e: *mut RawEvent) -> usize;
    pub fn webui_get_int_at(e: *mut RawEvent, index: usize) -> c_longlong;
    pub fn webui_get_int(e: *mut RawEvent) -> c_longlong;
    pub fn webui_get_float_at(e: *mut RawEvent, index: usize) -> f64;
    pub fn webui_get_float(e: *mut RawEvent) -> f64;
    pub fn webui_get_string_at(e: *mut RawEvent, index: usize) -> *const c_char;
    pub fn webui_get_string(e: *mut RawEvent) -> *const c_char;
    pub fn webui_get_bool_at(e: *mut RawEvent, index: usize) -> bool;
    pub fn webui_get_bool(e: *mut RawEvent) -> bool;
    pub fn webui_get_size_at(e: *mut RawEvent, index: usize) -> usize;
    pub fn webui_get_size(e: *mut RawEvent) -> usize;

    // -- Return values to JavaScript ------------------------------------

    pub fn webui_return_int(e: *mut RawEvent, n: c_longlong);
    pub fn webui_return_float(e: *mut RawEvent, f: f64);
    pub fn webui_return_string(e: *mut RawEvent, s: *const c_char);
    pub fn webui_return_bool(e: *mut RawEvent, b: bool);

    // -- Error handling -------------------------------------------------

    pub fn webui_get_last_error_number() -> usize;
    pub fn webui_get_last_error_message() -> *const c_char;

    // -- Cleanup --------------------------------------------------------

    pub fn webui_clean();
    pub fn webui_delete_all_profiles();
    pub fn webui_delete_profile(window: usize);

    // -- Interface API (used internally for closure callbacks) ----------

    pub fn webui_interface_bind(
        window: usize,
        element: *const c_char,
        func: Option<unsafe extern "C" fn(usize, usize, *mut c_char, usize, usize)>,
    ) -> usize;

    pub fn webui_interface_set_response(
        window: usize,
        event_number: usize,
        response: *const c_char,
    );

    pub fn webui_interface_is_app_running() -> bool;
    pub fn webui_interface_get_window_id(window: usize) -> usize;

    pub fn webui_interface_get_string_at(
        window: usize,
        event_number: usize,
        index: usize,
    ) -> *const c_char;

    pub fn webui_interface_get_int_at(
        window: usize,
        event_number: usize,
        index: usize,
    ) -> c_longlong;

    pub fn webui_interface_get_float_at(
        window: usize,
        event_number: usize,
        index: usize,
    ) -> f64;

    pub fn webui_interface_get_bool_at(
        window: usize,
        event_number: usize,
        index: usize,
    ) -> bool;

    pub fn webui_interface_get_size_at(
        window: usize,
        event_number: usize,
        index: usize,
    ) -> usize;

    pub fn webui_interface_show_client(
        window: usize,
        event_number: usize,
        content: *const c_char,
    ) -> bool;

    pub fn webui_interface_close_client(window: usize, event_number: usize);

    pub fn webui_interface_send_raw_client(
        window: usize,
        event_number: usize,
        function: *const c_char,
        raw: *const c_void,
        size: usize,
    );

    pub fn webui_interface_navigate_client(
        window: usize,
        event_number: usize,
        url: *const c_char,
    );

    pub fn webui_interface_run_client(
        window: usize,
        event_number: usize,
        script: *const c_char,
    );

    pub fn webui_interface_script_client(
        window: usize,
        event_number: usize,
        script: *const c_char,
        timeout: usize,
        buffer: *mut c_char,
        buffer_length: usize,
    ) -> bool;
}
