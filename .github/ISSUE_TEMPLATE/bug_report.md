---
name: Bug report
about: Create a report to help us improve
title: '[BUG] '
labels: 'bug'
assignees: ''

---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create an actor with '...'
2. Call method '....'
3. Send JSON payload '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Actual behavior**
What actually happened instead.

**Code Example**
```rust
// Minimal code example that reproduces the issue
use simple_json_server::{Actor, actor};

#[derive(Debug, Clone)]
struct MyActor;

#[actor]
impl MyActor {
    pub async fn my_method(&self) -> String {
        "test".to_string()
    }
}
```

**JSON Payload (if applicable)**
```json
{
  "method": "my_method",
  "params": {}
}
```

**Error Output**
```
Paste the full error message here
```

**Environment:**
 - OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
 - Rust version: [e.g. 1.75.0]
 - simple_json_server version: [e.g. 1.0.0]
 - Cargo version: [e.g. 1.75.0]

**Additional context**
Add any other context about the problem here.

**Checklist**
- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have provided a minimal code example that reproduces the issue
- [ ] I have included the full error output
- [ ] I have specified my environment details
