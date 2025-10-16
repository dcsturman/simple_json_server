# CI/CD Setup Documentation

This document describes the comprehensive CI/CD setup for the simple_json_server project.

## 🔄 GitHub Actions Workflows

### 1. Main CI Workflow (`.github/workflows/ci.yml`)

**Triggers**: Push to `main`/`develop`, Pull Requests
**Purpose**: Comprehensive testing and quality assurance

#### Jobs:

##### **Test Suite** (`test`)
- **Multi-Rust Testing**: Tests on stable, beta, and nightly Rust versions
- **Code Quality**: Runs `rustfmt` and `clippy` with strict settings
- **Build Verification**: Builds entire workspace with all features
- **Test Execution**: Runs all unit and integration tests
- **Example Validation**: Ensures examples compile and basic functionality

##### **Code Coverage** (`coverage`)
- **Coverage Generation**: Uses `cargo llvm-cov` for accurate coverage
- **Codecov Integration**: Uploads coverage reports to Codecov
- **Artifact Storage**: Stores HTML coverage reports
- **Coverage Tracking**: Monitors coverage trends over time

##### **Security Audit** (`security`)
- **Vulnerability Scanning**: Uses `cargo audit` for known vulnerabilities
- **Dependency Checking**: Validates dependency security status
- **Regular Updates**: Runs on every commit for immediate feedback

##### **Documentation** (`docs`)
- **Doc Building**: Validates all documentation builds correctly
- **API Docs**: Generates comprehensive API documentation
- **Artifact Upload**: Stores documentation for review

##### **Integration Testing** (`integration`)
- **JavaScript Integration**: Tests Rust-JavaScript interoperability
- **Server Lifecycle**: Tests complete server startup/shutdown cycle
- **Cross-Language Validation**: Ensures JSON-RPC interface works correctly
- **Real-World Testing**: Uses actual HTTP requests and WebSocket connections

##### **MSRV Testing** (`minimum-rust-version`)
- **Compatibility**: Tests with Rust 1.75 (minimum supported version)
- **Future-Proofing**: Ensures backward compatibility
- **Version Validation**: Confirms compilation with older Rust versions

### 2. Release Workflow (`.github/workflows/release.yml`)

**Triggers**: Git tags matching `v*` pattern
**Purpose**: Automated releases and publishing

#### Jobs:

##### **Create Release** (`create-release`)
- **GitHub Release**: Creates GitHub release from tag
- **Release Notes**: Generates release notes from commits
- **Asset Coordination**: Provides upload URL for other jobs

##### **Publish Crates** (`publish-crates`)
- **crates.io Publishing**: Publishes to Rust package registry
- **Dependency Order**: Publishes `actor_attribute_macro` first, then `simple_json_server`
- **Error Handling**: Continues on error (handles already-published crates)

##### **Build Binaries** (`build-binaries`)
- **Multi-Platform**: Builds for Linux, Windows, macOS (x86_64 + ARM64)
- **Release Assets**: Attaches binaries to GitHub release
- **Cross-Compilation**: Uses Rust's cross-compilation capabilities

##### **Generate Documentation** (`generate-docs`)
- **Release Docs**: Builds documentation for the release
- **Documentation Archive**: Creates downloadable documentation package
- **Release Assets**: Attaches docs to GitHub release

### 3. Dependencies Workflow (`.github/workflows/dependencies.yml`)

**Triggers**: Weekly schedule (Sundays 2 AM UTC), Manual dispatch
**Purpose**: Automated dependency management and security

#### Jobs:

##### **Update Dependencies** (`update-dependencies`)
- **Automated Updates**: Updates dependencies to latest compatible versions
- **Testing**: Runs full test suite with updated dependencies
- **Pull Request Creation**: Creates PR with dependency updates
- **Change Documentation**: Includes summary of changes

##### **Security Audit** (`security-audit`)
- **Regular Scanning**: Weekly security vulnerability checks
- **Multiple Tools**: Uses both `cargo audit` and `cargo deny`
- **Immediate Alerts**: Fails on security issues

##### **License Check** (`license-check`)
- **License Compliance**: Validates all dependency licenses
- **Report Generation**: Creates comprehensive license report
- **Compliance Tracking**: Monitors license changes over time

## 🛠️ Configuration Files

### `deny.toml`
- **Dependency Policy**: Defines allowed/denied dependencies
- **License Policy**: Specifies acceptable licenses
- **Security Policy**: Configures vulnerability handling
- **Multi-Platform**: Supports multiple target platforms

### Issue Templates
- **Bug Reports**: Structured bug reporting with environment details
- **Feature Requests**: Standardized feature request format
- **Pull Request Template**: Comprehensive PR checklist

### Local Development Script (`scripts/ci-local.sh`)
- **Pre-Push Validation**: Runs same checks as CI locally
- **Developer Experience**: Catches issues before GitHub Actions
- **Comprehensive Testing**: Includes all CI checks
- **Colored Output**: User-friendly terminal output

## 📊 Quality Metrics

### Code Coverage
- **Target**: 80%+ coverage on testable code
- **Tools**: `cargo llvm-cov` for accurate Rust coverage
- **Reporting**: Codecov integration with trend tracking
- **Artifacts**: HTML reports for detailed analysis

### Security
- **Vulnerability Scanning**: `cargo audit` for known issues
- **Dependency Validation**: `cargo deny` for policy enforcement
- **Regular Updates**: Weekly automated dependency updates
- **License Compliance**: Automated license checking

### Performance
- **Build Times**: Optimized with comprehensive caching
- **Test Execution**: Parallel test execution where possible
- **Artifact Management**: Efficient artifact storage and retrieval

## 🚀 Developer Workflow

### Local Development
1. **Clone Repository**: `git clone ...`
2. **Install Tools**: `rustup component add rustfmt clippy`
3. **Run Local CI**: `./scripts/ci-local.sh`
4. **Make Changes**: Develop features/fixes
5. **Pre-Push Check**: `./scripts/ci-local.sh`
6. **Push Changes**: GitHub Actions takes over

### Pull Request Process
1. **Create Feature Branch**: `git checkout -b feature/name`
2. **Develop and Test**: Use local CI script
3. **Create Pull Request**: Use PR template
4. **CI Validation**: All workflows must pass
5. **Code Review**: Maintainer review
6. **Merge**: Automated merge after approval

### Release Process
1. **Version Bump**: Update version in `Cargo.toml` files
2. **Create Tag**: `git tag v1.x.x`
3. **Push Tag**: `git push origin v1.x.x`
4. **Automated Release**: GitHub Actions handles everything
5. **Verification**: Check crates.io and GitHub releases

## 🔧 Maintenance

### Regular Tasks
- **Weekly**: Dependency updates (automated)
- **Monthly**: Review security audit results
- **Quarterly**: Update CI tool versions
- **As Needed**: Update MSRV when dropping old Rust support

### Monitoring
- **Build Status**: GitHub Actions status badges
- **Coverage Trends**: Codecov dashboard
- **Security Alerts**: GitHub security advisories
- **Performance**: Build time and test execution monitoring

## 📚 Resources

### Documentation
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)

### Best Practices
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Security Best Practices](https://rustsec.org/advisories/)
- [Semantic Versioning](https://semver.org/)

This CI/CD setup ensures high code quality, security, and reliability while maintaining developer productivity and project maintainability.
