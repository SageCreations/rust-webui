// examples/hello.rs
use webui_rs;

fn main() {
    let my_window = webui_rs::Window::new();

    // Bind a Rust closure to a JavaScript-callable function.
    // Any data you need can be captured in the closure.
    let greeting = String::from("Hello from Rust!");
    my_window.bind("sayHello", move |e: &webui_rs::Event| {
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
    "#, webui_rs::Browser::Webview);

    // Block until the window is closed.
    webui_rs::wait();

    // Free all WebUI resources.
    webui_rs::clean();
}
