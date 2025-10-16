use serde_json::json;
use simple_json_server::{actor, Actor, TlsConfig};
use std::fs;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::time::sleep;

// Global port counter starting from a high port number
static PORT_COUNTER: AtomicU16 = AtomicU16::new(40000);

// Helper function to get the next available port
fn get_next_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// Helper function to create self-signed certificates for testing
fn create_test_certificates() -> (String, String) {
    use rcgen::{Certificate, CertificateParams, DistinguishedName};

    let mut params = CertificateParams::new(vec!["localhost".to_string()]);
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "localhost");

    let cert = Certificate::from_params(params).expect("Failed to generate certificate");
    let cert_pem = cert
        .serialize_pem()
        .expect("Failed to serialize certificate");
    let key_pem = cert.serialize_private_key_pem();

    // Write to temporary files
    let cert_path = "/tmp/test_cert.pem";
    let key_path = "/tmp/test_key.pem";

    fs::write(cert_path, cert_pem).expect("Failed to write certificate");
    fs::write(key_path, key_pem).expect("Failed to write private key");

    (cert_path.to_string(), key_path.to_string())
}

#[derive(Debug, Clone)]
pub struct TestServer {
    pub name: String,
}

impl TestServer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[actor]
impl TestServer {
    /// Add two numbers
    pub async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Greet someone
    pub async fn greet(&self, name: String) -> String {
        format!("Hello, {}! I'm {}", name, self.name)
    }

    /// Get server info
    pub async fn info(&self) -> String {
        format!("Test server: {}", self.name)
    }

    /// Echo back the input
    pub async fn echo(&self, message: String) -> String {
        message
    }

    /// Test method with no parameters
    pub async fn ping(&self) -> String {
        "pong".to_string()
    }

    /// Test method that returns a Result
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }
}

#[tokio::test]
async fn test_http_server_end_to_end() {
    // Start HTTP server on an available port
    let port = get_next_port();
    let server = TestServer::new("HTTP-E2E-Test".to_string());

    // Start the server in the background
    server.create(port);

    // Give the server time to start
    sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test 1: Add method
    let response = client
        .post(format!("{base_url}/add"))
        .json(&json!({"a": 10, "b": 5}))
        .send()
        .await
        .expect("Failed to send add request");

    assert_eq!(response.status(), 200);
    let result: i32 = response.json().await.expect("Failed to parse add response");
    assert_eq!(result, 15);

    // Test 2: Greet method
    let response = client
        .post(format!("{base_url}/greet"))
        .json(&json!({"name": "World"}))
        .send()
        .await
        .expect("Failed to send greet request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse greet response");
    assert_eq!(result, "Hello, World! I'm HTTP-E2E-Test");

    // Test 3: Info method (no parameters)
    let response = client
        .post(format!("{base_url}/info"))
        .json(&json!({}))
        .send()
        .await
        .expect("Failed to send info request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse info response");
    assert_eq!(result, "Test server: HTTP-E2E-Test");

    // Test 4: Ping method (no parameters)
    let response = client
        .post(format!("{base_url}/ping"))
        .json(&json!({}))
        .send()
        .await
        .expect("Failed to send ping request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse ping response");
    assert_eq!(result, "pong");

    // Test 5: Echo method
    let response = client
        .post(format!("{base_url}/echo"))
        .json(&json!({"message": "Hello, Echo!"}))
        .send()
        .await
        .expect("Failed to send echo request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse echo response");
    assert_eq!(result, "Hello, Echo!");

    // Test 6: Divide method (success case)
    let response = client
        .post(format!("{base_url}/divide"))
        .json(&json!({"a": 20.0, "b": 4.0}))
        .send()
        .await
        .expect("Failed to send divide request");

    assert_eq!(response.status(), 200);
    let result: Result<f64, String> = response
        .json()
        .await
        .expect("Failed to parse divide response");
    assert_eq!(result, Ok(5.0));

    // Test 7: Divide method (error case)
    let response = client
        .post(format!("{base_url}/divide"))
        .json(&json!({"a": 10.0, "b": 0.0}))
        .send()
        .await
        .expect("Failed to send divide by zero request");

    assert_eq!(response.status(), 200);
    let result: Result<f64, String> = response
        .json()
        .await
        .expect("Failed to parse divide by zero response");
    assert_eq!(result, Err("Division by zero".to_string()));

    // Test 8: Unknown method (should return error)
    let response = client
        .post(format!("{base_url}/unknown_method"))
        .json(&json!({}))
        .send()
        .await
        .expect("Failed to send unknown method request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .text()
        .await
        .expect("Failed to get unknown method response");
    assert!(result.contains("Unknown method"));

    // Test 9: Invalid JSON (should return error)
    let response = client
        .post(format!("{base_url}/add"))
        .body("invalid json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send invalid JSON request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .text()
        .await
        .expect("Failed to get invalid JSON response");
    assert!(result.contains("Failed to parse JSON"));

    println!("✅ All HTTP end-to-end tests passed!");
}

#[tokio::test]
async fn test_http_method_not_allowed() {
    // Start HTTP server on an available port
    let port = get_next_port();
    let server = TestServer::new("HTTP-Method-Test".to_string());

    // Start the server in the background
    server.create(port);

    // Give the server time to start
    sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test 1: GET request (should return 405 Method Not Allowed)
    let response = client
        .get(format!("{base_url}/add"))
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), 405); // Method Not Allowed
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/plain"
    );
    let result = response.text().await.expect("Failed to get GET response");
    assert_eq!(result, "Method Not Allowed");

    // Test 2: PUT request (should return 405 Method Not Allowed)
    let response = client
        .put(format!("{base_url}/add"))
        .json(&json!({"a": 10, "b": 5}))
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), 405); // Method Not Allowed
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/plain"
    );
    let result = response.text().await.expect("Failed to get PUT response");
    assert_eq!(result, "Method Not Allowed");

    // Test 3: DELETE request (should return 405 Method Not Allowed)
    let response = client
        .delete(format!("{base_url}/add"))
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), 405); // Method Not Allowed
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/plain"
    );
    let result = response
        .text()
        .await
        .expect("Failed to get DELETE response");
    assert_eq!(result, "Method Not Allowed");

    // Test 4: PATCH request (should return 405 Method Not Allowed)
    let response = client
        .patch(format!("{base_url}/add"))
        .json(&json!({"a": 10, "b": 5}))
        .send()
        .await
        .expect("Failed to send PATCH request");

    assert_eq!(response.status(), 405); // Method Not Allowed
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/plain"
    );
    let result = response.text().await.expect("Failed to get PATCH response");
    assert_eq!(result, "Method Not Allowed");

    // Test 5: Verify POST still works (should return 200)
    let response = client
        .post(format!("{base_url}/add"))
        .json(&json!({"a": 10, "b": 5}))
        .send()
        .await
        .expect("Failed to send POST request");

    assert_eq!(response.status(), 200);
    let result: i32 = response
        .json()
        .await
        .expect("Failed to parse POST response");
    assert_eq!(result, 15);

    println!("✅ All HTTP method tests passed!");
}

#[tokio::test]
async fn test_websocket_server_end_to_end() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Start WebSocket server on an available port
    let port = get_next_port();
    let server = TestServer::new("WS-E2E-Test".to_string());

    // Start the server in the background
    println!("Starting WebSocket server on port {port}...");
    server.create_ws(port);

    println!("WebSocket server started on port {port}");
    // Give the server more time to start
    sleep(Duration::from_millis(500)).await;
    let url = format!("ws://127.0.0.1:{}", port);

    println!("Try connecting to {url}");
    let ws_stream = connect_async(&url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to {url}: {}", e));

    let (mut ws_sender, mut ws_receiver) = ws_stream.0.split();

    // Test 1: Add method
    let add_msg = json!({
        "method": "add",
        "params": {"a": 15, "b": 25}
    });
    ws_sender
        .send(Message::Text(add_msg.to_string()))
        .await
        .expect("Failed to send add message");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive add response");
        if let Message::Text(text) = response {
            let result: i32 = serde_json::from_str(&text).expect("Failed to parse add response");
            assert_eq!(result, 40);
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("No response received for add");
    }

    // Test 2: Greet method
    let greet_msg = json!({
        "method": "greet",
        "params": {"name": "WebSocket"}
    });
    ws_sender
        .send(Message::Text(greet_msg.to_string()))
        .await
        .expect("Failed to send greet message");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive greet response");
        if let Message::Text(text) = response {
            let result: String =
                serde_json::from_str(&text).expect("Failed to parse greet response");
            assert_eq!(result, "Hello, WebSocket! I'm WS-E2E-Test");
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("No response received for greet");
    }

    // Test 3: Ping method (no parameters)
    let ping_msg = json!({
        "method": "ping",
        "params": {}
    });
    ws_sender
        .send(Message::Text(ping_msg.to_string()))
        .await
        .expect("Failed to send ping message");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive ping response");
        if let Message::Text(text) = response {
            let result: String =
                serde_json::from_str(&text).expect("Failed to parse ping response");
            assert_eq!(result, "pong");
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("No response received for ping");
    }

    println!("✅ All WebSocket end-to-end tests passed!");
}

#[tokio::test]
async fn test_websocket_invalid_message_format() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Start WebSocket server on an available port
    let port = get_next_port();
    let server = TestServer::new("WS-Invalid-Test".to_string());

    // Start the server in the background
    println!("Starting WebSocket server for invalid message test on port {port}...");
    server.create_ws(port);

    println!("WebSocket server started on port {port}");
    // Give the server time to start
    sleep(Duration::from_millis(500)).await;
    let url = format!("ws://127.0.0.1:{}", port);

    println!("Connecting to {url} for invalid message test");
    let ws_stream = connect_async(&url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to {url}: {}", e));

    let (mut ws_sender, mut ws_receiver) = ws_stream.0.split();

    // Test 1: Completely malformed JSON (should trigger JSON parse error)
    let malformed_json = "{invalid json}";
    ws_sender
        .send(Message::Text(malformed_json.to_string()))
        .await
        .expect("Failed to send malformed JSON");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive response for malformed JSON");
        if let Message::Text(text) = response {
            // The non-TLS WebSocket handler returns "Invalid JSON" as a simple string
            // The TLS WebSocket handler returns a JSON object with an "error" field
            if text.starts_with('{') {
                // JSON object response (TLS handler)
                let error_response: serde_json::Value =
                    serde_json::from_str(&text).expect("Failed to parse error response");
                assert!(error_response.get("error").is_some());
                let error_msg = error_response["error"].as_str().unwrap();
                assert!(error_msg.contains("JSON parse error"));
            } else {
                // Simple string response (non-TLS handler)
                assert!(text.contains("Invalid JSON"));
            }
        } else {
            panic!("Expected text message for malformed JSON");
        }
    } else {
        panic!("No response received for malformed JSON");
    }

    // Test 2: Missing "method" field (non-TLS handler defaults to "unknown")
    let missing_method = json!({
        "params": {"a": 10, "b": 5}
    });
    ws_sender
        .send(Message::Text(missing_method.to_string()))
        .await
        .expect("Failed to send message with missing method");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive response for missing method");
        if let Message::Text(text) = response {
            // Non-TLS handler defaults to "unknown" method, so we get an "Unknown method" response
            assert!(
                text.contains("Invalid message format."),
                "Expected 'Invalid message format', instead found: {}",
                text
            );
        } else {
            panic!("Expected text message for missing method");
        }
    } else {
        panic!("No response received for missing method");
    }

    // Test 3: Missing "params" field (non-TLS handler defaults to empty object)
    let missing_params = json!({
        "method": "add"
    });
    ws_sender
        .send(Message::Text(missing_params.to_string()))
        .await
        .expect("Failed to send message with missing params");

    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive response for missing params");
        if let Message::Text(text) = response {
            // Non-TLS handler defaults to empty params {}, so "add" method gets called with no params
            // This should result in a parameter parsing error from the actor
            assert!(
                text.contains("error") || text.contains("missing") || text.contains("required")
            );
        } else {
            panic!("Expected text message for missing params");
        }
    } else {
        panic!("No response received for missing params");
    }

    println!("✅ All WebSocket invalid message format tests passed!");
}

#[tokio::test]
async fn test_websocket_message_types() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Start WebSocket server on an available port
    let port = get_next_port();
    let server = TestServer::new("WS-MessageTypes-Test".to_string());

    // Start the server in the background
    println!("Starting WebSocket server for message types test on port {port}...");
    server.create_ws(port);

    println!("WebSocket server started on port {port}");
    // Give the server time to start
    sleep(Duration::from_millis(500)).await;
    let url = format!("ws://127.0.0.1:{}", port);

    println!("Connecting to {url} for message types test");
    let ws_stream = connect_async(&url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to {url}: {}", e));

    let (mut ws_sender, mut ws_receiver) = ws_stream.0.split();

    // Test 1: Send a valid message first to ensure connection works
    let valid_message = json!({
        "method": "ping",
        "params": {}
    });
    ws_sender
        .send(Message::Text(valid_message.to_string()))
        .await
        .expect("Failed to send valid message");

    // Receive the response to clear the queue
    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive response for valid message");
        if let Message::Text(text) = response {
            assert_eq!(text, "\"pong\"");
        } else {
            panic!("Expected text message for valid ping");
        }
    } else {
        panic!("No response received for valid ping");
    }

    // Test 2: Send a binary message (should be ignored)
    let binary_data = vec![0x01, 0x02, 0x03, 0x04];
    ws_sender
        .send(Message::Binary(binary_data))
        .await
        .expect("Failed to send binary message");

    // Test 3: Send a ping message (should be ignored by our handler, but WebSocket protocol handles it)
    ws_sender
        .send(Message::Ping(vec![0x01, 0x02]))
        .await
        .expect("Failed to send ping message");

    // Test 4: Send another valid message to ensure server is still responsive after binary/ping
    let another_valid_message = json!({
        "method": "info",
        "params": {}
    });
    ws_sender
        .send(Message::Text(another_valid_message.to_string()))
        .await
        .expect("Failed to send second valid message");

    // Receive the response for the info message
    // Note: We might receive a pong response first due to the ping message we sent earlier
    let mut received_info_response = false;
    for _ in 0..2 {
        // Try up to 2 messages to handle potential pong + text response
        if let Some(msg) = ws_receiver.next().await {
            let response = msg.expect("Failed to receive response");
            match response {
                Message::Text(text) => {
                    assert_eq!(text, "\"Test server: WS-MessageTypes-Test\"");
                    received_info_response = true;
                    break;
                }
                Message::Pong(_) => {
                    // WebSocket protocol automatically responds to ping with pong, ignore it
                    continue;
                }
                other => {
                    panic!("Unexpected message type: {:?}", other);
                }
            }
        }
    }

    if !received_info_response {
        panic!("Did not receive expected info response");
    }

    // Test 5: Send a close message (this should terminate the connection)
    ws_sender
        .send(Message::Close(None))
        .await
        .expect("Failed to send close message");

    // The connection should close, so we shouldn't receive any more messages
    // We can verify this by trying to send another message and expecting it to fail
    let result = ws_sender
        .send(Message::Text("should fail".to_string()))
        .await;

    // The send should fail because the connection is closed
    assert!(result.is_err(), "Expected send to fail after close message");

    println!("✅ All WebSocket message types tests passed!");
}

#[tokio::test]
async fn test_websocket_close_message() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Start WebSocket server on an available port
    let port = get_next_port();
    let server = TestServer::new("WS-Close-Test".to_string());

    // Start the server in the background
    println!("Starting WebSocket server for close message test on port {port}...");
    server.create_ws(port);

    println!("WebSocket server started on port {port}");
    // Give the server time to start
    sleep(Duration::from_millis(500)).await;
    let url = format!("ws://127.0.0.1:{}", port);

    println!("Connecting to {url} for close message test");
    let ws_stream = connect_async(&url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to {url}: {}", e));

    let (mut ws_sender, mut ws_receiver) = ws_stream.0.split();

    // Send a valid message first to ensure connection works
    let valid_message = json!({
        "method": "ping",
        "params": {}
    });
    ws_sender
        .send(Message::Text(valid_message.to_string()))
        .await
        .expect("Failed to send valid message");

    // Receive the response
    if let Some(msg) = ws_receiver.next().await {
        let response = msg.expect("Failed to receive response for valid message");
        if let Message::Text(text) = response {
            assert_eq!(text, "\"pong\"");
        } else {
            panic!("Expected text message for valid ping");
        }
    } else {
        panic!("No response received for valid ping");
    }

    // Now try to send a close message with a custom close code
    // This might be processed by our application before the connection closes
    use tokio_tungstenite::tungstenite::protocol::CloseFrame;
    let close_frame = CloseFrame {
        code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
        reason: "Test close".into(),
    };

    println!("Sending close message...");
    ws_sender
        .send(Message::Close(Some(close_frame)))
        .await
        .expect("Failed to send close message");

    // Try to receive any response or close acknowledgment
    // The connection should close, but we might see the close message processed
    if let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => {
                println!("Received close acknowledgment from server");
            }
            Ok(other) => {
                println!("Received unexpected message: {:?}", other);
            }
            Err(e) => {
                println!("Connection closed with error: {}", e);
            }
        }
    } else {
        println!("Connection closed without additional messages");
    }

    // Verify that subsequent sends fail
    let result = ws_sender
        .send(Message::Text("should fail".to_string()))
        .await;

    assert!(result.is_err(), "Expected send to fail after close message");

    println!("✅ WebSocket close message test completed!");
}

#[tokio::test]
async fn test_dispatch_functionality() {
    // Test the dispatch functionality directly without network layer
    let server = TestServer::new("Direct-Test".to_string());

    // Test 1: Add method
    let result = server.dispatch("add", r#"{"a": 10, "b": 5}"#).await;
    assert_eq!(result, "15");

    // Test 2: Greet method
    let result = server.dispatch("greet", r#"{"name": "Direct"}"#).await;
    assert_eq!(result, r#""Hello, Direct! I'm Direct-Test""#);

    // Test 3: Info method
    let result = server.dispatch("info", r#"{}"#).await;
    assert_eq!(result, r#""Test server: Direct-Test""#);

    // Test 4: Ping method
    let result = server.dispatch("ping", r#"{}"#).await;
    assert_eq!(result, r#""pong""#);

    // Test 5: Echo method
    let result = server.dispatch("echo", r#"{"message": "test"}"#).await;
    assert_eq!(result, r#""test""#);

    // Test 6: Divide method (success)
    let result = server.dispatch("divide", r#"{"a": 20.0, "b": 4.0}"#).await;
    assert_eq!(result, r#"{"Ok":5.0}"#);

    // Test 7: Divide method (error)
    let result = server.dispatch("divide", r#"{"a": 10.0, "b": 0.0}"#).await;
    assert_eq!(result, r#"{"Err":"Division by zero"}"#);

    // Test 8: Unknown method
    let result = server.dispatch("unknown", r#"{}"#).await;
    assert!(result.contains("Unknown method"));

    // Test 9: Invalid JSON
    let result = server.dispatch("add", "invalid json").await;
    assert!(result.contains("Failed to parse JSON"));

    println!("✅ All direct dispatch tests passed!");
}

#[tokio::test]
async fn test_https_server_end_to_end() {
    // Create self-signed certificates for testing
    let (cert_path, key_path) = create_test_certificates();

    // Start HTTPS server on an available port
    let port = get_next_port();
    let server = TestServer::new("HTTPS-E2E-Test".to_string());

    // Create TLS config
    let tls_config = TlsConfig::new(cert_path, key_path);

    // Start the server in the background
    server.create_https(port, tls_config);

    // Give the server time to start
    sleep(Duration::from_millis(500)).await;

    // Create a client that accepts self-signed certificates
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTPS client");

    let base_url = format!("https://127.0.0.1:{}", port);

    // Test 1: Add method
    let response = client
        .post(format!("{}/add", base_url))
        .json(&json!({"a": 15, "b": 25}))
        .send()
        .await
        .expect("Failed to send HTTPS add request");

    assert_eq!(response.status(), 200);
    let result: i32 = response
        .json()
        .await
        .expect("Failed to parse HTTPS response");
    assert_eq!(result, 40);

    // Test 2: Greet method
    let response = client
        .post(format!("{}/greet", base_url))
        .json(&json!({"name": "TLS"}))
        .send()
        .await
        .expect("Failed to send HTTPS greet request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse HTTPS greet response");
    assert_eq!(result, "Hello, TLS! I'm HTTPS-E2E-Test");

    // Test 3: Info method (no parameters)
    let response = client
        .post(format!("{}/info", base_url))
        .json(&json!({}))
        .send()
        .await
        .expect("Failed to send HTTPS info request");

    assert_eq!(response.status(), 200);
    let result: String = response
        .json()
        .await
        .expect("Failed to parse HTTPS info response");
    assert_eq!(result, "Test server: HTTPS-E2E-Test");

    // Clean up certificate files
    let _ = fs::remove_file("/tmp/test_cert.pem");
    let _ = fs::remove_file("/tmp/test_key.pem");

    println!("✅ All HTTPS end-to-end tests passed!");
}

#[tokio::test]
async fn test_wss_server_end_to_end() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::protocol::Message;
    use tokio_tungstenite::{connect_async_tls_with_config, Connector};

    let port = get_next_port();
    let (cert_path, key_path) = create_test_certificates();

    // Create and start the WSS server
    let server = TestServer::new("WSS-E2E-Test".to_string());
    let tls_config = TlsConfig::new(&cert_path, &key_path);
    server.create_wss(port, tls_config);

    // Give the server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create TLS connector that accepts self-signed certificates
    let connector = Connector::NativeTls(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .expect("Failed to create TLS connector"),
    );

    let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default();

    // Connect to the WSS server
    let url = format!("wss://localhost:{}", port);
    let (ws_stream, _) = connect_async_tls_with_config(&url, Some(config), false, Some(connector))
        .await
        .expect("Failed to connect to WSS server");

    let (mut write, mut read) = ws_stream.split();

    // Test add method
    let add_msg = r#"{"method": "add", "params": {"a": 5, "b": 3}}"#;
    write
        .send(Message::Text(add_msg.to_string()))
        .await
        .expect("Failed to send message");

    if let Some(Ok(Message::Text(response))) = read.next().await {
        assert_eq!(response, "8");
    } else {
        panic!("Expected text response from WSS server");
    }

    // Test greet method
    let greet_msg = r#"{"method": "greet", "params": {"name": "WSS"}}"#;
    write
        .send(Message::Text(greet_msg.to_string()))
        .await
        .expect("Failed to send message");

    if let Some(Ok(Message::Text(response))) = read.next().await {
        assert_eq!(response, r#""Hello, WSS! I'm WSS-E2E-Test""#);
    } else {
        panic!("Expected text response from WSS server");
    }

    // Test error case - unknown method
    let unknown_msg = r#"{"method": "unknown", "params": {}}"#;
    write
        .send(Message::Text(unknown_msg.to_string()))
        .await
        .expect("Failed to send message");

    if let Some(Ok(Message::Text(response))) = read.next().await {
        assert!(response.contains("Unknown method"));
    } else {
        panic!("Expected error response from WSS server");
    }

    // Clean up certificate files
    let _ = fs::remove_file("/tmp/test_cert.pem");
    let _ = fs::remove_file("/tmp/test_key.pem");

    println!("✅ All WSS end-to-end tests passed!");
}
