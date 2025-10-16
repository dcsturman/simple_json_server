# Contributing to simple_json_server

Thank you for your interest in contributing to simple_json_server! This document provides guidelines and information for contributors.

## 🚀 Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/simple_json_server.git
   cd simple_json_server
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** and test them
5. **Run local CI checks**:
   ```bash
   ./scripts/ci-local.sh
   ```
6. **Commit and push** your changes
7. **Create a Pull Request** on GitHub

## 🛠️ Development Setup

### Prerequisites
- **Rust** (1.77+ required, latest stable recommended)
- **Node.js** (for JavaScript integration tests)
- **Git**

### Required Tools
Install these tools for development:

```bash
# Code formatting and linting
rustup component add rustfmt clippy

# Coverage reporting
cargo install cargo-llvm-cov

# Security auditing
cargo install cargo-audit

# License checking
cargo install cargo-deny

# Documentation tools
cargo install cargo-doc
```

### Building the Project

```bash
# Build the entire workspace
cargo build --workspace --all-features

# Run all tests
cargo test --workspace --all-features

# Build examples
cargo build --examples

# Generate documentation
cargo doc --workspace --all-features --open
```

## 🧪 Testing

### Running Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --test integration_test

# Coverage report
cargo llvm-cov --all-features --workspace --open
```

### Test Structure
- **Unit tests**: Located in `src/` files using `#[cfg(test)]`
- **Integration tests**: Located in `tests/` directory
- **Example tests**: Located in `examples/` directory
- **JavaScript tests**: Located in `examples/demo/client.js`

### Writing Tests
- Write tests for all new functionality
- Maintain or improve code coverage (target: 80%+)
- Include both positive and negative test cases
- Test error conditions and edge cases

## 📝 Code Style

### Rust Code Style
- Use `cargo fmt` for formatting
- Follow `clippy` recommendations
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Follow Rust naming conventions

### Documentation
- Document all public APIs with `///` comments
- Include examples in documentation
- Update README.md for significant changes
- Keep examples up to date

## 🔍 Code Review Process

### Before Submitting
1. **Run local CI**: `./scripts/ci-local.sh`
2. **Test thoroughly**: Ensure all tests pass
3. **Update documentation**: Keep docs current
4. **Check coverage**: Maintain coverage levels
5. **Review your own code**: Self-review before submission

### Pull Request Guidelines
- **Clear title**: Describe what the PR does
- **Detailed description**: Explain the changes and why
- **Link issues**: Reference related issues
- **Small PRs**: Keep changes focused and reviewable
- **Tests included**: Add tests for new functionality

## 🏗️ Project Structure

```
simple_json_server/
├── actor_attribute_macro/     # Procedural macro crate
│   ├── src/lib.rs            # Macro implementation
│   └── Cargo.toml
├── simple_json_server/       # Main library crate
│   ├── src/
│   │   ├── lib.rs           # Main library code
│   │   ├── tls.rs           # TLS configuration
│   │   └── test_actor.rs    # Test utilities
│   ├── examples/            # Library examples
│   └── tests/               # Integration tests
├── examples/demo/            # Complete demo application
│   ├── src/main.rs          # Demo server
│   ├── client.js            # JavaScript client
│   └── index.html           # Web interface
├── .github/                 # GitHub Actions workflows
├── scripts/                 # Development scripts
└── docs/                    # Additional documentation
```

## 🎯 Contribution Areas

### High Priority
- **Bug fixes**: Address reported issues
- **Performance improvements**: Optimize hot paths
- **Documentation**: Improve clarity and examples
- **Test coverage**: Increase test coverage

### Medium Priority
- **New features**: Add requested functionality
- **Examples**: Create more usage examples
- **Platform support**: Improve cross-platform compatibility

### Low Priority
- **Code cleanup**: Refactoring and cleanup
- **Tooling**: Improve development tools
- **CI/CD**: Enhance automation

## 🐛 Reporting Issues

### Bug Reports
Use the bug report template and include:
- **Clear description** of the issue
- **Reproduction steps** with minimal example
- **Expected vs actual behavior**
- **Environment details** (OS, Rust version, etc.)
- **Error messages** (full output)

### Feature Requests
Use the feature request template and include:
- **Use case description**
- **Proposed solution**
- **Example usage**
- **Impact assessment**

## 🔒 Security

### Reporting Security Issues
- **Do not** create public issues for security vulnerabilities
- **Email** security issues to the maintainers
- **Include** detailed reproduction steps
- **Wait** for confirmation before public disclosure

### Security Best Practices
- Keep dependencies updated
- Follow secure coding practices
- Validate all inputs
- Use safe Rust patterns

## 📋 Release Process

### Version Numbering
We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist
1. Update version numbers in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release tag
5. GitHub Actions handles the rest

## 🤝 Community

### Code of Conduct
- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Maintain a welcoming environment

### Getting Help
- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check the docs first
- **Examples**: Look at existing examples

## 📚 Resources

### Learning Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Serde Guide](https://serde.rs/)

### Project Resources
- [API Documentation](https://docs.rs/simple_json_server)
- [Examples](./examples/)
- [Coverage Reports](./COVERAGE.md)
- [Architecture Overview](./docs/architecture.md)

Thank you for contributing to simple_json_server! 🎉
