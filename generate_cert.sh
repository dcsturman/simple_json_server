#!/bin/bash

# Generate self-signed certificate for TLS testing
# This script creates cert.pem and key.pem files for development/testing purposes

echo "Generating self-signed certificate for TLS testing..."
echo

# Generate private key and certificate in one command
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
    -subj "/C=US/ST=Test/L=Test/O=Test/OU=Test/CN=localhost"

if [ $? -eq 0 ]; then
    echo
    echo "✅ Certificate files generated successfully!"
    echo "   - cert.pem (certificate)"
    echo "   - key.pem (private key)"
    echo
    echo "You can now run the TLS server example:"
    echo "   cargo run --example tls_server"
    echo
    echo "⚠️  These are self-signed certificates for development only!"
    echo "   Use -k flag with curl to accept self-signed certificates"
    echo "   Example: curl -k -X POST https://127.0.0.1:8443/add -d '{\"a\": 10, \"b\": 5}'"
else
    echo
    echo "❌ Failed to generate certificate files"
    echo "   Make sure OpenSSL is installed on your system"
fi
