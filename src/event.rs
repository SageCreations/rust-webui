// event.rs
// Safe wrapper around a webui_event_t pointer.
// An `Event` is only ever valid for the lifetime of a callback invocation —
// never store it or send it across threads.

use std::ffi::{CStr, CString};
use std::time::Duration;

use crate::ffi::{self, RawEvent};
use crate::types::{EventType, ScriptError};

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

/// A UI event passed to every bound callback.
///
/// All methods are safe — they validate pointers and handle string conversion.
///
/// ```no_run
/// use webui_rs::{Window, Event};
///
/// let win = Window::new();
/// win.bind("myButton", |e: &Event| {
///     println!("clicked! arg0 = {}", e.get_string());
///     e.return_string("pong");
/// });
/// ```
pub struct Event {
    // Raw pointer into C memory — valid for the duration of the C callback only.
    raw: *mut RawEvent,
}

// SAFETY: WebUI guarantees a callback is invoked from a single thread at a
// time when event_blocking is enabled. When it's disabled each callback runs
// on its own thread so Event must be Send for that case. It is never actually
// stored beyond a callback invocation.
unsafe impl Send for Event {}

impl Event {
    /// Wrap a raw C event pointer. Called exclusively from the trampoline in lib.rs.
    ///
    /// # Safety
    /// `raw` must be a valid, non-null pointer to a `webui_event_t` that lives
    /// for the entire duration this `Event` object exists.
    pub(crate) unsafe fn from_raw(raw: *mut RawEvent) -> Self {
        Event { raw }
    }

    // -----------------------------------------------------------------------
    // Metadata
    // -----------------------------------------------------------------------

    /// The window number this event belongs to.
    pub fn window_id(&self) -> usize {
        unsafe { (*self.raw).window }
    }

    /// The type of this event (click, navigation, callback, …).
    pub fn event_type(&self) -> EventType {
        unsafe { EventType::from((*self.raw).event_type) }
    }

    /// The HTML element ID / JavaScript object name that triggered this event.
    /// Returns an empty string if the element pointer is null.
    pub fn element(&self) -> String {
        unsafe {
            let ptr = (*self.raw).element;
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    /// Internal event number — used by the interface API.
    pub fn event_number(&self) -> usize {
        unsafe { (*self.raw).event_number }
    }

    /// The bind ID of the binding that triggered this event.
    pub fn bind_id(&self) -> usize {
        unsafe { (*self.raw).bind_id }
    }

    /// A unique ID identifying the connected client (browser tab / window).
    pub fn client_id(&self) -> usize {
        unsafe { (*self.raw).client_id }
    }

    /// The connection ID for this specific WebSocket connection.
    pub fn connection_id(&self) -> usize {
        unsafe { (*self.raw).connection_id }
    }

    /// The full cookies string from the client. Empty string if null.
    pub fn cookies(&self) -> String {
        unsafe {
            let ptr = (*self.raw).cookies;
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    // -----------------------------------------------------------------------
    // Argument getters
    // -----------------------------------------------------------------------

    /// Number of arguments passed from JavaScript.
    pub fn arg_count(&self) -> usize {
        unsafe { ffi::webui_get_count(self.raw) }
    }

    /// Get the argument at `index` as a `String`. Returns empty string on error.
    pub fn get_string_at(&self, index: usize) -> String {
        unsafe {
            let ptr = ffi::webui_get_string_at(self.raw, index);
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    /// Get the first argument as a `String`.
    pub fn get_string(&self) -> String {
        unsafe {
            let ptr = ffi::webui_get_string(self.raw);
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    /// Get the argument at `index` as an `i64`.
    pub fn get_int_at(&self, index: usize) -> i64 {
        unsafe { ffi::webui_get_int_at(self.raw, index) }
    }

    /// Get the first argument as an `i64`.
    pub fn get_int(&self) -> i64 {
        unsafe { ffi::webui_get_int(self.raw) }
    }

    /// Get the argument at `index` as an `f64`.
    pub fn get_float_at(&self, index: usize) -> f64 {
        unsafe { ffi::webui_get_float_at(self.raw, index) }
    }

    /// Get the first argument as an `f64`.
    pub fn get_float(&self) -> f64 {
        unsafe { ffi::webui_get_float(self.raw) }
    }

    /// Get the argument at `index` as a `bool`.
    pub fn get_bool_at(&self, index: usize) -> bool {
        unsafe { ffi::webui_get_bool_at(self.raw, index) }
    }

    /// Get the first argument as a `bool`.
    pub fn get_bool(&self) -> bool {
        unsafe { ffi::webui_get_bool(self.raw) }
    }

    /// Get the byte-size of the argument at `index`.
    pub fn get_size_at(&self, index: usize) -> usize {
        unsafe { ffi::webui_get_size_at(self.raw, index) }
    }

    /// Get the byte-size of the first argument.
    pub fn get_size(&self) -> usize {
        unsafe { ffi::webui_get_size(self.raw) }
    }

    // -----------------------------------------------------------------------
    // Return values to JavaScript
    // -----------------------------------------------------------------------

    /// Send an integer back to the JavaScript caller.
    pub fn return_int(&self, n: i64) {
        unsafe { ffi::webui_return_int(self.raw, n) }
    }

    /// Send a float back to the JavaScript caller.
    pub fn return_float(&self, f: f64) {
        unsafe { ffi::webui_return_float(self.raw, f) }
    }

    /// Send a string back to the JavaScript caller.
    pub fn return_string(&self, s: &str) {
        let cstr = CString::new(s).unwrap_or_default();
        unsafe { ffi::webui_return_string(self.raw, cstr.as_ptr()) }
    }

    /// Send a boolean back to the JavaScript caller.
    pub fn return_bool(&self, b: bool) {
        unsafe { ffi::webui_return_bool(self.raw, b) }
    }

    // -----------------------------------------------------------------------
    // Single-client actions (operating on just this client)
    // -----------------------------------------------------------------------

    /// Show HTML/URL to this specific client only.
    pub fn show_client(&self, content: &str) -> bool {
        let cstr = CString::new(content).unwrap_or_default();
        unsafe { ffi::webui_show_client(self.raw, cstr.as_ptr()) }
    }

    /// Close this specific client's connection.
    pub fn close_client(&self) {
        unsafe { ffi::webui_close_client(self.raw) }
    }

    /// Navigate this specific client to a URL.
    pub fn navigate_client(&self, url: &str) {
        let cstr = CString::new(url).unwrap_or_default();
        unsafe { ffi::webui_navigate_client(self.raw, cstr.as_ptr()) }
    }

    /// Run JavaScript on this specific client without waiting for a response.
    pub fn run_client(&self, script: &str) {
        let cstr = CString::new(script).unwrap_or_default();
        unsafe { ffi::webui_run_client(self.raw, cstr.as_ptr()) }
    }

    /// Run JavaScript on this specific client and return the result.
    ///
    /// Returns `Ok(response)` on success, `Err(())` on execution error.
    pub fn script_client(&self, script: &str, timeout: Duration) -> Result<String, ScriptError> {
        let cstr = CString::new(script).unwrap_or_default();
        let buf_size = 4096;
        let mut buf: Vec<u8> = vec![0u8; buf_size];
        let ok = unsafe {
            ffi::webui_script_client(
                self.raw,
                cstr.as_ptr(),
                timeout.as_secs() as usize,
                buf.as_mut_ptr() as *mut i8,
                buf_size,
            )
        };
        if ok {
            let s = CStr::from_bytes_until_nul(&buf)
                .map(|cs| cs.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(s)
        } else {
            Err(ScriptError)
        }
    }

    /// Send raw binary data to a JavaScript function on this specific client.
    pub fn send_raw_client(&self, function: &str, data: &[u8]) {
        let cstr = CString::new(function).unwrap_or_default();
        unsafe {
            ffi::webui_send_raw_client(
                self.raw,
                cstr.as_ptr(),
                data.as_ptr() as *const std::ffi::c_void,
                data.len(),
            )
        }
    }
}
