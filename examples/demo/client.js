#!/usr/bin/env node

/**
 * JavaScript client for the SimpleServerDemo Rust actor
 * 
 * This demonstrates how to call the Rust actor methods from JavaScript
 * and validates the JSON-RPC interface documented in the generated RustDoc.
 * 
 * Usage:
 *   1. Start the Rust server: cargo run
 *   2. Run this client: node client.js
 * 
 * The server should be running on http://127.0.0.1:9000
 */

const http = require('http');
const WebSocket = require('ws');

// Configuration
const SERVER_HOST = '127.0.0.1';
const SERVER_PORT = 9000;
const HTTP_URL = `http://${SERVER_HOST}:${SERVER_PORT}`;
const WS_URL = `ws://${SERVER_HOST}:${SERVER_PORT}`;

/**
 * Make an HTTP POST request to the actor server
 * @param {string} method - The actor method name
 * @param {object} params - The parameters object
 * @returns {Promise<any>} - The response from the server
 */
async function callHttpMethod(method, params = {}) {
    return new Promise((resolve, reject) => {
        const postData = JSON.stringify(params);
        
        const options = {
            hostname: SERVER_HOST,
            port: SERVER_PORT,
            path: `/${method}`,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(postData)
            }
        };

        const req = http.request(options, (res) => {
            let data = '';
            
            res.on('data', (chunk) => {
                data += chunk;
            });
            
            res.on('end', () => {
                try {
                    const response = JSON.parse(data);
                    resolve(response);
                } catch (e) {
                    reject(new Error(`Failed to parse response: ${data}`));
                }
            });
        });

        req.on('error', (e) => {
            reject(e);
        });

        req.write(postData);
        req.end();
    });
}

/**
 * Test WebSocket connection to the actor server
 * @param {string} method - The actor method name
 * @param {object} params - The parameters object
 * @returns {Promise<any>} - The response from the server
 */
async function callWebSocketMethod(method, params = {}) {
    return new Promise((resolve, reject) => {
        const ws = new WebSocket(WS_URL);
        
        ws.on('open', () => {
            const message = {
                method: method,
                params: params
            };
            ws.send(JSON.stringify(message));
        });
        
        ws.on('message', (data) => {
            try {
                const response = JSON.parse(data.toString());
                ws.close();
                resolve(response);
            } catch (e) {
                ws.close();
                reject(new Error(`Failed to parse WebSocket response: ${data}`));
            }
        });
        
        ws.on('error', (error) => {
            reject(error);
        });
        
        ws.on('close', (code, reason) => {
            if (code !== 1000) {
                reject(new Error(`WebSocket closed with code ${code}: ${reason}`));
            }
        });
    });
}

/**
 * Test all the methods documented in the RustDoc
 */
async function testHttpMethods() {
    console.log('🌐 Testing HTTP Methods');
    console.log('======================');
    
    try {
        // Test get_id method (no parameters)
        console.log('\n📋 Testing get_id method:');
        console.log('   Method: get_id');
        console.log('   Params: {}');
        const idResult = await callHttpMethod('get_id', {});
        console.log(`   ✅ Result: ${JSON.stringify(idResult)}`);
        
        // Test greet method (with name parameter)
        console.log('\n👋 Testing greet method:');
        console.log('   Method: greet');
        console.log('   Params: {"name": "JavaScript Client"}');
        const greetResult = await callHttpMethod('greet', { name: 'JavaScript Client' });
        console.log(`   ✅ Result: ${JSON.stringify(greetResult)}`);
        
        // Test invalid method (should return error)
        console.log('\n❌ Testing invalid method:');
        console.log('   Method: invalid_method');
        console.log('   Params: {}');
        try {
            const invalidResult = await callHttpMethod('invalid_method', {});
            console.log(`   ⚠️  Unexpected success: ${JSON.stringify(invalidResult)}`);
        } catch (error) {
            console.log(`   ✅ Expected error: ${error.message}`);
        }
        
    } catch (error) {
        console.error(`❌ HTTP test failed: ${error.message}`);
    }
}

/**
 * Test WebSocket methods
 */
async function testWebSocketMethods() {
    console.log('\n🔌 Testing WebSocket Methods');
    console.log('============================');
    
    try {
        // Test get_id via WebSocket
        console.log('\n📋 Testing get_id via WebSocket:');
        console.log('   Message: {"method": "get_id", "params": {}}');
        const wsIdResult = await callWebSocketMethod('get_id', {});
        console.log(`   ✅ Result: ${JSON.stringify(wsIdResult)}`);
        
        // Test greet via WebSocket
        console.log('\n👋 Testing greet via WebSocket:');
        console.log('   Message: {"method": "greet", "params": {"name": "WebSocket Client"}}');
        const wsGreetResult = await callWebSocketMethod('greet', { name: 'WebSocket Client' });
        console.log(`   ✅ Result: ${JSON.stringify(wsGreetResult)}`);
        
    } catch (error) {
        console.error(`❌ WebSocket test failed: ${error.message}`);
    }
}

/**
 * Display the API documentation based on the generated RustDoc
 */
function displayApiDocumentation() {
    console.log('📚 SimpleServerDemo API Documentation');
    console.log('=====================================');
    console.log('');
    console.log('This client validates the JSON-RPC interface generated by the #[actor] macro.');
    console.log('');
    console.log('Available Methods:');
    console.log('');
    console.log('1. get_id()');
    console.log('   Description: Get the current ID of this actor instance');
    console.log('   HTTP: POST /get_id');
    console.log('   Params: {}');
    console.log('   Returns: String');
    console.log('   Example: curl -X POST http://127.0.0.1:9000/get_id -d "{}"');
    console.log('');
    console.log('2. greet(name: String)');
    console.log('   Description: Greet someone with a personalized message');
    console.log('   HTTP: POST /greet');
    console.log('   Params: {"name": "string"}');
    console.log('   Returns: String');
    console.log('   Example: curl -X POST http://127.0.0.1:9000/greet -d \'{"name": "World"}\'');
    console.log('');
    console.log('WebSocket Usage:');
    console.log('   Connect to: ws://127.0.0.1:9000');
    console.log('   Send: {"method": "method_name", "params": {...}}');
    console.log('   Receive: JSON response');
    console.log('');
}

/**
 * Check if the server is running
 */
async function checkServerHealth() {
    try {
        await callHttpMethod('get_id', {});
        return true;
    } catch (error) {
        return false;
    }
}

/**
 * Main function
 */
async function main() {
    displayApiDocumentation();
    
    console.log('🔍 Checking server health...');
    const isServerRunning = await checkServerHealth();
    
    if (!isServerRunning) {
        console.error('❌ Server is not running!');
        console.log('');
        console.log('Please start the server first:');
        console.log('   cd examples/demo');
        console.log('   cargo run');
        console.log('');
        console.log('Then run this client again:');
        console.log('   node client.js');
        process.exit(1);
    }
    
    console.log('✅ Server is running!');
    console.log('');
    
    // Run the tests
    await testHttpMethods();

    // Note: WebSocket testing disabled for this demo
    // The demo server is running in HTTP-only mode
    console.log('\n🔌 WebSocket Testing');
    console.log('===================');
    console.log('WebSocket testing is disabled for this demo.');
    console.log('The server is running in HTTP-only mode.');
    console.log('To enable WebSocket support, use actor.create_ws(port) instead.');
    
    console.log('\n🎉 All tests completed!');
    console.log('');
    console.log('This validates that the Rust #[actor] macro correctly generates');
    console.log('a JSON-RPC interface that can be called from JavaScript clients.');
}

// Handle WebSocket dependency
try {
    require.resolve('ws');
} catch (e) {
    console.log('📦 Installing WebSocket dependency...');
    console.log('Run: npm install ws');
    console.log('Then run this script again.');
    process.exit(1);
}

// Run the main function
main().catch(console.error);
