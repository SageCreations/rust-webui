// examples/call_js_from_rust.rs
use webui;

fn my_function_exit(_e: &webui::Event) {
    // Close all opened windows
    webui::exit();
}

fn my_function_count(e: &webui::Event) {
    // This function gets called every time the user clicks on "my_function_count"

    let timeout = std::time::Duration::from_secs(0);

    // Run JavaScript
    let count = match e.get_window().script("return GetCount();", timeout) {
        Ok(response) => {
            // Parse the count from the response string
            response.trim().parse::<i32>().unwrap_or(0)
        }
        Err(_) => {
            if !e.get_window().is_shown() {
                println!("Window closed.");
            } else {
                println!("JavaScript Error");
            }
            return;
        }
    };

    // Increment
    let count = count + 1;

    // Run JavaScript to set the new count
    let js = format!("SetCount({});", count);
    e.get_window().run(&js);
}

fn main() {
    let my_html = r#"
        <!DOCTYPE html>
	    <html>
	      <head>
	        <meta charset="UTF-8">
	        <script src="webui.js"></script>
	        <title>Call JavaScript from Rust Example</title>
	        <style>
	           body {
	                font-family: 'Arial', sans-serif;
	                color: white;
	                background: linear-gradient(to right, #507d91, #1c596f, #022737);
	                text-align: center;
	                font-size: 18px;
	            }
	            button, input {
	                padding: 10px;
	                margin: 10px;
	                border-radius: 3px;
	                border: 1px solid #ccc;
	                box-shadow: 0 3px 5px rgba(0,0,0,0.1);
	                transition: 0.2s;
	            }
	            button {
	                background: #3498db;
	                color: #fff;
	                cursor: pointer;
	                font-size: 16px;
	            }
	            h1 { text-shadow: -7px 10px 7px rgb(67 57 57 / 76%); }
	            button:hover { background: #c9913d; }
	            button:disabled {
	                opacity: 0.6;
	                cursor: not-allowed;
	                box-shadow: none;
	                filter: grayscale(30%);
	            }
	            button:disabled:hover { background: #3498db; }
	            input:focus { outline: none; border-color: #3498db; }
	        </style>
	      </head>
	      <body>
	        <h1>WebUI - Call JavaScript from Rust</h1>
	        <br>
	        <h1 id="count">0</h1>
	        <br>
	        <button id="ManualBtn" OnClick="my_function_count();">Manual Count</button>
	        <br>
	        <button id="MyTest" OnClick="AutoTest();">Auto Count (Every 10ms)</button>
	        <br>
	        <button id="ExitBtn" OnClick="this.disabled=true; my_function_exit();">Exit</button>
	        <script>
	          let count = 0;
	          let auto_running = false;
	          function GetCount() {
	            return count;
	          }
	          function SetCount(number) {
	            document.getElementById('count').innerHTML = number;
	            count = number;
	          }
	          function AutoTest(number) {
	            if (auto_running) return;
	            auto_running = true;
	            document.getElementById('MyTest').disabled = true;
	            document.getElementById('ManualBtn').disabled = true;
	            setInterval(function(){ my_function_count(); }, 10);
	          }
	        </script>
	      </body>
	    </html>
    "#;

    // Set WebUI configuration to proceess UI events one at a time
    webui::set_config(webui::Config::UiEventBlocking, true);

    // Create a window
    let my_window = webui::Window::new();

    // Bind HTML elements with Rust functions
    my_window.bind("my_function_count", my_function_count);
    my_window.bind("my_function_exit", my_function_exit);

    // Show the window
    my_window.show(my_html);

    // Wait until all windows get closed
    webui::wait();

    // Free all memory resources (Optional)
    webui::clean();
}
