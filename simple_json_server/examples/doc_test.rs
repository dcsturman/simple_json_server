use simple_json_server::{Actor, actor};

/// A sample actor to demonstrate the generated documentation
#[derive(Debug, Clone)]
pub struct DocumentedActor {
    pub name: String,
    pub count: i32,
}

impl DocumentedActor {
    pub fn new(name: String) -> Self {
        Self { name, count: 0 }
    }
}

#[actor]
impl DocumentedActor {
    /// Add two numbers together
    /// This method performs basic arithmetic addition
    pub async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Get the current count value
    pub async fn get_count(&self) -> i32 {
        self.count
    }

    /// Greet someone with a personalized message
    /// Returns a friendly greeting including the actor's name
    pub async fn greet(&self, name: String) -> String {
        format!("Hello {}, I'm {}!", name, self.name)
    }

    /// Calculate the area of a rectangle
    pub async fn calculate_area(&self, width: f64, height: f64) -> f64 {
        width * height
    }

    /// Check if a number is even
    pub async fn is_even(&self, number: i32) -> bool {
        number % 2 == 0
    }

    /// Get basic information about this actor
    /// Returns metadata about the actor instance
    pub async fn info(&self) -> String {
        format!(
            "DocumentedActor named '{}' with count {}",
            self.name, self.count
        )
    }

    /// A method with no parameters
    /// This demonstrates how methods without parameters are documented
    pub async fn ping(&self) -> String {
        "pong".to_string()
    }
}

#[tokio::main]
async fn main() {
    let actor = DocumentedActor::new("TestActor".to_string());

    println!("Testing documented actor methods:");

    // Test add method
    let result = actor.dispatch("add", r#"{"a": 10, "b": 5}"#).await;
    println!("add(10, 5) = {}", result);

    // Test greet method
    let result = actor.dispatch("greet", r#"{"name": "World"}"#).await;
    println!("greet(\"World\") = {}", result);

    // Test ping method (no parameters)
    let result = actor.dispatch("ping", "{}").await;
    println!("ping() = {}", result);

    // Test is_even method
    let result = actor.dispatch("is_even", r#"{"number": 42}"#).await;
    println!("is_even(42) = {}", result);

    println!("\nTo see the generated documentation, run:");
    println!("cargo doc --open");
}
