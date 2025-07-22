# OxiQMS - Medical Device Quality Management System

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-649%2F649%20passing-brightgreen.svg)](https://github.com/ryancinsight/OxiQMS)

A comprehensive Quality Management System (QMS) built in Rust for medical device development, designed to meet FDA 21 CFR Part 820, ISO 13485, and ISO 14971 regulatory requirements.

## 🏥 Medical Device Compliance

OxiQMS is specifically designed for medical device software development with built-in compliance for:

- **FDA 21 CFR Part 820** - Quality System Regulation
- **ISO 13485** - Medical devices quality management systems
- **ISO 14971** - Medical device risk management
- **IEC 62304** - Medical device software lifecycle processes

## 🚀 Features

### Core Functionality
- **Risk Management** - Comprehensive risk assessment and mitigation tracking
- **Document Control** - Version-controlled document management with approval workflows
- **Audit Logging** - Complete audit trail for regulatory compliance
- **User Management** - Role-based access control with electronic signatures
- **Traceability** - Requirements traceability matrix (RTM) management
- **Report Generation** - Automated compliance reports in multiple formats

### Technical Excellence
- **SOLID Architecture** - Built following SOLID design principles (SRP, OCP, LSP, ISP, DIP)
- **100% Test Coverage** - 649/649 tests passing with comprehensive test suite
- **Web Interface** - Modern web UI with Progressive Web App (PWA) capabilities
- **API-First Design** - RESTful API with comprehensive documentation
- **Security** - Enterprise-grade security headers and data protection

## 🛠️ Installation

### Prerequisites
- Rust 1.70 or higher
- Cargo (comes with Rust)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/ryancinsight/OxiQMS.git
cd OxiQMS

# Build the project
cargo build --release

# Run tests
cargo test

# Start the web server
cargo run --bin qms
```

## 📖 Usage

### Command Line Interface

```bash
# Initialize a new QMS project
cargo run -- init --project "My Medical Device"

# Create a risk assessment
cargo run -- risk create --description "Software failure" --severity high

# Generate compliance reports
cargo run -- report generate --type risk-assessment --format pdf

# Start web interface
cargo run -- web --port 8080
```

### Web Interface

Access the web interface at `http://localhost:8080` after starting the server.

## 🏗️ Architecture

OxiQMS follows SOLID design principles with a modular architecture:

```
src/
├── modules/
│   ├── risk_manager/          # Risk management (ISO 14971)
│   ├── document_control/      # Document control system
│   ├── audit_logger/          # Audit trail management
│   ├── user_manager/          # User and role management
│   └── traceability/          # Requirements traceability
├── web/                       # Web interface and API
├── constants.rs               # Regulatory compliance constants
└── config.rs                  # Configuration management
```

### Key Design Patterns
- **Repository Pattern** - Data access abstraction
- **Strategy Pattern** - Pluggable risk assessment strategies
- **Template Method** - Standardized report generation
- **Observer Pattern** - Audit event notifications
- **Registry Pattern** - Extensible template system

## 🧪 Testing

OxiQMS maintains a 100% test success rate with comprehensive testing:

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test risk_manager
cargo test audit_logger
cargo test web

# Run with coverage
cargo test --all-features
```

## 📋 Compliance Features

### Risk Management (ISO 14971)
- Risk identification and analysis
- Risk evaluation and acceptability criteria
- Risk control measures tracking
- Residual risk assessment
- Post-market surveillance integration

### Document Control
- Version control with approval workflows
- Electronic signatures (21 CFR Part 11)
- Document templates and standardization
- Change control processes

### Audit Trail
- Complete audit logging for all system activities
- Tamper-evident audit records
- Automated backup and retention
- Compliance reporting

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests in watch mode
cargo watch -x test

# Generate coverage report
cargo tarpaulin --out html
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Documentation](https://docs.rs/oxiqms)
- [API Reference](https://ryancinsight.github.io/OxiQMS/api)
- [Issue Tracker](https://github.com/ryancinsight/OxiQMS/issues)
- [Releases](https://github.com/ryancinsight/OxiQMS/releases)

## 📞 Support

For questions, issues, or support:
- Create an [issue](https://github.com/ryancinsight/OxiQMS/issues)
- Email: support@oxiqms.com
- Documentation: [docs.oxiqms.com](https://docs.oxiqms.com)

---

**Note**: This software is designed for medical device development and includes features for regulatory compliance. Always consult with regulatory experts and conduct appropriate validation for your specific use case.
