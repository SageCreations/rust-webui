// examples/minimal/main.rs
use webui;

fn main() {
    let my_window = webui::Window::new();
    my_window.show("<html><head><script src=\"webui.js\"></script></head> Hello World ! </html>");
    webui::wait();
}
