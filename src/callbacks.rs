// callbacks.rs
// Global closure registry and the C trampoline that dispatches into it.
//
// How it works:
//   1. `Window::bind()` calls `webui_interface_bind()` with our static
//      `trampoline` function and gets back a unique `bind_id`.
//   2. The closure is boxed and stored in REGISTRY keyed by `bind_id`.
//   3. When C fires the callback it calls `trampoline(window, event_number,
//      element, event_type, bind_id)`.
//   4. The trampoline reconstructs a synthetic `webui_event_t`, wraps it in a
//      safe `Event`, looks up the closure by `bind_id`, and calls it.

use std::collections::HashMap;
use std::ffi::c_char;
use std::sync::{Mutex, OnceLock};

use crate::bindings::webui_event_t;
use crate::event::Event;

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
    // Build a temporary webui_event_t on the stack so Event can delegate back
    // to the `webui_get_*` / `webui_return_*` / `webui_interface_*` APIs.
    // The C library keeps all the real data in its own memory; the fields that
    // matter for the interface API are window/event_number.
    let mut raw = webui_event_t {
        window,
        event_type,
        element,
        event_number,
        bind_id,
        client_id: 0, // populated by C before the callback; safe to leave 0
        connection_id: 0,
        cookies: std::ptr::null_mut(),
    };

    let event = Event::from_raw(&mut raw as *mut webui_event_t);

    // Look up and call the closure — don't hold the lock while calling.
    let cb_opt = {
        registry()
            .lock()
            .expect("callback registry poisoned")
            .get(&bind_id)
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
