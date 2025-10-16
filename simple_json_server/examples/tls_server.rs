use simple_json_server::{Actor, TlsConfig, actor};
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
        log::info!("[{}] Secure addition: {} + {}", self.name, a, b);
        a + b
    }

    /// Subtract two numbers
    pub async fn subtract(&self, a: f64, b: f64) -> f64 {
        log::info!("[{}] Secure subtraction: {} - {}", self.name, a, b);
        a - b
    }

    /// Multiply two numbers
    pub async fn multiply(&self, a: f64, b: f64) -> f64 {
        log::info!("[{}] Secure multiplication: {} * {}", self.name, a, b);
        a * b
    }

    /// Divide two numbers with error handling
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        log::info!("[{}] Secure division: {} / {}", self.name, a, b);
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

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    log::info!("TLS Server Example");
    log::info!("==================");

    // Check if certificate files exist
    let cert_path = "cert.pem";
    let key_path = "key.pem";

    if !std::path::Path::new(cert_path).exists() || !std::path::Path::new(key_path).exists() {
        println!("⚠️  Certificate files not found!");
        println!();
        println!("To run this example with TLS, you need to generate certificate files:");
        println!();
        println!("1. Generate a self-signed certificate:");
        println!(
            "   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes"
        );
        println!();
        println!("2. Or use the provided script:");
        println!("   ../generate_cert.sh");
        println!();
        println!();
    } else {
        println!("✅ Certificate files found! Starting TLS servers...");
        println!();

        // Create TLS configuration
        let tls_config = TlsConfig::new(cert_path, key_path);

        // Create calculator instances
        let https_calc = SecureCalculator::new("HTTPS-Server".to_string());
        let wss_calc = SecureCalculator::new("WSS-Server".to_string());

        // Start HTTPS server on port 8443
        log::info!("Starting HTTPS server on port 8443...");
        https_calc.create_https(8443, tls_config.clone());

        // Start WSS server on port 8444
        log::info!("Starting WSS server on port 8444...");
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

        println!();
        println!("Press Ctrl+C to stop the servers");

        // Keep the main thread alive
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
