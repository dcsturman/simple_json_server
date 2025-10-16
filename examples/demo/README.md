# Simple JSON Server Demo

This demo showcases the `simple_json_server` crate with a JavaScript client that validates the generated JSON-RPC interface.

## Overview

The demo consists of:

1. **Rust Server** (`src/main.rs`) - A simple actor with two methods using the `#[actor]` macro
2. **JavaScript Client** (`client.js`) - Tests both HTTP and WebSocket interfaces
3. **Generated Documentation** - RustDoc showing the exact JSON payloads needed

## Quick Start

### 1. Start the Rust Server

```bash
# From the examples/demo directory
cargo run
```

The server will start on `http://127.0.0.1:9000` and display:
```
Starting server on port 9000...
Server started!

Test the server:
  curl -X POST http://127.0.0.1:9000/get_id -d '{}'
  curl -X POST http://127.0.0.1:9000/greet -d '{"name": "World"}'

Press Ctrl+C to stop
```

### 2. Install JavaScript Dependencies

```bash
npm install
```

### 3. Test the API

**Option A: JavaScript Client**
```bash
node client.js
# or
npm test
```

**Option B: Web Interface**
Open `index.html` in your browser to test the API interactively.

**Option C: Manual Testing**
```bash
curl -X POST http://127.0.0.1:9000/get_id -d '{}'
curl -X POST http://127.0.0.1:9000/greet -d '{"name": "World"}'
```

## What the Demo Shows

### Actor Methods

The `SimpleServerDemo` actor has two methods:

#### `get_id() -> String`
- **HTTP**: `POST /get_id` with body `{}`
- **WebSocket**: `{"method": "get_id", "params": {}}`
- **Returns**: The actor's ID string

#### `greet(name: String) -> String`
- **HTTP**: `POST /greet` with body `{"name": "YourName"}`
- **WebSocket**: `{"method": "greet", "params": {"name": "YourName"}}`
- **Returns**: A greeting message

### JavaScript Client Features

The client demonstrates:

1. **HTTP Interface Testing**
   - Makes POST requests to each endpoint
   - Validates JSON request/response format
   - Tests error handling for invalid methods

2. **WebSocket Interface Testing**
   - Connects via WebSocket
   - Sends JSON-RPC messages
   - Receives and validates responses

3. **API Documentation Validation**
   - Shows the exact JSON payloads needed
   - Validates the interface matches the generated RustDoc

## Example Output

```
📚 SimpleServerDemo API Documentation
=====================================

Available Methods:

1. get_id()
   Description: Get the current ID of this actor instance
   HTTP: POST /get_id
   Params: {}
   Returns: String

2. greet(name: String)
   Description: Greet someone with a personalized message
   HTTP: POST /greet
   Params: {"name": "string"}
   Returns: String

🔍 Checking server health...
✅ Server is running!

🌐 Testing HTTP Methods
======================

📋 Testing get_id method:
   Method: get_id
   Params: {}
   ✅ Result: "demo-actor"

👋 Testing greet method:
   Method: greet
   Params: {"name": "JavaScript Client"}
   ✅ Result: "Hello JavaScript Client, I'm demo-actor"

🔌 Testing WebSocket Methods
============================

📋 Testing get_id via WebSocket:
   Message: {"method": "get_id", "params": {}}
   ✅ Result: "demo-actor"

👋 Testing greet via WebSocket:
   Message: {"method": "greet", "params": {"name": "WebSocket Client"}}
   ✅ Result: "Hello WebSocket Client, I'm demo-actor"

🎉 All tests completed!
```

## Manual Testing

You can also test manually with curl:

```bash
# Test get_id
curl -X POST http://127.0.0.1:9000/get_id -d '{}'

# Test greet
curl -X POST http://127.0.0.1:9000/greet -d '{"name": "World"}'

# Test invalid method (should return error)
curl -X POST http://127.0.0.1:9000/invalid -d '{}'
```

## Generated Documentation

To see the complete generated RustDoc:

```bash
cargo doc --open
```

This will show the automatically generated documentation for the `SimpleServerDemo` actor, including:

- Method signatures
- Parameter types
- Return types
- Example JSON payloads
- HTTP endpoint mappings

## Key Features Demonstrated

1. **Move Semantics**: The actor is consumed when starting the server
2. **JSON-RPC Interface**: Automatic generation from Rust method signatures
3. **Dual Protocol Support**: Both HTTP and WebSocket interfaces
4. **Type Safety**: Rust types are automatically serialized/deserialized
5. **Documentation Generation**: Complete API docs generated from code
6. **Cross-Language Compatibility**: JavaScript can easily call Rust methods

This demo validates that the `#[actor]` macro creates a robust, well-documented JSON-RPC interface that works seamlessly with JavaScript clients.
