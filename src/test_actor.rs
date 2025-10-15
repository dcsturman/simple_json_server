use crate::{Actor, actor};

#[derive(Debug, Clone)]
pub struct TestActor {
    pub counter: i32,
}

impl TestActor {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

#[actor]
impl TestActor {
    pub async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    pub async fn get_counter(&self) -> i32 {
        self.counter
    }

    pub async fn greet(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }

    pub async fn no_params(&self) -> String {
        "No parameters needed".to_string()
    }

    // This should be ignored (not public)
    #[allow(dead_code)]
    async fn private_method(&self) -> String {
        "This should not be accessible".to_string()
    }

    #[allow(dead_code)]
    // This should be ignored (not async)
    pub fn sync_method(&self) -> String {
        "This should not be accessible".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_method() {
        let actor = TestActor::new();
        let message = r#"{"a": 5, "b": 3}"#;
        let result = actor.dispatch("add", message);
        assert_eq!(result, "8");
    }

    #[test]
    fn test_get_counter_method() {
        let actor = TestActor::new();
        let message = r#"{}"#;
        let result = actor.dispatch("get_counter", message);
        assert_eq!(result, "0");
    }

    #[test]
    fn test_greet_method() {
        let actor = TestActor::new();
        let message = r#"{"name": "World"}"#;
        let result = actor.dispatch("greet", message);
        assert_eq!(result, r#""Hello, World!""#);
    }

    #[test]
    fn test_no_params_method() {
        let actor = TestActor::new();
        let message = r#"{}"#;
        let result = actor.dispatch("no_params", message);
        assert_eq!(result, r#""No parameters needed""#);
    }

    #[test]
    fn test_unknown_method() {
        let actor = TestActor::new();
        let message = r#"{}"#;
        let result = actor.dispatch("unknown", message);
        assert!(result.contains("Unknown method"));
    }

    #[test]
    fn test_invalid_json() {
        let actor = TestActor::new();
        let message = r#"{"invalid": json"#;
        let result = actor.dispatch("invalid", message);
        assert!(result.contains("Failed to parse JSON"));
    }
}
