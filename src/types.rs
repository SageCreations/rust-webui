// types.rs
// Idiomatic Rust enums that map to the C integer constants.
// These are what users of the library will actually touch.

// Import the bindgen type alias and every config constant we need to map to.
use crate::bindings::{
    webui_config, webui_config_asynchronous_response, webui_config_folder_monitor,
    webui_config_multi_client, webui_config_show_wait_connection, webui_config_ui_event_blocking,
    webui_config_use_cookies,
};

// ---------------------------------------------------------------------------
// Browser
// ---------------------------------------------------------------------------

/// The web browser to use when showing a window.
/// Passed as `size_t` in C; represented as a `usize` at the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Browser {
    /// No web browser — suppress opening any window.
    None = 0,
    /// Let WebUI pick the best available browser.
    Any = 1,
    Chrome = 2,
    Firefox = 3,
    Edge = 4,
    Safari = 5,
    Chromium = 6,
    Opera = 7,
    Brave = 8,
    Vivaldi = 9,
    Epic = 10,
    Yandex = 11,
    /// Any Chromium-based browser.
    ChromiumBased = 12,
    /// Use a WebView instead of a real browser.
    Webview = 13,
}

impl From<Browser> for usize {
    fn from(b: Browser) -> usize {
        b as usize
    }
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

/// JavaScript / TypeScript runtime for `.js` / `.ts` files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Runtime {
    /// No runtime — WebUI won't execute JS/TS files automatically.
    None = 0,
    Deno = 1,
    NodeJS = 2,
    Bun = 3,
}

impl From<Runtime> for usize {
    fn from(r: Runtime) -> usize {
        r as usize
    }
}

// ---------------------------------------------------------------------------
// EventType
// ---------------------------------------------------------------------------

/// The type of a UI event delivered to a bound callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum EventType {
    Disconnected = 0,
    Connected = 1,
    MouseClick = 2,
    Navigation = 3,
    Callback = 4,
    /// Catch-all for any future event type the C library may add.
    Unknown(usize),
}

impl From<usize> for EventType {
    fn from(n: usize) -> EventType {
        match n {
            0 => EventType::Disconnected,
            1 => EventType::Connected,
            2 => EventType::MouseClick,
            3 => EventType::Navigation,
            4 => EventType::Callback,
            other => EventType::Unknown(other),
        }
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Global WebUI configuration options.
/// Used with [`crate::set_config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    /// Whether `show()` should block until the browser connects.
    /// Default: `true`.
    ShowWaitConnection,
    /// Process UI events one at a time in a single thread (`true`)
    /// or each in its own thread (`false`). Default: `false`.
    UiEventBlocking,
    /// Auto-refresh the window when a file in the root folder changes.
    /// Default: `false`.
    FolderMonitor,
    /// Allow multiple clients to connect to the same window.
    /// Default: `false`.
    MultiClient,
    /// Let WebUI add `webui_auth` cookies to identify and authenticate clients.
    /// Default: `true`.
    UseCookies,
    /// Wait for the backend to call `return_*()` before sending a JS response.
    /// Default: `false`.
    AsynchronousResponse,
}

/// Convert to the bindgen `webui_config` type (`c_uint` alias) using the
/// named constants that bindgen generated from the C enum.
impl From<Config> for webui_config {
    fn from(c: Config) -> webui_config {
        match c {
            Config::ShowWaitConnection => webui_config_show_wait_connection,
            Config::UiEventBlocking => webui_config_ui_event_blocking,
            Config::FolderMonitor => webui_config_folder_monitor,
            Config::MultiClient => webui_config_multi_client,
            Config::UseCookies => webui_config_use_cookies,
            Config::AsynchronousResponse => webui_config_asynchronous_response,
        }
    }
}

// ---------------------------------------------------------------------------
// ScriptError
// ---------------------------------------------------------------------------

/// Returned by [`crate::Window::script`] and [`crate::Event::script_client`]
/// when JavaScript execution fails on the C side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError;

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebUI JavaScript execution error")
    }
}

impl std::error::Error for ScriptError {}

// ---------------------------------------------------------------------------
// LoggerLevel
// ---------------------------------------------------------------------------

/// Verbosity level for the custom logger set via [`crate::set_logger`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum LoggerLevel {
    /// All logs with full details.
    Debug = 0,
    /// General informational logs only.
    Info = 1,
    /// Fatal errors only.
    Error = 2,
}

impl From<usize> for LoggerLevel {
    fn from(n: usize) -> LoggerLevel {
        match n {
            0 => LoggerLevel::Debug,
            1 => LoggerLevel::Info,
            _ => LoggerLevel::Error,
        }
    }
}
