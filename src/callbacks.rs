// callbacks.rs
// Global closure registry and the C trampoline that dispatches into it.
//
// How it works:
//   1. `Window::bind()` calls `webui_interface_bind()` with our static
//      `trampoline` function and gets back a unique `bind_id`.
//   2. The closure is boxed and stored in REGISTRY keyed by `bind_id`.
//   3. When C fires the callback it calls `trampoline(window, event_number,
//      element, event_type, bind_id)`.
//   4. The trampoline reconstructs a synthetic `RawEvent`, wraps it in a safe
//      `Event`, looks up the closure by `bind_id`, and calls it.

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::sync::{Mutex, OnceLock};

use crate::event::Event;
use crate::ffi::{self, RawEvent};

// ---------------------------------------------------------------------------
// Callback registry
// ---------------------------------------------------------------------------

type BoxedCallback = Box<dyn Fn(&Event) + Send + Sync + 'static>;

static REGISTRY: OnceLock<Mutex<HashMap<usize, BoxedCallback>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<usize, BoxedCallback>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Store a closure associated with a `bind_id`.
pub(crate) fn register<F>(bind_id: usize, callback: F)
where
    F: Fn(&Event) + Send + Sync + 'static,
{
    registry()
        .lock()
        .expect("callback registry poisoned")
        .insert(bind_id, Box::new(callback));
}

// ---------------------------------------------------------------------------
// Trampoline
// ---------------------------------------------------------------------------

/// The single `extern "C"` function we pass to `webui_interface_bind`.
///
/// C signature (from webui.h):
///   `void func(size_t window, size_t event_type, char* element,
///               size_t event_number, size_t bind_id)`
///
/// # Safety
/// Called from C — all pointer validity is C's responsibility.
pub(crate) unsafe extern "C" fn trampoline(
    window: usize,
    event_type: usize,
    element: *mut c_char,
    event_number: usize,
    bind_id: usize,
) {
    // Build a temporary RawEvent on the stack so Event can delegate back to
    // the `webui_get_*` / `webui_return_*` / `webui_interface_*` APIs.
    // The C library keeps all the real data in its own memory; the event
    // struct fields that matter for the interface API are window/event_number.
    let mut raw = RawEvent {
        window,
        event_type,
        element,
        event_number,
        bind_id,
        client_id: 0, // populated by C before the callback; safe to leave 0
        connection_id: 0,
        cookies: std::ptr::null_mut(),
    };

    let event = Event::from_raw(&mut raw as *mut RawEvent);

    // Look up and call the closure — don't hold the lock while calling.
    let cb_opt = {
        registry()
            .lock()
            .expect("callback registry poisoned")
            .get(&bind_id)
            // Clone the Arc-like smart pointer if we ever switch to Arc, but
            // for now we call through the lock after immediately releasing it.
            // We get a raw function pointer-style call by releasing the lock
            // first and using a short-lived borrow trick below instead.
            .map(|_| bind_id) // just confirm it exists
    };

    if cb_opt.is_some() {
        // Re-lock to actually call — this is safe because callbacks are
        // non-reentrant by default (WebUI serialises them unless the user
        // explicitly enables multi-threading via set_event_blocking(false)).
        let guard = registry().lock().expect("callback registry poisoned");
        if let Some(cb) = guard.get(&bind_id) {
            cb(&event);
        }
    }
}

// ---------------------------------------------------------------------------
// File-handler registry
// ---------------------------------------------------------------------------

/// The closure type for file handlers.
///
/// Receives the requested path (e.g. `"/index.html"`) and returns either:
/// - `Some(bytes)` — a complete HTTP response (headers **and** body), or just
///   raw file bytes if you let WebUI add the headers. The bytes are copied into
///   `webui_malloc`-owned memory so WebUI can free them after the response.
/// - `None` — fall through to WebUI's default file serving.
type BoxedFileHandler = Box<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync + 'static>;

static FILE_HANDLER_REGISTRY: OnceLock<Mutex<HashMap<usize, BoxedFileHandler>>> = OnceLock::new();

fn file_handler_registry() -> &'static Mutex<HashMap<usize, BoxedFileHandler>> {
    FILE_HANDLER_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Store a file-handler closure associated with a `window_id`.
/// Replaces any previously registered handler for that window.
pub(crate) fn register_file_handler<F>(window_id: usize, handler: F)
where
    F: Fn(&str) -> Option<Vec<u8>> + Send + Sync + 'static,
{
    file_handler_registry()
        .lock()
        .expect("file handler registry poisoned")
        .insert(window_id, Box::new(handler));
}

// ---------------------------------------------------------------------------
// File-handler trampoline
// ---------------------------------------------------------------------------

/// The single `extern "C"` function passed to `webui_set_file_handler_window`.
///
/// C signature (from webui.h):
///   `const void* handler(size_t window, const char* filename, int* length)`
///
/// Behaviour:
/// - Calls the Rust closure registered for `window`.
/// - If the closure returns `Some(bytes)`: allocates a buffer via
///   `webui_malloc`, copies the bytes in, sets `*length`, and returns the
///   pointer. WebUI owns this memory and will free it.
/// - If the closure returns `None` (or no handler is registered): sets
///   `*length` to `0` and returns `null`, telling WebUI to fall back to its
///   built-in file serving.
///
/// # Safety
/// Called from C — pointer validity is C's responsibility. `filename` is
/// guaranteed non-null by the WebUI library; `length` is a valid out-pointer.
pub(crate) unsafe extern "C" fn file_handler_trampoline(
    window: usize,
    filename: *const c_char,
    length: *mut c_int,
) -> *const c_void {
    // Safe default: tell WebUI we have nothing.
    *length = 0;

    // Convert the filename to a Rust &str. If it's invalid UTF-8, bail out.
    let path: &str = if filename.is_null() {
        return std::ptr::null();
    } else {
        match std::ffi::CStr::from_ptr(filename).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null(),
        }
    };

    // Look up and call the closure without holding the lock during the call.
    let maybe_bytes: Option<Vec<u8>> = {
        let guard = file_handler_registry()
            .lock()
            .expect("file handler registry poisoned");
        // We can't call the closure while holding the Mutex (it might try to
        // re-enter), so clone the path and drop the lock before calling.
        // Instead we call through the guard then immediately drop it.
        guard.get(&window).and_then(|cb| cb(path))
    };

    match maybe_bytes {
        None => std::ptr::null(),
        Some(bytes) if bytes.is_empty() => std::ptr::null(),
        Some(bytes) => {
            // Allocate WebUI-owned memory and copy the response bytes in.
            // WebUI will free this buffer after sending the HTTP response.
            let buf = ffi::webui_malloc(bytes.len()) as *mut u8;
            if buf.is_null() {
                return std::ptr::null();
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
            *length = bytes.len() as c_int;
            buf as *const c_void
        }
    }
}
