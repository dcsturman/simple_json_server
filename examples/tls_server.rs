use simple_json_server::{Actor, actor, TlsConfig};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SecureCalculator {
    pub name: String,
}

impl SecureCalculator {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[actor]
impl SecureCalculator {
    /// Add two numbers
    pub async fn add(&self, a: f64, b: f64) -> f64 {
        println!("[{}] Secure addition: {} + {}", self.name, a, b);
        a + b
    }
    
    /// Subtract two numbers
    pub async fn subtract(&self, a: f64, b: f64) -> f64 {
        println!("[{}] Secure subtraction: {} - {}", self.name, a, b);
        a - b
    }
    
    /// Multiply two numbers
    pub async fn multiply(&self, a: f64, b: f64) -> f64 {
        println!("[{}] Secure multiplication: {} * {}", self.name, a, b);
        a * b
    }
    
    /// Divide two numbers with error handling
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        println!("[{}] Secure division: {} / {}", self.name, a, b);
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }
    
    /// Get server information
    pub async fn info(&self) -> String {
        format!("Secure Calculator '{}' - TLS enabled", self.name)
    }
    
    /// Health check endpoint
    pub async fn health(&self) -> String {
        "Secure OK".to_string()
    }
}

fn main() {
    println!("TLS Server Example");
    println!("==================");
    println!();
    
    // Check if certificate files exist
    let cert_path = "cert.pem";
    let key_path = "key.pem";
    
    if !std::path::Path::new(cert_path).exists() || !std::path::Path::new(key_path).exists() {
        println!("⚠️  Certificate files not found!");
        println!();
        println!("To run this example with TLS, you need to generate certificate files:");
        println!();
        println!("1. Generate a self-signed certificate:");
        println!("   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes");
        println!();
        println!("2. Or use the provided script:");
        println!("   ./generate_cert.sh");
        println!();
        println!("For now, starting regular HTTP/WebSocket servers...");
        println!();
        
        // Start regular servers without TLS
        let http_calc = SecureCalculator::new("HTTP-Server".to_string());
        let ws_calc = SecureCalculator::new("WebSocket-Server".to_string());
        
        println!("Starting HTTP server on port 8080...");
        http_calc.create(8080);
        
        println!("Starting WebSocket server on port 8081...");
        ws_calc.create_ws(8081);
        
        println!();
        println!("Test the servers:");
        println!("  curl -X POST http://127.0.0.1:8080/add -d '{{\"a\": 10, \"b\": 5}}'");
        println!("  curl -X POST http://127.0.0.1:8080/info -d '{{}}'");
        
    } else {
        println!("✅ Certificate files found! Starting TLS servers...");
        println!();
        
        // Create TLS configuration
        let tls_config = TlsConfig::new(cert_path, key_path);
        
        // Create calculator instances
        let https_calc = SecureCalculator::new("HTTPS-Server".to_string());
        let wss_calc = SecureCalculator::new("WSS-Server".to_string());
        
        // Start HTTPS server on port 8443
        println!("Starting HTTPS server on port 8443...");
        https_calc.create_https(8443, tls_config.clone());
        
        // Start WSS server on port 8444
        println!("Starting WSS server on port 8444...");
        wss_calc.create_wss(8444, tls_config);
        
        println!();
        println!("🔒 Secure servers started!");
        println!();
        println!("HTTPS Server Examples:");
        println!("  curl -k -X POST https://127.0.0.1:8443/add -d '{{\"a\": 10, \"b\": 5}}'");
        println!("  curl -k -X POST https://127.0.0.1:8443/divide -d '{{\"a\": 20, \"b\": 4}}'");
        println!("  curl -k -X POST https://127.0.0.1:8443/info -d '{{}}'");
        println!();
        println!("WSS Server:");
        println!("  Connect to wss://127.0.0.1:8444 (use -k flag for self-signed certs)");
        println!("  Send: {{\"method\": \"add\", \"params\": {{\"a\": 10, \"b\": 5}}}}");
        println!("  Send: {{\"method\": \"info\", \"params\": {{}}}}");
        println!();
        println!("Note: Use -k flag with curl to accept self-signed certificates");
    }
    
    println!();
    println!("Press Ctrl+C to stop the servers");
    
    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
