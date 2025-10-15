use simple_json_server::{Actor, actor};

/// A simple calculator actor that demonstrates the #[actor] macro
#[derive(Debug, Clone)]
pub struct Calculator {
    pub memory: f64,
}

impl Default for Calculator {
    fn default() -> Self {
        Self { memory: 0.0 }
    }
}

#[actor]
impl Calculator {
    /// Add two numbers
    pub async fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    /// Subtract two numbers
    pub async fn subtract(&self, a: f64, b: f64) -> f64 {
        a - b
    }

    /// Multiply two numbers
    pub async fn multiply(&self, a: f64, b: f64) -> f64 {
        a * b
    }

    /// Divide two numbers
    pub async fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    /// Get the current memory value
    pub async fn get_memory(&self) -> f64 {
        self.memory
    }

    /// Clear memory (set to 0)
    pub async fn clear_memory(&self) -> String {
        "Memory cleared".to_string()
    }

    /// Get calculator info
    pub async fn info(&self) -> String {
        "Simple JSON Calculator v1.0".to_string()
    }
}

fn main() {
    let calc = Calculator::default();

    // Example usage
    println!("Calculator Actor Example");
    println!("========================");

    // Test addition
    let add_msg = r#"{"a": 10.5, "b": 5.2}"#;
    let result = calc.dispatch("add", add_msg);
    println!("Add 10.5 + 5.2 = {}", result);

    // Test division
    let div_msg = r#"{"a": 20.0, "b": 4.0}"#;
    let result = calc.dispatch("divide", div_msg);
    println!("Divide 20.0 / 4.0 = {}", result);

    // Test division by zero
    let div_zero_msg = r#"{"a": 10.0, "b": 0.0}"#;
    let result = calc.dispatch("divide", div_zero_msg);
    println!("Divide 10.0 / 0.0 = {}", result);

    // Test method with no parameters
    let info_msg = r#"{}"#;
    let result = calc.dispatch("info", info_msg);
    println!("Info: {}", result);

    // Test unknown method
    let unknown_msg = r#"{}"#;
    let result = calc.dispatch("unknown", unknown_msg);
    println!("Unknown method: {}", result);
}
