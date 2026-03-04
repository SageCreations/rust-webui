// examples/serve_a_folder/main.rs
use webui;

const MY_WINDOW: usize = 1;
const MY_SECOND_WINDOW: usize = 2;

fn exit_app(_e: &webui::Event) {
    // Close all opened windows
    webui::exit();
}

fn events(e: &webui::Event) {
    // This function gets called every time
    // there is an event

    match e.event_type() {
        webui::EventType::Disconnected => {
            println!("Disconnected. ");
        }
        webui::EventType::Connected => {
            println!("Connected. ");
        }
        webui::EventType::MouseClick => {
            println!("Click. ");
        }
        webui::EventType::Navigation => {
            let url = e.get_string();
            println!("Starting navigation to: {}", url);

            // Because we used `MY_WINDOW.bind("", events);`
            // WebUI will block all `href` link clicks and sent here instead.
            // We can then control the behaviour of links as needed.
            e.get_window().navigate(&url);
        }
        webui::EventType::Callback => {}
        webui::EventType::Unknown(_) => {}
    }
}

fn switch_to_second_page(e: &webui::Event) {
    // This function gets called every
    // time the user clicks on "SwitchToSecondPage"

    // Switch to `/second.html` in the same opened window.
    e.get_window().show("second.html");
}

fn show_second_window(_e: &webui::Event) {
    // This function gets called every
    // time the user clicks on "OpenNewWindow"

    // Show a new window, and navigate to `/second.html`
    // if it's already open, then switch in the same window
    webui::Window::from_id(MY_SECOND_WINDOW).show("second.html");
}

fn my_files_handler(path: &str) -> Option<Vec<u8>> {
    println!("File: {}", path);

    match path {
        "/test.txt" => {
            // Const static file example — build the full HTTP response in one shot.
            // The Content-Length must match the body byte-length exactly.
            let body = "<html>\
                            This is a static embedded file content example. \
                            <script src=\"webui.js\"></script>\
                        </html>";

            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/html\r\n\
                    Content-Length: {}\r\n\r\n\
                    {}",
                body.len(),
                body
            );

            Some(response.into_bytes())
        }

        "/dynamic.html" => {
            // Dynamic file example — the body is generated at call-time.
            // We use a thread-local counter to mirror the static int in the C version.
            use std::cell::Cell;
            thread_local! {
                static COUNT: Cell<u32> = Cell::new(0);
            }

            let count = COUNT.with(|c| {
                let next = c.get() + 1;
                c.set(next);
                next
            });

            let body = format!(
                "<html>\
                    This is a dynamic file content example. <br>\
                    Count: {} <a href=\"dynamic.html\">[Refresh]</a><br>\
                    <script src=\"webui.js\"></script>\
                    </html>",
                count
            );

            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/html\r\n\
                    Content-Length: {}\r\n\r\n\
                    {}",
                body.len(),
                body
            );

            Some(response.into_bytes())

            // No manual malloc/free needed — the Vec<u8> is returned to the
            // trampoline which copies it into webui_malloc memory and then drops
            // the Vec automatically at end of scope.
        }

        // Any other path: return None so WebUI falls back to local file serving.
        _ => None,
    }
}

fn main() {
    // Create new windows
    let my_window = webui::Window::new_with_id(MY_WINDOW);
    let my_second_window = webui::Window::new_with_id(MY_SECOND_WINDOW);

    // Bind HTML element IDs with a Rust functions
    my_window.bind("SwitchToSecondPage", switch_to_second_page);
    my_window.bind("OpenNewWindow", show_second_window);
    my_window.bind("Exit", exit_app);
    my_second_window.bind("Exit", exit_app);

    // Bind events
    my_window.bind("", events);

    // Set the `.ts` and `.js` runtime
    my_window.set_runtime(webui::Runtime::NodeJS);
    // my_window.set_runtime(webui::Runtime::Bun);
    //my_window.set_runtime(webui::Runtime::Deno);

    // Set a custom files handler
    my_window.set_file_handler(my_files_handler);

    // Set window size
    my_window.set_size(800, 800);

    // Set window position
    my_window.set_position(200, 200);

    // Show a new window
    //my_window.set_root_folder("./");
    my_window.show_browser("index.html", webui::Browser::Brave);
    //my_window.show("./index.html");

    // Wait until all windows get closed
    webui::wait();

    // Free all memory resources (Optional)
    webui::clean();
}
