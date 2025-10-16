use simple_json_server::{actor, Actor};
use std::thread;
use std::time::Duration;

/// A simple calculator actor that can be served over HTTP or WebSocket
#[derive(Debug, Clone)]
pub struct ServerCalculator {
    pub name: String,
}

impl ServerCalculator {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[actor]
impl ServerCalculator {
    /// Add two numbers
    pub async fn add(&self, a: f64, b: f64) -> f64 {
        log::info!("[{}] Adding {} + {}", self.name, a, b);
        a + b
    }

    /// Subtract two numbers
    pub async fn subtract(&self, a: f64, b: f64) -> f64 {
        log::info!("[{}] Subtracting {} - {}", self.name, a, b);
        a - b
    }

    /// Multiply two numbers
    pub async fn multiply(&self, a: f64, b: f64) -> f64 {
        log::info!("[{}] Multiplying {} * {}", self.name, a, b);
        a * b
    }

    /// Divide two numbers
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        log::info!("[{}] Dividing {} / {}", self.name, a, b);
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    /// Get server info
    pub async fn info(&self) -> String {
        format!("Calculator Server: {}", self.name)
    }

    /// Health check endpoint
    pub async fn health(&self) -> String {
        "OK".to_string()
    }
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    log::info!("Starting Calculator Servers...");

    // Create calculator instances
    let http_calc = ServerCalculator::new("HTTP-Server".to_string());
    let ws_calc = ServerCalculator::new("WebSocket-Server".to_string());

    // Start HTTP server on port 8080 (consumes http_calc)
    log::info!("Starting HTTP server on port 8080...");
    http_calc.create_options(8080, false, None);

    // Start WebSocket server on port 8081 (consumes ws_calc)
    log::info!("Starting WebSocket server on port 8081...");
    ws_calc.create_options(8081, true, None);

    log::info!("Servers started!");
    println!();
    println!("HTTP Server Examples:");
    println!("  curl -X POST http://127.0.0.1:8080/add -d '{{\"a\": 10, \"b\": 5}}'");
    println!("  curl -X POST http://127.0.0.1:8080/divide -d '{{\"a\": 20, \"b\": 4}}'");
    println!("  curl -X POST http://127.0.0.1:8080/info -d '{{}}'");
    println!();
    println!("WebSocket Server:");
    println!("  Connect to ws://127.0.0.1:8081");
    println!("  Send: {{\"method\": \"add\", \"params\": {{\"a\": 10, \"b\": 5}}}}");
    println!("  Send: {{\"method\": \"info\", \"params\": {{}}}}");
    println!();
    println!("Press Ctrl+C to stop the servers");

    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
