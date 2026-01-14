use include_dir::{Dir, include_dir};
use std::thread;
use tiny_http::{Header, Response, Server};

static DIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/leptos-site/dist");

pub fn start_server() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("Failed to start HTTP server");
    let port = server.server_addr().to_ip().unwrap().port();

    thread::spawn(move || {
        for request in server.incoming_requests() {
            let path = request.url().trim_start_matches('/');
            let path = if path.is_empty() { "index.html" } else { path };

            let response = if let Some(file) = DIST_DIR.get_file(path) {
                let content_type = get_content_type(path);
                let header =
                    Header::from_bytes("Content-Type", content_type).expect("Invalid header");
                Response::from_data(file.contents()).with_header(header)
            } else if let Some(file) = DIST_DIR.get_file("index.html") {
                let header =
                    Header::from_bytes("Content-Type", "text/html").expect("Invalid header");
                Response::from_data(file.contents()).with_header(header)
            } else {
                Response::from_string("Not Found").with_status_code(404)
            };

            let _ = request.respond(response);
        }
    });

    port
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "application/octet-stream"
    }
}
