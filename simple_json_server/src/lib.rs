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
//! #[tokio::main]
//! async fn main() {
//!     GreetActor::new().create(8080);
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

#![allow(clippy::needless_doctest_main)]

// Re-export the actor macro
pub use actor_attribute_macro::actor;

pub mod tls;
pub use tls::TlsConfig;

/// The Actor trait must be implemented by all servers.  Implementation is most commonly achieved by using
/// the `#[actor]` macro with any other Rust `struct` and `impl`.
pub trait Actor {
    /// Takes a method name and a JSON message, processes it appropriately, and returns a JSON response.
    fn dispatch(
        &self,
        method_name: &str,
        msg: &str,
    ) -> impl std::future::Future<Output = String> + Send;

    /// Creates a new actor with TLS support by spawning a thread to listen on the specified port for incoming JSON messages and processes them using dispatch.
    /// If websocket is true, the server will use the websocket protocol instead of HTTP.
    /// If tls_config is provided, the server will use TLS/SSL encryption.
    /// This method consumes the actor, preventing further use after starting the server.
    fn create_options(self, port: u16, websocket: bool, tls_config: Option<TlsConfig>)
    where
        Self: Send + Sync + Sized + 'static,
    {
        let actor = std::sync::Arc::new(self);

        // Try to spawn on existing runtime first, fallback to new thread with runtime
        let handle = tokio::runtime::Handle::current();
        handle.spawn(async move {
            match (websocket, tls_config) {
                (true, Some(tls_config)) => {
                    start_websocket_server_with_tls(actor, port, tls_config).await;
                }
                (true, None) => {
                    start_websocket_server(actor, port).await;
                }
                (false, Some(tls_config)) => {
                    start_http_server_with_tls(actor, port, tls_config).await;
                }
                (false, None) => {
                    start_http_server(actor, port).await;
                }
            }
        });
    }

    /// Creates a new actor using HTTP and without TLS. The simplest case so with the least
    /// boilerplate.
    ///
    /// This method consumes the actor, preventing further use after starting the server.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    fn create(self, port: u16)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, false, None);
    }

    /// Creates a new actor using WebSocket and without TLS.
    ///
    /// This method consumes the actor, preventing further use after starting the server.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    fn create_ws(self, port: u16)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, true, None);
    }

    /// Creates a new actor using HTTP and with TLS encryption.
    ///
    /// This method consumes the actor, preventing further use after starting the server.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    /// * `tls_config` - The TLS configuration
    fn create_https(self, port: u16, tls_config: TlsConfig)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, false, Some(tls_config));
    }

    /// Create a WSS (WebSocket Secure) server with TLS encryption
    ///
    /// This method consumes the actor, preventing further use after starting the server.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    /// * `tls_config` - The TLS configuration
    fn create_wss(self, port: u16, tls_config: TlsConfig)
    where
        Self: Send + Sync + Sized + 'static,
    {
        self.create_options(port, true, Some(tls_config));
    }
}

use futures_util::{SinkExt, StreamExt};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Start an HTTP server that processes JSON messages
async fn start_http_server<T>(actor: Arc<T>, port: u16)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        panic!("Failed to bind HTTP server to {addr:?}: {}", e);
    });

    log::info!("HTTP server listening on http://{}", addr);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        let actor = Arc::clone(&actor);

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| {
                let actor = Arc::clone(&actor);
                async move { handle_http_request(actor, req).await }
            });

            if let Err(e) = Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                log::error!("HTTP connection error: {}", e);
            }
        });
    }
}

/// Handle individual HTTP requests (unified for HTTP and HTTPS)
async fn handle_http_request<T>(
    actor: Arc<T>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible>
where
    T: Actor + Send + Sync + 'static,
{
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();

    // Read the request body
    let body_str = match http_body_util::BodyExt::collect(req.into_body()).await {
        Ok(collected) => match std::str::from_utf8(&collected.to_bytes()) {
            Ok(s) => s.to_string(),
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from("Invalid UTF-8 in request body")))
                    .unwrap());
            }
        },
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Failed to read request body")))
                .unwrap());
        }
    };

    // Process the HTTP request
    if method == "POST" {
        // Extract method name from path (e.g., "/add" -> "add")
        let method_name = path.trim_start_matches('/');

        // Process the message using the actor
        let response_body = (*actor).dispatch(method_name, &body_str).await;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .body(Full::new(Bytes::from(response_body)))
            .unwrap())
    } else if method == "OPTIONS" {
        // Handle CORS preflight requests
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Content-Length", "0")
            .body(Full::new(Bytes::new()))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Content-Type", "text/plain")
            .header("Access-Control-Allow-Origin", "*")
            .body(Full::new(Bytes::from("Method Not Allowed")))
            .unwrap())
    }
}

/// Start a WebSocket server that processes JSON messages
async fn start_websocket_server<T>(actor: Arc<T>, port: u16)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to bind WebSocket server with address {addr:?}: {}",
                e
            );
        });

    log::info!("WebSocket server listening on ws://{}", addr);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to accept WebSocket connection: {}", e);
                continue;
            }
        };

        let actor = Arc::clone(&actor);
        tokio::spawn(async move {
            // Handle WebSocket upgrade and connection
            if let Err(e) = handle_websocket_connection(actor, stream).await {
                log::error!("WebSocket connection error: {}", e);
            }
        });
    }
}

/// Handle individual WebSocket connections (unified for both TLS and non-TLS)
async fn handle_websocket_connection<T, S>(
    actor: Arc<T>,
    stream: S,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: Actor + Send + Sync + 'static,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                // Parse the JSON message
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        // TLS behavior: strict validation
                        if let (Some(method), Some(params)) = (
                            json.get("method").and_then(|v| v.as_str()),
                            json.get("params"),
                        ) {
                            let params_str = params.to_string();
                            let response = (*actor).dispatch(method, &params_str).await;

                            if let Err(_e) = ws_sender.send(Message::Text(response)).await {
                                log::error!("Failed to send WebSocket response: {}", _e);
                                break;
                            }
                        } else {
                            let error_response = serde_json::json!({
                                    "error": "Invalid message format. Expected {\"method\": \"method_name\", \"params\": {...}}"
                                }).to_string();

                            if let Err(e) = ws_sender.send(Message::Text(error_response)).await {
                                log::error!("Failed to send WebSocket error response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let error_response =
                            serde_json::json!({"error": format!("JSON parse error: {}", e)})
                                .to_string();

                        if let Err(e) = ws_sender.send(Message::Text(error_response)).await {
                            log::error!("Failed to send WebSocket error response: {}", e);
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
async fn start_http_server_with_tls<T>(actor: Arc<T>, port: u16, tls_config: TlsConfig)
where
    T: Actor + Send + Sync + 'static,
{
    // HTTPS server with TLS
    match tls_config.load_server_config().await {
        Ok(tls_server_config) => {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_server_config));

            log::info!("HTTPS server listening on https://{}", addr);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("Failed to accept HTTPS connection: {}", e);
                        continue;
                    }
                };

                let actor = Arc::clone(&actor);
                let tls_acceptor = tls_acceptor.clone();

                tokio::spawn(async move {
                    match tls_acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(e) = handle_https_connection(actor, tls_stream).await {
                                log::error!("HTTPS connection error: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("TLS handshake error: {}", e);
                        }
                    }
                });
            }
        }
        Err(e) => {
            log::error!("Failed to load TLS configuration: {}", e);
        }
    }
}

/// Start a WebSocket server with optional TLS support
async fn start_websocket_server_with_tls<T>(actor: Arc<T>, port: u16, tls_config: TlsConfig)
where
    T: Actor + Send + Sync + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to bind WSS server address {addr:?}: {}", e);
        });

    // WSS server with TLS
    match tls_config.load_server_config().await {
        Ok(tls_server_config) => {
            let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_server_config));

            log::info!("WSS server listening on wss://{}", addr);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("Failed to accept WSS connection: {}", e);
                        continue;
                    }
                };

                let actor = Arc::clone(&actor);
                let tls_acceptor = tls_acceptor.clone();

                tokio::spawn(async move {
                    match tls_acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(e) = handle_websocket_connection(actor, tls_stream).await {
                                log::error!("WSS connection error: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("TLS handshake error: {}", e);
                        }
                    }
                });
            }
        }
        Err(e) => {
            log::error!("Failed to load TLS configuration: {}", e);
        }
    }
}

/// Handle individual HTTPS connections using hyper-rustls
async fn handle_https_connection<T>(
    actor: Arc<T>,
    stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: Actor + Send + Sync + 'static,
{
    let io = TokioIo::new(stream);

    let service = service_fn(move |req| {
        let actor = actor.clone();
        async move { handle_http_request(actor, req).await }
    });

    // Serve the HTTP request using hyper 1.7 API
    if let Err(e) = Builder::new(hyper_util::rt::TokioExecutor::new())
        .serve_connection(io, service)
        .await
    {
        log::error!("HTTPS connection error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod test_actor;
