# Code Coverage Strategy

## Current Coverage: ~80%

### Coverage Philosophy

This project maintains **high coverage for testable code** while acknowledging that some error paths cannot be reliably tested in a CI environment.

### What's Covered ✅

- **Business Logic**: 95%+ coverage
  - All actor methods (`add`, `greet`, `info`, etc.)
  - JSON serialization/deserialization
  - Method dispatch and routing
  - Parameter validation

- **Integration Paths**: 90%+ coverage
  - HTTP server functionality
  - WebSocket server functionality  
  - TLS/HTTPS support
  - WSS (WebSocket Secure) support
  - End-to-end client-server communication

- **Error Handling**: Testable errors covered
  - Invalid JSON parsing
  - Unknown method calls
  - Parameter validation failures
  - Malformed requests

### What's Not Covered (By Design) ❌

- **OS-Level Failures**: Cannot be reliably simulated
  - Socket binding failures
  - Network interface errors
  - TLS handshake failures
  - File system errors

- **Resource Exhaustion**: Not practical to test
  - Out of memory conditions
  - File descriptor limits
  - Network timeouts

- **Concurrent Edge Cases**: Extremely rare
  - Race conditions in WebSocket cleanup
  - Simultaneous connection drops

### Coverage Commands

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace

# Generate HTML report
cargo llvm-cov --all-features --workspace --html

# Open HTML report
cargo llvm-cov --all-features --workspace --open
```

### Coverage Targets

- **Minimum Acceptable**: 75%
- **Current Target**: 80%
- **Ideal Target**: 85%

Higher coverage would require testing OS-level failures, which adds complexity without meaningful value.

### Philosophy

> "Perfect is the enemy of good. We focus on testing what matters: the code that users interact with and the business logic that drives value."

**80% coverage with comprehensive integration tests** is better than **95% coverage with only unit tests**.
