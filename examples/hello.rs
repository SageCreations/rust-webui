// examples/hello.rs
use webui;

fn main() {
    let my_window = webui::Window::new();

    // Bind a Rust closure to a JavaScript-callable function.
    // Any data you need can be captured in the closure.
    let greeting = String::from("Hello from Rust!");
    my_window.bind("sayHello", move |e: &webui::Event| {
        let name = e.get_string(); // first JS argument
        println!("JS called sayHello('{}'), greeting = {}", name, greeting);
        e.return_string(&format!("{}, {}!", greeting, name));
    });

    // Show the window — inline HTML, a file path, or a URL all work.
    my_window.show_browser(r#"
        <!DOCTYPE html>
        <html>
        <head><script src="webui.js"></script></head>
        <body>
            <h1>WebUI + Rust</h1>
            <button onclick="
                webui.call('sayHello', 'World').then(r => alert(r));
            ">Click me</button>
        </body>
        </html>
    "#, webui::Browser::Webview);

    // Block until the window is closed.
    webui::wait();

    // Free all WebUI resources.
    webui::clean();
}
