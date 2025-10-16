use simple_json_server::{Actor, actor};

/// A simple actor to demonstrate the move semantics
#[derive(Debug, Clone)]
pub struct SimpleServerDemo {
    pub id: String,
}

impl SimpleServerDemo {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[actor]
impl SimpleServerDemo {
    pub async fn get_id(&self) -> String {
        self.id.clone()
    }

    pub async fn greet(&self, name: String) -> String {
        format!("Hello {}, I'm {}", name, self.id)
    }
}

#[tokio::main]
async fn main() {
    println!("Demonstrating move semantics with create_options");

    let actor = SimpleServerDemo::new("demo-actor".to_string());

    // We can use the actor normally before starting the server
    println!("Actor ID: {}", actor.id);

    // This would work if we had a dispatch call:
    // let result = actor.dispatch("get_id", "{}");
    // println!("Dispatch result: {}", result);

    println!("Starting server on port 9000...");
    println!("This server supports both HTTP and WebSocket connections.");
    println!();

    // This consumes the actor - after this line, `actor` can no longer be used
    actor.create(9000); // Start HTTP server

    // The following line would cause a compile error because `actor` has been moved:
    // println!("Actor ID after move: {}", actor.id); // ❌ Compile error!

    println!("✅ Server started successfully!");
    println!();
    println!("📡 Available Interfaces:");
    println!("  HTTP:      http://127.0.0.1:9000");
    println!("  WebSocket: ws://127.0.0.1:9000");
    println!();
    println!("🧪 Test with curl:");
    println!("  curl -X POST http://127.0.0.1:9000/get_id -d '{{}}'");
    println!("  curl -X POST http://127.0.0.1:9000/greet -d '{{\"name\": \"World\"}}'");
    println!();
    println!("🔧 Test with JavaScript client:");
    println!("  npm install");
    println!("  node client.js");
    println!();
    println!("🌐 Test with web interface:");
    println!("  Open index.html in your browser");
    println!();
    println!("📚 View generated documentation:");
    println!("  cargo doc --open");
    println!();
    println!("Press Ctrl+C to stop the server");

    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
