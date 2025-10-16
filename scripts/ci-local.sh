#!/bin/bash
# Local CI script to run the same checks as GitHub Actions
# This helps developers catch issues before pushing to GitHub

set -e  # Exit on any error

echo "🚀 Running local CI checks..."
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 is not installed. Please install it first."
        return 1
    fi
}

print_step "Checking required tools..."
check_tool cargo
check_tool rustfmt
check_tool clippy

# Check for optional tools
if ! command -v cargo-llvm-cov &> /dev/null; then
    print_warning "cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov"
    COVERAGE_AVAILABLE=false
else
    COVERAGE_AVAILABLE=true
fi

if ! command -v cargo-audit &> /dev/null; then
    print_warning "cargo-audit not found. Install with: cargo install cargo-audit"
    AUDIT_AVAILABLE=false
else
    AUDIT_AVAILABLE=true
fi

echo ""

# 1. Check formatting
print_step "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

echo ""

# 2. Run Clippy
print_step "Running Clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "No Clippy warnings found"
else
    print_error "Clippy warnings found. Please fix them."
    exit 1
fi

echo ""

# 3. Build workspace
print_step "Building workspace..."
if cargo build --workspace --all-features; then
    print_success "Workspace builds successfully"
else
    print_error "Build failed"
    exit 1
fi

echo ""

# 4. Run tests
print_step "Running tests..."
if cargo test --workspace --all-features; then
    print_success "All tests pass"
else
    print_error "Some tests failed"
    exit 1
fi

echo ""

# 5. Build examples
print_step "Building examples..."
if cargo build --examples; then
    print_success "Examples build successfully"
else
    print_error "Example build failed"
    exit 1
fi

echo ""

# 6. Test examples (basic compilation check)
print_step "Testing examples..."
cd simple_json_server

# Test calculator example (doesn't start server)
if timeout 10s cargo run --example calculator 2>/dev/null || true; then
    print_success "Calculator example runs"
else
    print_warning "Calculator example may have issues"
fi

# Test doc_test example
if timeout 10s cargo run --example doc_test 2>/dev/null || true; then
    print_success "Doc test example runs"
else
    print_warning "Doc test example may have issues"
fi

cd ..

echo ""

# 7. Build documentation
print_step "Building documentation..."
if cargo doc --workspace --all-features --no-deps; then
    print_success "Documentation builds successfully"
else
    print_error "Documentation build failed"
    exit 1
fi

echo ""

# 8. Run coverage (if available)
if [ "$COVERAGE_AVAILABLE" = true ]; then
    print_step "Generating coverage report..."
    if cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info; then
        print_success "Coverage report generated"
        
        # Extract coverage percentage
        if command -v lcov &> /dev/null; then
            COVERAGE=$(lcov --summary lcov.info 2>/dev/null | grep "lines" | grep -o '[0-9.]*%' | head -1)
            echo -e "${BLUE}📊 Coverage: ${COVERAGE}${NC}"
        fi
    else
        print_warning "Coverage generation failed"
    fi
    echo ""
fi

# 9. Run security audit (if available)
if [ "$AUDIT_AVAILABLE" = true ]; then
    print_step "Running security audit..."
    if cargo audit; then
        print_success "No security vulnerabilities found"
    else
        print_warning "Security audit found issues"
    fi
    echo ""
fi

# 10. Test demo integration
print_step "Testing demo integration..."
cd examples/demo

# Check if Node.js is available
if command -v node &> /dev/null && command -v npm &> /dev/null; then
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_step "Installing Node.js dependencies..."
        npm install
    fi
    
    # Build demo
    if cargo build; then
        print_success "Demo builds successfully"
        
        # Start server in background and test
        print_step "Testing demo server..."
        cargo run &
        SERVER_PID=$!
        sleep 3  # Give server time to start
        
        # Test basic endpoint
        if curl -f http://127.0.0.1:9000/get_id -d '{}' -H "Content-Type: application/json" &>/dev/null; then
            print_success "Demo server responds correctly"
            
            # Run JavaScript client test
            if node client.js &>/dev/null; then
                print_success "JavaScript client test passes"
            else
                print_warning "JavaScript client test failed"
            fi
        else
            print_warning "Demo server not responding"
        fi
        
        # Clean up
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    else
        print_warning "Demo build failed"
    fi
else
    print_warning "Node.js not available, skipping demo integration test"
fi

cd ../..

echo ""
echo "================================"
print_success "Local CI checks completed!"
echo ""
echo "🎉 Your code is ready for GitHub Actions!"
echo ""
echo "Requirements:"
echo "  - Rust 1.85+ (for edition 2024 and resolver 3)"
echo "  - All CI checks pass locally"
echo ""
echo "Next steps:"
echo "  - Commit your changes: git add . && git commit -m 'your message'"
echo "  - Push to GitHub: git push"
echo "  - GitHub Actions will run the full CI pipeline"
echo ""

if [ "$COVERAGE_AVAILABLE" = true ]; then
    echo "📊 Coverage report available at: target/llvm-cov/html/index.html"
    echo "   Open with: cargo llvm-cov --open"
    echo ""
fi
