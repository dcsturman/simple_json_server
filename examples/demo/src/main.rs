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

fn main() {
    println!("Demonstrating move semantics with create_options");
    
    let actor = SimpleServerDemo::new("demo-actor".to_string());
    
    // We can use the actor normally before starting the server
    println!("Actor ID: {}", actor.id);
    
    // This would work if we had a dispatch call:
    // let result = actor.dispatch("get_id", "{}");
    // println!("Dispatch result: {}", result);
    
    println!("Starting server on port 9000...");
    
    // This consumes the actor - after this line, `actor` can no longer be used
    actor.create_options(9000, false);
    
    // The following line would cause a compile error because `actor` has been moved:
    // println!("Actor ID after move: {}", actor.id); // ❌ Compile error!
    
    println!("Server started! Actor has been moved and can no longer be used.");
    println!("This prevents accidental use of the actor after starting the server.");
    println!();
    println!("Test the server:");
    println!("  curl -X POST http://127.0.0.1:9000/get_id -d '{{}}'");
    println!("  curl -X POST http://127.0.0.1:9000/greet -d '{{\"name\": \"World\"}}'");
    println!();
    println!("Press Ctrl+C to stop");
    
    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
