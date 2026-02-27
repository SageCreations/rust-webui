// window.rs
// The main `Window` type — a safe wrapper around a WebUI window handle.

use std::ffi::{c_void, CString, c_char};
use std::time::Duration;

use crate::ffi;
use crate::types::{Browser, Runtime, ScriptError};
use crate::{callbacks, Event};

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

/// A WebUI window handle.
///
/// The underlying window is **not** automatically destroyed when this struct
/// is dropped. Call [`Window::destroy`] explicitly when you are done, or call
/// [`crate::exit`] to close everything.
///
/// ```no_run
/// use webui_rs::{Window, Browser};
///
/// let win = Window::new();
/// win.show("<html><body><h1>Hello from Rust!</h1></body></html>");
/// webui_rs::wait();
/// webui_rs::clean();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    pub(crate) id: usize,
}

impl Window {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a new WebUI window and return its handle.
    pub fn new() -> Self {
        Window {
            id: unsafe { ffi::webui_new_window() },
        }
    }

    /// Create a window using a specific numeric ID.
    /// `id` must be > 0 and < 65535 (`WEBUI_MAX_IDS`).
    pub fn new_with_id(id: usize) -> Self {
        Window {
            id: unsafe { ffi::webui_new_window_id(id) },
        }
    }

    /// Get the next available free window ID without creating a window.
    /// Useful when you need to know the ID before calling [`Window::new_with_id`].
    pub fn get_free_id() -> usize {
        unsafe { ffi::webui_get_new_window_id() }
    }

    // -----------------------------------------------------------------------
    // Binding / callbacks
    // -----------------------------------------------------------------------

    /// Bind an HTML element or JavaScript object name to a Rust closure.
    ///
    /// Pass an empty string `""` to catch **all** events from this window.
    ///
    /// The closure receives a reference to an [`Event`] which provides access
    /// to arguments and lets you return values to JavaScript.
    ///
    /// ```no_run
    /// win.bind("myButton", |e: &webui_rs::Event| {
    ///     println!("Button clicked, arg = {}", e.get_string());
    ///     e.return_int(42);
    /// });
    /// ```
    pub fn bind<F>(&self, element: &str, callback: F) -> usize
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let cstr = CString::new(element).unwrap_or_default();
        let bind_id =
            unsafe { ffi::webui_interface_bind(self.id, cstr.as_ptr(), Some(callbacks::trampoline)) };
        callbacks::register(bind_id, callback);
        bind_id
    }

    /// Attach arbitrary user data to a binding created with [`Window::bind`].
    /// Retrieve it inside the callback via [`Event`]… (advanced use via raw FFI).
    ///
    /// Note: for most use-cases you can just capture data in the closure instead.
    ///
    /// # Safety
    /// `context` must remain valid for as long as callbacks bound to `element`
    /// can be invoked. WebUI does not manage the lifetime of this pointer.
    pub unsafe fn set_context(&self, element: &str, context: *mut c_void) {
        let cstr = CString::new(element).unwrap_or_default();
        ffi::webui_set_context(self.id, cstr.as_ptr(), context)
    }

    /// Return the recommended browser ID for this window.
    pub fn get_best_browser(&self) -> Browser {
        let id = unsafe { ffi::webui_get_best_browser(self.id) };
        // Try to match known variants; fall back to Any.
        match id {
            0 => Browser::None,
            2 => Browser::Chrome,
            3 => Browser::Firefox,
            4 => Browser::Edge,
            5 => Browser::Safari,
            6 => Browser::Chromium,
            7 => Browser::Opera,
            8 => Browser::Brave,
            9 => Browser::Vivaldi,
            10 => Browser::Epic,
            11 => Browser::Yandex,
            12 => Browser::ChromiumBased,
            13 => Browser::Webview,
            _ => Browser::Any,
        }
    }

    // -----------------------------------------------------------------------
    // Show
    // -----------------------------------------------------------------------

    /// Show the window with embedded HTML, a file path, or a URL.
    /// Refreshes the window if it is already open.
    pub fn show(&self, content: &str) -> bool {
        let cstr = CString::new(content).unwrap_or_default();
        unsafe { ffi::webui_show(self.id, cstr.as_ptr()) }
    }

    /// Same as [`show`], but open in a specific browser.
    pub fn show_browser(&self, content: &str, browser: Browser) -> bool {
        let cstr = CString::new(content).unwrap_or_default();
        unsafe { ffi::webui_show_browser(self.id, cstr.as_ptr(), browser.into()) }
    }

    /// Start the web server only (no visible window). Returns the server URL.
    pub fn start_server(&self, content: &str) -> String {
        let cstr = CString::new(content).unwrap_or_default();
        unsafe {
            let ptr = ffi::webui_start_server(self.id, cstr.as_ptr());
            if ptr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Show the window using the embedded WebView instead of a browser.
    /// Requires `WebView2Loader.dll` on Windows.
    pub fn show_wv(&self, content: &str) -> bool {
        let cstr = CString::new(content).unwrap_or_default();
        unsafe { ffi::webui_show_wv(self.id, cstr.as_ptr()) }
    }

    // -----------------------------------------------------------------------
    // Close / destroy
    // -----------------------------------------------------------------------

    /// Close the window. The window object remains valid.
    pub fn close(&self) {
        unsafe { ffi::webui_close(self.id) }
    }

    /// Close the window and free all associated memory.
    /// The `Window` handle becomes invalid after this call.
    pub fn destroy(&self) {
        unsafe { ffi::webui_destroy(self.id) }
    }

    /// Minimize a WebView window.
    pub fn minimize(&self) {
        unsafe { ffi::webui_minimize(self.id) }
    }

    /// Maximize a WebView window.
    pub fn maximize(&self) {
        unsafe { ffi::webui_maximize(self.id) }
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    /// Returns `true` if the window is still open / connected.
    pub fn is_shown(&self) -> bool {
        unsafe { ffi::webui_is_shown(self.id) }
    }

    // -----------------------------------------------------------------------
    // Appearance settings
    // -----------------------------------------------------------------------

    /// Enable or disable kiosk (full-screen) mode.
    pub fn set_kiosk(&self, enabled: bool) {
        unsafe { ffi::webui_set_kiosk(self.id, enabled) }
    }

    /// Pass additional CLI parameters to the web browser process.
    pub fn set_custom_parameters(&self, params: &str) {
        let cstr = CString::new(params).unwrap_or_default();
        unsafe { ffi::webui_set_custom_parameters(self.id, cstr.as_ptr() as *mut c_char) }
    }

    /// Enable or disable the high-contrast theme hint.
    pub fn set_high_contrast(&self, enabled: bool) {
        unsafe { ffi::webui_set_high_contrast(self.id, enabled) }
    }

    /// Set whether the WebView window is resizable (WebView only).
    pub fn set_resizable(&self, enabled: bool) {
        unsafe { ffi::webui_set_resizable(self.id, enabled) }
    }

    /// Hide the window (call before [`show`]).
    pub fn set_hide(&self, hidden: bool) {
        unsafe { ffi::webui_set_hide(self.id, hidden) }
    }

    /// Set the window size in pixels.
    pub fn set_size(&self, width: u32, height: u32) {
        unsafe { ffi::webui_set_size(self.id, width, height) }
    }

    /// Set the minimum window size in pixels (WebView only).
    pub fn set_minimum_size(&self, width: u32, height: u32) {
        unsafe { ffi::webui_set_minimum_size(self.id, width, height) }
    }

    /// Set the window position on screen.
    pub fn set_position(&self, x: u32, y: u32) {
        unsafe { ffi::webui_set_position(self.id, x, y) }
    }

    /// Center the window on screen (works best with WebView; call before `show`).
    pub fn set_center(&self) {
        unsafe { ffi::webui_set_center(self.id) }
    }

    /// Make the WebView window frameless (no title bar / borders).
    pub fn set_frameless(&self, enabled: bool) {
        unsafe { ffi::webui_set_frameless(self.id, enabled) }
    }

    /// Make the WebView window transparent.
    pub fn set_transparent(&self, enabled: bool) {
        unsafe { ffi::webui_set_transparent(self.id, enabled) }
    }

    /// Set the window favicon. `icon` is SVG (or similar) content as a string.
    /// `icon_type` is the MIME type, e.g. `"image/svg+xml"`.
    pub fn set_icon(&self, icon: &str, icon_type: &str) {
        let icon_c = CString::new(icon).unwrap_or_default();
        let type_c = CString::new(icon_type).unwrap_or_default();
        unsafe { ffi::webui_set_icon(self.id, icon_c.as_ptr(), type_c.as_ptr()) }
    }

    // -----------------------------------------------------------------------
    // Browser profile / proxy / network
    // -----------------------------------------------------------------------

    /// Set the browser profile name and path.
    /// Pass empty strings for the default profile.
    pub fn set_profile(&self, name: &str, path: &str) {
        let name_c = CString::new(name).unwrap_or_default();
        let path_c = CString::new(path).unwrap_or_default();
        unsafe { ffi::webui_set_profile(self.id, name_c.as_ptr(), path_c.as_ptr()) }
    }

    /// Set a proxy server URL for the browser (call before `show`).
    pub fn set_proxy(&self, proxy: &str) {
        let cstr = CString::new(proxy).unwrap_or_default();
        unsafe { ffi::webui_set_proxy(self.id, cstr.as_ptr()) }
    }

    /// Get the current URL of this running window.
    pub fn get_url(&self) -> String {
        unsafe {
            let ptr = ffi::webui_get_url(self.id);
            if ptr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Make the window's address accessible from a public network.
    pub fn set_public(&self, enabled: bool) {
        unsafe { ffi::webui_set_public(self.id, enabled) }
    }

    /// Navigate all connected clients to a URL.
    pub fn navigate(&self, url: &str) {
        let cstr = CString::new(url).unwrap_or_default();
        unsafe { ffi::webui_navigate(self.id, cstr.as_ptr()) }
    }

    /// Get the network port this window's web server is listening on.
    pub fn get_port(&self) -> usize {
        unsafe { ffi::webui_get_port(self.id) }
    }

    /// Set a custom network port for this window's web server.
    /// Returns `true` if the port is free and usable.
    pub fn set_port(&self, port: usize) -> bool {
        unsafe { ffi::webui_set_port(self.id, port) }
    }

    // -----------------------------------------------------------------------
    // File server
    // -----------------------------------------------------------------------

    /// Set the root folder for this window's web server.
    pub fn set_root_folder(&self, path: &str) -> bool {
        let cstr = CString::new(path).unwrap_or_default();
        unsafe { ffi::webui_set_root_folder(self.id, cstr.as_ptr()) }
    }

    // -----------------------------------------------------------------------
    // JavaScript
    // -----------------------------------------------------------------------

    /// Run JavaScript on all connected clients without waiting for a result.
    pub fn run(&self, script: &str) {
        let cstr = CString::new(script).unwrap_or_default();
        unsafe { ffi::webui_run(self.id, cstr.as_ptr()) }
    }

    /// Run JavaScript on all connected clients and return the result.
    ///
    /// `timeout` of `Duration::ZERO` means wait indefinitely.
    ///
    /// Returns `Ok(response_string)` on success, `Err(())` on JS error.
    pub fn script(&self, script: &str, timeout: Duration) -> Result<String, ScriptError> {
        let cstr = CString::new(script).unwrap_or_default();
        let buf_size: usize = 4096;
        let mut buf: Vec<u8> = vec![0u8; buf_size];
        let ok = unsafe {
            ffi::webui_script(
                self.id,
                cstr.as_ptr(),
                timeout.as_secs() as usize,
                buf.as_mut_ptr() as *mut i8,
                buf_size,
            )
        };
        if ok {
            let s = std::ffi::CStr::from_bytes_until_nul(&buf)
                .map(|cs| cs.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(s)
        } else {
            Err(ScriptError)
        }
    }

    /// Set the JS/TS runtime for this window's `.js` / `.ts` files.
    pub fn set_runtime(&self, runtime: Runtime) {
        unsafe { ffi::webui_set_runtime(self.id, runtime.into()) }
    }

    // -----------------------------------------------------------------------
    // Raw binary data
    // -----------------------------------------------------------------------

    /// Send raw binary data to a JavaScript function on all connected clients.
    ///
    /// On the JavaScript side: `function myFunc(data) { /* data is a Uint8Array */ }`
    pub fn send_raw(&self, function: &str, data: &[u8]) {
        let cstr = CString::new(function).unwrap_or_default();
        unsafe {
            ffi::webui_send_raw(
                self.id,
                cstr.as_ptr(),
                data.as_ptr() as *const c_void,
                data.len(),
            )
        }
    }

    // -----------------------------------------------------------------------
    // Event threading
    // -----------------------------------------------------------------------

    /// Control whether events from this window are processed one at a time
    /// (`true`) or each in its own thread (`false`).
    pub fn set_event_blocking(&self, blocking: bool) {
        unsafe { ffi::webui_set_event_blocking(self.id, blocking) }
    }

    // -----------------------------------------------------------------------
    // Process / OS info
    // -----------------------------------------------------------------------

    /// Get the backend (parent) process ID.
    pub fn get_parent_process_id(&self) -> usize {
        unsafe { ffi::webui_get_parent_process_id(self.id) }
    }

    /// Get the browser (child) process ID.
    pub fn get_child_process_id(&self) -> usize {
        unsafe { ffi::webui_get_child_process_id(self.id) }
    }

    /// Get the native window handle as a raw pointer.
    /// On Windows: `HWND`. On Linux (WebView only): `GtkWindow*`.
    pub fn get_hwnd(&self) -> *mut c_void {
        unsafe { ffi::webui_get_hwnd(self.id) }
    }

    /// Get the Win32 `HWND`. More reliable than [`get_hwnd`] for WebView.
    #[cfg(target_os = "windows")]
    pub fn win32_get_hwnd(&self) -> *mut c_void {
        unsafe { ffi::webui_win32_get_hwnd(self.id) }
    }

    // -----------------------------------------------------------------------
    // Profile cleanup
    // -----------------------------------------------------------------------

    /// Delete the local browser profile folder for this window.
    /// Call after [`crate::wait`] and before [`crate::clean`].
    pub fn delete_profile(&self) {
        unsafe { ffi::webui_delete_profile(self.id) }
    }

    // -----------------------------------------------------------------------
    // Close handler (WebView only)
    // -----------------------------------------------------------------------

    /// Set a callback to intercept the WebView close button.
    /// Return `false` from the callback to prevent closing, `true` to allow it.
    pub fn set_close_handler_wv(&self, handler: Option<unsafe extern "C" fn(usize) -> bool>) {
        unsafe { ffi::webui_set_close_handler_wv(self.id, handler) }
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
