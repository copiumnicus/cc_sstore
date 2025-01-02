use cc_sstore::generate_key;
use cc_sstore::read_key;
use cc_sstore::run_menu;
use cc_sstore::toast;
use cc_sstore::ZeroizingVecReader;
use cc_sstore::BIND;
use tiny_http::Header;
use tiny_http::Method;
use tiny_http::StatusCode;
use tiny_http::{Response, Server};

fn is_valid_string_name(name: &str) -> bool {
    // Check that all characters in the name are valid (a-z, A-Z, _)
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn handle_request(path: &str) -> Option<String> {
    if let Some(name) = path.strip_prefix("/generate/") {
        if is_valid_string_name(name) {
            match generate_key(name) {
                Ok(_) => {
                    toast("Generate", format!("Success generating: {}", name));
                    return Some(format!("Generated: {}", name));
                }
                Err(e) => {
                    toast("Generate error", format!("{:?}", e));
                }
            }
        }
    } else if let Some(name) = path.strip_prefix("/read/") {
        if is_valid_string_name(name) {
            match read_key(name) {
                Ok(key) => {
                    toast("Read", format!("Success read: {}", name));
                    return Some(key);
                }
                Err(e) => {
                    toast("Read error", format!("{:?}", e));
                }
            }
        }
    }

    None
}

fn main() {
    let server = Server::https(
        BIND,
        tiny_http::SslConfig {
            certificate: include_bytes!("ssl-cert.pem").to_vec(),
            private_key: include_bytes!("ssl-key.pem").to_vec(),
        },
    )
    .unwrap();
    toast("Started server", format!("Bind on {}", BIND).as_str());

    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            assert!(request.secure());
            if !matches!(request.method(), Method::Get) {
                let _ = request.respond(Response::empty(StatusCode(400)));
                continue;
            }
            toast(
                "Request",
                format!(
                    "{} -> {} {}",
                    request
                        .remote_addr()
                        .map(|x| x.to_string())
                        .unwrap_or("undefined".into()),
                    request.method(),
                    request.url()
                )
                .as_str(),
            );
            match handle_request(request.url()) {
                Some(r) => {
                    let dl = r.len();
                    let res = Response::new(
                        StatusCode(200),
                        vec![Header::from_bytes(
                            &b"Content-Type"[..],
                            &b"text/plain; charset=UTF-8"[..],
                        )
                        .unwrap()],
                        ZeroizingVecReader::new(r.into_bytes()),
                        Some(dl),
                        None,
                    );
                    // mby can use this to prevent leaving sensitive info in memory
                    // request.into_writer()
                    let _ = request.respond(res);
                }
                None => {
                    let _ = request.respond(Response::empty(StatusCode(400)));
                }
            }
        }
    });
    // need on main thread
    unsafe { run_menu() };
}
