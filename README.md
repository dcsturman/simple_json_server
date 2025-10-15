# simple_json_server

This library is intended to make it very easy to build a JSON-based server. Because its JSON it is easily called from multiple languages (think a TypeScript client).
The goal is to provide a lot of functionality out of the box but with excellent ergonomics so that simple servers are easily built without almost any boilerplate.

The approach is based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model) of computation with some necessary deviations from the pure model. We'll document those
deviations here. Initially they are:

- Rust itself isn't functional in the same way as Lisp. So we eschew `become` as a keyword and generally find completely changing behavior is superfluous for general work.
- Each actor is single threaded; we take this approach instead of using `become` to manage state changes.
- Addresses are well known. This is critical to support cross language invocation (the classic client/server case) and contrasts from the Actor model where addresses are passed around.
- RPC and return values are supported. Again, this is important to simplify cross language invocation and generally simplifies all distributed code at the cost of asynchronous execution and parallelism. In the future we'll look to optimize cases that don't need return values by not waiting for method completion.

## Quick Start

The easiest way to create an actor is using the `#[actor]` macro:

```rust
use simple_json_server::{Actor, actor};

#[derive(Debug, Clone)]
struct Calculator {
    memory: f64,
}

impl Calculator {
    fn new() -> Self {
        Self { memory: 0.0 }
    }
}

#[actor]
impl Calculator {
    pub async fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    pub async fn subtract(&self, a: f64, b: f64) -> f64 {
        a - b
    }

    pub async fn get_memory(&self) -> f64 {
        self.memory
    }
}

fn main() {
    let calc = Calculator::new();

    // The macro automatically implements the Actor trait
    let result = calc.dispatch(r#"{"method": "add", "params": {"a": 10.5, "b": 5.2}}"#);
    println!("Result: {}", result); // Prints: Result: 15.7
}
```

## The `#[actor]` Macro

The `#[actor]` procedural macro automatically implements the `Actor` trait for your struct by:

1. **Analyzing public async methods** in the impl block
2. **Generating message structs** for each method's parameters
3. **Creating a dispatch method** that handles JSON messages mapped from the method parameters.

### Features

- **Automatic JSON serialization/deserialization** of parameters and return values
- **Error handling** for invalid JSON, unknown methods, and parameter mismatches
- **Async support** with automatic runtime management
- **Type safety** with generated message structs
- **Snake_case to PascalCase** conversion for message struct names
- **Comprehensive RustDoc generation** with method tables, JSON payload examples, and usage instructions
- **TLS/SSL support** for secure HTTPS and WSS (WebSocket Secure) connections

### Method Requirements

The macro only processes methods that are:

- **Public** (`pub`)
- **Async** (`async fn`)

Private methods and synchronous methods are ignored.

## Server Support

The library includes built-in HTTP and WebSocket server support. Use the `create_options` method to start a server.

**Note**: `create_options` consumes the actor (moves it), preventing accidental use after starting the server. This is a safety feature that ensures the actor is only used within the server context.

```rust
use simple_json_server::{Actor, actor};

#[derive(Debug, Clone)]
struct MyActor {
    name: String,
}

#[actor]
impl MyActor {
    pub async fn greet(&self, name: String) -> String {
        format!("Hello, {}! I'm {}", name, self.name)
    }
}

fn main() {
    // Create separate actor instances since create_options consumes the actor
    let http_actor = MyActor { name: "HTTP-Server".to_string() };
    let ws_actor = MyActor { name: "WebSocket-Server".to_string() };

    // Start HTTP server on port 8080 (consumes http_actor)
    http_actor.create_options(8080, false);

    // Start WebSocket server on port 8081 (consumes ws_actor)
    ws_actor.create_options(8081, true);

    // Keep main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

### HTTP Server

The HTTP server expects POST requests with the method name in the URL path and parameters in the JSON body:

```bash
# Call the 'greet' method
curl -X POST http://127.0.0.1:8080/greet -d '{"name": "World"}'
```

### WebSocket Server

The WebSocket server expects JSON messages in the standard format:

```json
{"method": "greet", "params": {"name": "World"}}
```

## TLS/SSL Support

The library supports secure connections using TLS for both HTTP (HTTPS) and WebSocket (WSS) protocols.

### Setting up TLS

First, create a `TlsConfig` with your certificate and private key files:

```rust
use simple_json_server::{Actor, actor, TlsConfig};

let tls_config = TlsConfig::new("cert.pem", "key.pem");
```

### HTTPS Server

```rust
// Start an HTTPS server
actor.create_https(8443, tls_config);
```

### WSS (WebSocket Secure) Server

```rust
// Start a WSS server
actor.create_wss(8444, tls_config);
```

### Generating Test Certificates

For development and testing, you can generate self-signed certificates:

```bash
# Use the provided script
./generate_cert.sh

# Or manually with OpenSSL
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

### Testing HTTPS Endpoints

```bash
# Use -k flag to accept self-signed certificates
curl -k -X POST https://127.0.0.1:8443/greet -d '{"name": "World"}'
```

### TLS Configuration

The `TlsConfig` struct supports:

- **Certificate file**: PEM format certificate file
- **Private key file**: PEM format private key file (PKCS#8)
- **Automatic loading**: Certificates are loaded asynchronously when the server starts

## Examples

There are examples in the `simple_json_server` crate itself.  See the `examples/` directory for a more complete (yet simple) demo.  The demo will build on its own.

```bash
cd simple_json_server/examples 
# Basic calculator example
cargo run --example calculator

# HTTP and WebSocket server example
cargo run --example server

# TLS/SSL server example (HTTPS and WSS)
cargo run --example tls_server

# The full example
cd ../../examples/demo
cargo run
```

## Actor Trait

The `Actor` trait defines two methods:

```rust
pub trait Actor {
    /// Process a method call with parameters
    fn dispatch(&self, method_name: &str, msg: &str) -> String;

    /// Start a server (HTTP or WebSocket) on the specified port
    /// This method consumes the actor, preventing further use after starting the server
    fn create_options(self, port: u16, websocket: bool)
    where
        Self: Send + Sync + Sized + 'static;
}
```

If you need more control, you can implement the `Actor` trait manually instead of using the `#[actor]` macro.

## Documentation

The `#[actor]` macro automatically generates comprehensive RustDoc documentation for your Actor implementations. To view the generated documentation:

```bash
cargo doc --open
```

The generated documentation includes:

- **Method overview table** with parameters and return types
- **Detailed method documentation** extracted from your doc comments
- **JSON payload examples** for each method
- **Usage examples** showing how to call each method via `dispatch`
- **Parameter specifications** with type information

This makes it easy to understand the JSON API without looking at the source code!

