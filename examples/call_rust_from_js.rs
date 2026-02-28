// examples/call_rust_from_js.rs
use webui;

fn my_function_string(e: &webui::Event) {
    
	// JavaScript:
	// my_function_string('Hello', 'World`);

    let str_1: String = e.get_string();
    let str_2: String = e.get_string_at(1);

    println!("my_function_string 1: {}", str_1); // Hello
    println!("my_function_string 2: {}", str_2); // World
}

fn my_function_integer(e: &webui::Event) {
    
    // Javascript:
    // my_function_integer(123, 456, 789, 12345.6789);

    let count: usize = e.get_count();
    println!("my_function_integer: There is {} arguments in this event", count); // 4

    let number_1: i64 = e.get_int(); // Or e.get_int_at(0);
    let number_2: i64 = e.get_int_at(1);
    let number_3: i64 = e.get_int_at(2);

    println!("my_function_integer 1: {}", number_1); // 123
    println!("my_function_integer 2: {}", number_2); // 456
    println!("my_function_integer 3: {}", number_3); // 789

    let float_1: f64 = e.get_float_at(3);

    println!("my_function_integer 4: {}", float_1); // 12345.6789
}

fn my_function_boolean(e: &webui::Event) {

	// JavaScript:
	// my_function_boolean(true, false);

	let status_1: bool = e.get_bool(); // Or e.get_bool_at(0);
	let status_2: bool = e.get_bool_at(1);

	println!("my_function_boolean 1: {}", if status_1 { "True" } else { "False" }); // True
	println!("my_function_boolean 2: {}", if status_2 { "True" } else { "False" }); // False
}

fn my_function_raw_binary(e: &webui::Event) {

	// JavaScript:
	// my_function_raw_binary(new Uint8Array([0x41]), new Uint8Array([0x42, 0x43]));

	let raw_1: String = e.get_string(); // Or e.get_string_at(0);
	let raw_2: String = e.get_string_at(1);

	let len_1: usize = e.get_size(); // Or e.get_size_at(0);
	let len_2: usize = e.get_size_at(1);

    let raw_1 = raw_1.as_bytes();
    let raw_2 = raw_2.as_bytes();

	// Print raw_1
	print!("my_function_raw_binary 1 ({} bytes): ", len_1);
	for byte in raw_1 {
		print!("0x{:02x} ", byte);
    }
	println!();

	// Check raw_2 (Big)
	// [0xA1, 0x00..., 0xA2]
	let valid = raw_2.first() == Some(&0xA1) && raw_2.last() == Some(&0xA2);

	// Print raw_2
	println!(
        "my_function_raw_binary 2 big ({} bytes): valid data? {}",
        len_2,
        if valid { "Yes" } else { "No" }
    );
}

fn my_function_with_response(e: &webui::Event) {

	// JavaScript:
	// my_function_with_response(number, 2).then(...)

	let number: i64 = e.get_int(); // Or e.get_int_at(0);
	let times: i64 = e.get_int_at(1);

	let res: i64 = number * times;
	println!("my_function_with_response: {} * {} = {}", number, times, res);

	// Send back the response to JavaScript
	e.return_int(res);
}

fn main() {

	// HTML
	let my_html = r#"
	    <!DOCTYPE html>
	    <html>
	      <head>
	        <meta charset="UTF-8">
	        <script src="webui.js"></script>
	        <title>Call Rust from JavaScript Example</title>
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
	            input:focus { outline: none; border-color: #3498db; }
	        </style>
	      </head>
	      <body>
	        <h1>WebUI - Call Rust from JavaScript</h1>
	        <p>Call Rust functions with arguments (<em>See the logs in your terminal</em>)</p>
	        <button onclick="my_function_string('Hello', 'World');">Call my_function_string()</button>
	        <br>
	        <button onclick="my_function_integer(123, 456, 789, 12345.6789);">Call my_function_integer()</button>
	        <br>
	        <button onclick="my_function_boolean(true, false);">Call my_function_boolean()</button>
	        <br>
	        <button onclick="my_function_raw_binary(new Uint8Array([0x41,0x42,0x43]), big_arr);"> 
	         Call my_function_raw_binary()</button>
	        <br>
	        <p>Call a Rust function that returns a response</p>
	        <button onclick="MyJS();">Call my_function_with_response()</button>
	        <div>Double: <input type="text" id="MyInputID" value="2"></div>
	        <script>
	          const arr_size = 512 * 1000;
	          const big_arr = new Uint8Array(arr_size);
	          big_arr[0] = 0xA1;
	          big_arr[arr_size - 1] = 0xA2;
	          function MyJS() {
	            const MyInput = document.getElementById('MyInputID');
	            const number = MyInput.value;
	            my_function_with_response(number, 2).then((response) => {
	                MyInput.value = response;
	            });
	          }
	        </script>
	      </body>
	    </html>
        "#;

	// Create a window
	let my_window = webui::Window::new();

	// Bind HTML elements with C functions
	my_window.bind("my_function_string", my_function_string);
	my_window.bind("my_function_integer", my_function_integer);
	my_window.bind("my_function_boolean", my_function_boolean);
	my_window.bind("my_function_with_response", my_function_with_response);
	my_window.bind("my_function_raw_binary", my_function_raw_binary);

	// Show the window
	my_window.show(my_html); // my_window.show_browser(my_html, webui::Browser::Chrome);

	// Wait until all windows get closed
	webui::wait();

	// Free all memory resources (Optional)
	webui::clean();
}