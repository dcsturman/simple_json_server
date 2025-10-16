---
name: Feature request
about: Suggest an idea for this project
title: '[FEATURE] '
labels: 'enhancement'
assignees: ''

---

**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Use Case**
Describe the specific use case or scenario where this feature would be helpful.

**Example Usage**
If possible, provide an example of how you would like to use this feature:

```rust
// Example of how the feature might work
use simple_json_server::{Actor, actor};

#[derive(Debug, Clone)]
struct MyActor;

#[actor]
impl MyActor {
    // Your proposed feature usage here
    pub async fn new_feature(&self) -> String {
        "example".to_string()
    }
}
```

**Impact**
- [ ] This would improve performance
- [ ] This would improve developer experience
- [ ] This would add new functionality
- [ ] This would improve security
- [ ] This would improve documentation
- [ ] Other: [please specify]

**Breaking Changes**
- [ ] This feature would require breaking changes
- [ ] This feature is backward compatible
- [ ] Unsure about compatibility impact

**Additional context**
Add any other context, screenshots, or examples about the feature request here.

**Checklist**
- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have provided a clear use case for this feature
- [ ] I have considered the impact on existing users
- [ ] I have provided example usage if applicable
