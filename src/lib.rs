//! # Simple JSON Server
//!
//! This library is intended to make it very easy to build a JSON-based server. Because its JSON it is easily called from multiple languages (think a TypeScript client).
//! The goal is to provide a lot of functionality out of the box but with excellent ergonomics so that simple servers are easily built without almost any boilerplate.
//!
//! The macro both creates your simple server (an actor) from an impl of your struct and also generates documentation for your actor so its 100% clear how to call it.
//!
//! The way `simple_json_server` works is:
//! 1. Create a `struct` and create an `impl` block for it.  The `impl` block should contain all the methods you want to expose on your server.  The methods must be `pub async fn` and take `&self` as the first parameter.  Others are ignored for this purpose.
//! 2. The `#[actor]` macro will generate the code to make your struct into an actor.  Each method in turn gets turned into a JSON-RPC where:
//! * the RPC name is the name of the method in your `impl`.
//! * the parameters are JSONified parameters of the method.  They are automatically turned into a JSON object with the name of the parameter as the key.  The value is the JSON equivalent of the value of the parameter.
//! * the return value is the JSONified return value of the method.
//! * the macro will generate documentation for your actor so its 100% clear how to call it including the specific JSON payload needed to call the method.
//!
//! 3. The `create` method will start a server for you and begin listening for incoming JSON-RPC calls via HTTP. `create_ws` will do the same but via WebSocket.
//!
//! Note that the `create` and `create_ws` methods consume the actor.  
//! Here is an example with a simple "greet" method:
//! ```rust
//! use simple_json_server::{Actor, actor};
//! #[derive(Debug, Clone)]
//! struct GreetActor;
//! #[actor]
//! impl GreetActor {
//!     pub async fn greet(&self, name: String) -> String {
//!         format!("Hello, {}!", name)
//!     }
//! }
//! 
//! fn main() {
//!     GreetActor.create(8080);
//! }
//! ```
//!
//! You can then (easily) invoke these methods in any way you make a web request.  See `examples/demo/src/main.rs` for a complete example but for example:
//!
//! ```bash
//! # Call the 'greet' method
//! curl -X POST http://127.0.0.1:8080/greet -d '{"name": "World"}'
//! ```

//! The approach is based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model) of computation with some necessary deviations from the pure model. For those not familiar
//! with actors, what that basically means is that:
//! - State is not shared and the only way to communicate is through messages.  
//! - Message cannot contain pointers, etc. There is no shared memory (of course the internal implementation of an actor can be as complex as you like).
//! - The actor is the only owner of its state.  Other actors can send messages but cannot directly access the state.
//!
//! For those very familiar with the actor model, the deviations are:
//! - Rust itself isn't functional in the same way as Lisp. So we eschew `become` as a keyword and generally find completely changing behavior is superfluous for general work.
//! - Each actor is single threaded; we take this approach instead of using an atomic `become` to manage state changes.
//! - Addresses are well known. This is critical to support cross language invocation (the classic client/server case) and contrasts from the Actor model where addresses are passed around.
//! - RPC and return values are supported. Again, this is important to simplify cross language invocation and generally simplifies all distributed code at the cost of asynchronous execution and parallelism. 
//!   In the future we'll look to optimize cases that don't need return values by not waiting for method completion.
//!
//! ## Examples
//!
//! See the `examples/` directory for more comprehensive examples:
//!
//! ```bash
//! # Basic calculator example
//! cargo run --example calculator
//!
//! # HTTP and WebSocket server example
//! cargo run --example server
//!
//! # A full example with docs
//! cd examples/demo
//! cargo run
//! cargo doc --open
//!
//!
//! ```
//!

// Re-export the actor macro
pub use actor_attribute_macro::actor;





/// TLS configuration for secure connections
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the certificate file (PEM format)
    pub cert_path: String,
    /// Path to the private key file (PEM format)
    pub key_path: String,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }

    /// Load the TLS configuration and create a rustls ServerConfig
    pub(crate) async fn load_server_config(&self) -> Result<rustls::ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::BufReader;
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        // Read certificate file
        let mut cert_file = File::open(&self.cert_path).await?;
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data).await?;
        let mut cert_reader = BufReader::new(cert_data.as_slice());
        let cert_chain = certs(&mut cert_reader)?
            .into_iter()
            .map(rustls::Certificate)
            .collect();

        // Read private key file
        let mut key_file = File::open(&self.key_path).await?;
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data).await?;
        let mut key_reader = BufReader::new(key_data.as_slice());
        let mut keys = pkcs8_private_keys(&mut key_reader)?;

        if keys.is_empty() {
            return Err("No private key found".into());
        }

        let private_key = rustls::PrivateKey(keys.remove(0));

        // Create server config
        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;

        Ok(config)
    }
}

/// The Actor trait must be implemented by all servers.  Implementation is most commonly achieved by using
/// the `#[actor]` macro with any other Rust `struct` and `impl`.
pub trait Actor {
    /// Takes a method name and a JSON message, processes it appropriately, and returns a JSON response.
    fn dispatch(&self, method_name: &str, msg: &str) -> String;

    /// Creates a new actor by spawning a thread to listen on the specified port for incoming JSON messages and processes them using dispatch.
    /// If websocket is true, the server will use the websocket protocol instead of HTTP.
    /// This method consumes the actor, preventing further use after starting the server.
    fn create_options(self, port: u16, websocket: bool)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options_with_tls(port, websocket, None)
    }

    /// Creates a new actor with TLS support by spawning a thread to listen on the specified port for incoming JSON messages and processes them using dispatch.
    /// If websocket is true, the server will use the websocket protocol instead of HTTP.
    /// If tls_config is provided, the server will use TLS/SSL encryption.
    /// This method consumes the actor, preventing further use after starting the server.
    fn create_options_with_tls(self, port: u16, websocket: bool, tls_config: Option<TlsConfig>)
    where
        Self: Send + Sync + Sized + 'static,
    {
        let actor = std::sync::Arc::new(self);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if websocket {
                    start_websocket_server_with_tls(actor, port, tls_config).await;
                } else {
                    start_http_server_with_tls(actor, port, tls_config).await;
                }
            });
        });
    }

    fn create(self, port: u16)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, false);
    }

    fn create_ws(self, port: u16)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, true);
    }

    /// Create an HTTPS server with TLS encryption
    fn create_https(self, port: u16, tls_config: TlsConfig)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options_with_tls(port, false, Some(tls_config));
    }

    /// Create a WSS (WebSocket Secure) server with TLS encryption
    fn create_wss(self, port: u16, tls_config: TlsConfig)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options_with_tls(port, true, Some(tls_config));
    }
}

use futures_util::{SinkExt, StreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_tungstenite::{accept_async, tungstenite::Message};


// Implement Actor for Arc<T> where T: Actor
impl<T: Actor + Send + Sync + 'static> Actor for Arc<T> {
    fn dispatch(&self, method_name: &str, msg: &str) -> String {
        (**self).dispatch(method_name, msg)
    }

    fn create_options(self, port: u16, websocket: bool)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options_with_tls(port, websocket, None)
    }

    fn create_options_with_tls(self, port: u16, websocket: bool, tls_config: Option<TlsConfig>)
    where
        Self: Send + Sync + Sized + 'static,
    {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if websocket {
                    start_websocket_server_with_tls(self, port, tls_config).await;
                } else {
                    start_http_server_with_tls(self, port, tls_config).await;
                }
            });
        });
    }
}

/// Start an HTTP server that processes JSON messages
async fn start_http_server<T>(actor: Arc<T>, port: u16)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let make_svc = make_service_fn(move |_conn| {
        let actor = Arc::clone(&actor);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let actor = Arc::clone(&actor);
                async move { handle_http_request(actor, req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("HTTP server listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("HTTP server error: {}", e);
    }
}

/// Handle individual HTTP requests
async fn handle_http_request<T>(
    actor: Arc<T>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible>
where
    T: Actor,
{
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    match (&method, path.as_str()) {
        (&Method::POST, path) => {
            // Extract method name from path (e.g., "/add" -> "add")
            let method_name = path.trim_start_matches('/');

            // Read the request body
            let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Failed to read request body"))
                        .unwrap());
                }
            };

            let body_str = match std::str::from_utf8(&body_bytes) {
                Ok(s) => s,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Invalid UTF-8 in request body"))
                        .unwrap());
                }
            };

            // Process the message using the actor
            let response = actor.dispatch(method_name, body_str);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(response))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

/// Start a WebSocket server that processes JSON messages
async fn start_websocket_server<T>(actor: Arc<T>, port: u16)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let actor = Arc::clone(&actor);
        tokio::spawn(async move {
            if let Err(e) = handle_websocket_connection(actor, stream).await {
                eprintln!("WebSocket connection error: {}", e);
            }
        });
    }
}

/// Handle individual WebSocket connections
async fn handle_websocket_connection<T>(
    actor: Arc<T>,
    stream: tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Actor,
{
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                // Parse the JSON to extract method and params
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(json) => {
                        let method_name = json
                            .get("method")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        let params = json
                            .get("params")
                            .cloned()
                            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                        let params_str =
                            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string());

                        // Process the message using the actor
                        let response = actor.dispatch(method_name, &params_str);

                        // Send response back
                        if let Err(e) = ws_sender.send(Message::Text(response)).await {
                            eprintln!("Failed to send WebSocket response: {}", e);
                            break;
                        }
                    }
                    Err(_) => {
                        let error_response = serde_json::to_string("Invalid JSON").unwrap();
                        if let Err(e) = ws_sender.send(Message::Text(error_response)).await {
                            eprintln!("Failed to send WebSocket error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => {
                break;
            }
            _ => {
                // Ignore other message types (binary, ping, pong)
            }
        }
    }

    Ok(())
}

/// Start an HTTP server with optional TLS support
async fn start_http_server_with_tls<T>(actor: Arc<T>, port: u16, tls_config: Option<TlsConfig>)
where
    T: Actor + Send + Sync + 'static,
{
    match tls_config {
        Some(tls_config) => {
            // HTTPS server with TLS
            match tls_config.load_server_config().await {
                Ok(tls_server_config) => {
                    let addr = SocketAddr::from(([127, 0, 0, 1], port));
                    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_server_config));

                    println!("HTTPS server listening on https://{}", addr);

                    while let Ok((stream, _)) = listener.accept().await {
                        let actor = Arc::clone(&actor);
                        let tls_acceptor = tls_acceptor.clone();

                        tokio::spawn(async move {
                            match tls_acceptor.accept(stream).await {
                                Ok(tls_stream) => {
                                    if let Err(e) = handle_https_connection(actor, tls_stream).await {
                                        eprintln!("HTTPS connection error: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("TLS handshake error: {}", e);
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load TLS configuration: {}", e);
                }
            }
        }
        None => {
            // Regular HTTP server
            start_http_server(actor, port).await;
        }
    }
}

/// Start a WebSocket server with optional TLS support
async fn start_websocket_server_with_tls<T>(actor: Arc<T>, port: u16, tls_config: Option<TlsConfig>)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    match tls_config {
        Some(tls_config) => {
            // WSS server with TLS
            match tls_config.load_server_config().await {
                Ok(tls_server_config) => {
                    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_server_config));

                    println!("WSS server listening on wss://{}", addr);

                    while let Ok((stream, _)) = listener.accept().await {
                        let actor = Arc::clone(&actor);
                        let tls_acceptor = tls_acceptor.clone();

                        tokio::spawn(async move {
                            match tls_acceptor.accept(stream).await {
                                Ok(tls_stream) => {
                                    if let Err(e) = handle_websocket_connection_tls(actor, tls_stream).await {
                                        eprintln!("WSS connection error: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("TLS handshake error: {}", e);
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load TLS configuration: {}", e);
                }
            }
        }
        None => {
            // Regular WebSocket server
            start_websocket_server(actor, port).await;
        }
    }
}

/// Handle individual WebSocket connections over TLS
async fn handle_websocket_connection_tls<T>(
    actor: Arc<T>,
    stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: Actor + Send + Sync + 'static,
{
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                // Parse the JSON message
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        if let (Some(method), Some(params)) = (
                            json.get("method").and_then(|v| v.as_str()),
                            json.get("params"),
                        ) {
                            let params_str = params.to_string();
                            let response = actor.dispatch(method, &params_str);

                            if let Err(e) = ws_sender.send(Message::Text(response)).await {
                                eprintln!("Failed to send WebSocket response: {}", e);
                                break;
                            }
                        } else {
                            let error_response = serde_json::json!({
                                "error": "Invalid message format. Expected {\"method\": \"method_name\", \"params\": {...}}"
                            }).to_string();

                            if let Err(e) = ws_sender.send(Message::Text(error_response)).await {
                                eprintln!("Failed to send WebSocket error response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("JSON parse error: {}", e)
                        }).to_string();

                        if let Err(e) = ws_sender.send(Message::Text(error_response)).await {
                            eprintln!("Failed to send WebSocket error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => {
                break;
            }
            // Ignore other message types (binary, ping, pong)
            _ => {}
        }
    }

    Ok(())
}

/// Handle individual HTTPS connections
async fn handle_https_connection<T>(
    actor: Arc<T>,
    stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: Actor + Send + Sync + 'static,
{
    // For a full HTTPS implementation, you would need to implement HTTP/1.1 parsing
    // This is a simplified version that demonstrates the TLS connection handling
    // In production, you'd want to use hyper-rustls for proper HTTPS support

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0; 1024];
    let mut stream = stream;

    // Read the HTTP request
    match stream.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let request = String::from_utf8_lossy(&buffer[..n]);

            // Simple HTTP parsing - extract method and path
            let lines: Vec<&str> = request.lines().collect();
            if let Some(request_line) = lines.first() {
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "POST" {
                    let path = parts[1];
                    let method_name = path.trim_start_matches('/');

                    // Find the request body (after empty line)
                    let mut body = String::new();
                    let mut found_empty_line = false;
                    for line in lines.iter() {
                        if found_empty_line {
                            body.push_str(line);
                        } else if line.is_empty() {
                            found_empty_line = true;
                        }
                    }

                    // Process the request
                    let response_body = actor.dispatch(method_name, &body);

                    // Send HTTP response
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        response_body.len(),
                        response_body
                    );

                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write HTTPS response: {}", e);
                    }
                } else {
                    // Send 405 Method Not Allowed
                    let response = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
        }
        Ok(_) => {
            // Empty request
        }
        Err(e) => {
            eprintln!("Failed to read HTTPS request: {}", e);
        }
    }

    Ok(())
}

/// Example actor demonstrating the generated documentation
///
/// This actor shows how the `#[actor]` macro generates comprehensive RustDoc
/// documentation for your Actor implementations.
#[derive(Debug, Clone)]
pub struct ExampleActor {
    /// The name of this actor instance
    pub name: String,
    /// A counter that can be incremented
    pub counter: i32,
}

impl ExampleActor {
    /// Create a new ExampleActor with the given name
    pub fn new(name: String) -> Self {
        Self { name, counter: 0 }
    }
}

#[actor]
impl ExampleActor {
    /// Add two numbers together
    ///
    /// This method performs basic arithmetic addition and returns the result.
    /// It demonstrates how methods with multiple parameters are documented.
    pub async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Get the current counter value
    ///
    /// Returns the current value of the internal counter.
    /// This method takes no parameters besides `&self`.
    pub async fn get_counter(&self) -> i32 {
        self.counter
    }

    /// Greet someone with a personalized message
    ///
    /// Creates a friendly greeting that includes both the provided name
    /// and this actor's name. Demonstrates string parameter handling.
    pub async fn greet(&self, name: String) -> String {
        format!("Hello {}, I'm {}!", name, self.name)
    }

    /// Calculate the area of a rectangle
    ///
    /// Given width and height as floating-point numbers, calculates
    /// and returns the area. Shows how floating-point parameters work.
    pub async fn calculate_area(&self, width: f64, height: f64) -> f64 {
        width * height
    }

    /// Check if a number is even
    ///
    /// Returns `true` if the given number is even, `false` otherwise.
    /// Demonstrates boolean return values.
    pub async fn is_even(&self, number: i32) -> bool {
        number % 2 == 0
    }

    /// Get information about this actor
    ///
    /// Returns a formatted string containing the actor's name and counter value.
    /// This method shows how to access actor state in the documentation.
    pub async fn info(&self) -> String {
        format!("ExampleActor '{}' with counter {}", self.name, self.counter)
    }

    /// Simple ping method with no parameters
    ///
    /// A basic method that takes no parameters and returns a fixed response.
    /// Useful for health checks or testing connectivity.
    pub async fn ping(&self) -> String {
        "pong".to_string()
    }

    /// Divide two numbers with error handling
    ///
    /// Performs division and returns a Result to handle division by zero.
    /// Demonstrates how Result types are documented and handled.
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }
}

#[cfg(test)]
mod test_actor;
