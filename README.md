# OxiQMS - Medical Device Quality Management System

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Implementation](https://img.shields.io/badge/status-Core%20Modules%20In%20Progress-yellow.svg)](https://github.com/ryancinsight/OxiQMS)

A comprehensive Quality Management System (QMS) built in Rust for medical device development, designed to meet FDA 21 CFR Part 820, ISO 13485, and ISO 14971 regulatory requirements.

## 🏥 Medical Device Compliance

OxiQMS is specifically designed for medical device software development with built-in compliance for:

- **FDA 21 CFR Part 820** - Quality System Regulation
- **ISO 13485** - Medical devices quality management systems
- **ISO 14971** - Medical device risk management
- **IEC 62304** - Medical device software lifecycle processes
- **21 CFR Part 11** - Electronic records and electronic signatures

## 🚀 Features

### ✅ Implemented Features
- **✅ FDA-Compliant Audit Logging** - Comprehensive tracing system with file rotation and JSON structured logging
- **✅ Document Control Foundation** - Version-controlled document management with approval workflows
- **✅ Configuration Management** - LoggingConfig with FDA-compliant presets and validation
- **✅ SOLID Architecture** - Clean separation of concerns with dependency injection
- **✅ Core Module Structure** - Document control, user management, risk management, traceability modules

### 🔄 In Progress
- **🔄 User Management System** - Role-based access control with electronic signatures
- **🔄 Document Control Testing** - Comprehensive test suite implementation
- **🔄 Unified Interface System** - Consistent functionality across CLI, Web, and TUI interfaces

### 📋 Planned Features
- **📋 Risk Management** - Comprehensive risk assessment and mitigation tracking per ISO 14971
- **📋 Requirements Traceability** - Requirements traceability matrix (RTM) management
- **📋 Report Generation** - Automated compliance reports in multiple formats
- **📋 TUI Interface** - Terminal User Interface for command-line environments

### Technical Excellence
- **SOLID Architecture** - Built following SOLID design principles (SRP, OCP, LSP, ISP, DIP)
- **TDD Approach** - Test-driven development with FIRST principles
- **FDA-Compliant Logging** - Structured JSON logging with file rotation and audit trails
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

# Run tests (note: some modules still in development)
cargo test --lib

# Initialize a new QMS project (creates project data directories)
cargo run -- init --project "My Medical Device"

# Start the web server
cargo run --bin qms
```

### First-Time Setup

After cloning the repository, you'll need to set up your project configuration:

1. **Copy the example configuration:**
   ```bash
   cp config/config.example.json config/config.json
   ```

2. **Edit the configuration** to match your project requirements

3. **Initialize your first project:**
   ```bash
   cargo run -- init --project "Your Project Name"
   ```

The application will create the necessary data directories and project structure automatically.

## 📖 Usage

### Command Line Interface

```bash
# Initialize a new QMS project
cargo run -- init --project "My Medical Device"

# Create a risk assessment (when implemented)
cargo run -- risk create --description "Software failure" --severity high

# Generate compliance reports (when implemented)
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
├── interfaces/                # Unified interface system (CLI/Web/TUI)
│   ├── adapters/             # Interface-specific adapters
│   ├── routing.rs            # Unified command routing
│   ├── state.rs              # Unified state management
│   └── authentication.rs     # Unified authentication
├── modules/
│   ├── audit_logger/         # ✅ FDA-compliant audit logging
│   ├── document_control/     # ✅ Document control system
│   ├── user_manager/         # 🔄 User and role management
│   ├── risk_manager/         # 📋 Risk management (ISO 14971)
│   ├── traceability/         # 📋 Requirements traceability
│   └── cupid/                # CUPID architecture patterns
├── tui/                      # 📋 Terminal User Interface
├── web/                      # 🔄 Web interface and API
├── config.rs                 # ✅ Configuration management
├── audit.rs                  # ✅ FDA-compliant logging system
└── constants.rs              # Regulatory compliance constants
```

### Repository Structure

This repository contains only the source code and templates. User data and project files are created at runtime:

```
OxiQMS/
├── src/                       # Source code (tracked in Git)
├── templates/                 # Document templates (tracked in Git)
├── config/                    # Configuration directory
│   ├── config.example.json    # Example configuration (tracked)
│   └── config.json           # Your config (ignored by Git)
├── documents/                 # User documents (ignored by Git)
├── trace/                     # Traceability data (ignored by Git)
├── [uuid-directories]/        # Project data (ignored by Git)
└── tests/                     # Test code (tracked in Git)
```

**Note**: The repository excludes user-generated content, project data, and runtime files. These are created automatically when you initialize and use the QMS system.

### Key Design Patterns
- **Repository Pattern** - Data access abstraction
- **Strategy Pattern** - Pluggable risk assessment strategies
- **Template Method** - Standardized report generation
- **Observer Pattern** - Audit event notifications
- **Registry Pattern** - Extensible template system

## 📊 Current Development Status

### Implementation Progress
- ✅ **FDA-Compliant Audit Logging**: Comprehensive tracing system with file rotation and JSON structured logging
- ✅ **Core Module Architecture**: Document control, user management, risk management, traceability modules
- ✅ **Configuration Management**: LoggingConfig with FDA-compliant presets and validation
- ✅ **SOLID Architecture**: Clean separation of concerns with dependency injection
- 🔄 **User Management System**: Authentication and authorization implementation in progress
- 🔄 **Document Control Testing**: Comprehensive test suite implementation in progress
- 📋 **Risk Management Module**: ISO 14971 compliance features planned
- 📋 **TUI Framework**: Terminal interface implementation planned

### Current Development Phase: Core Module Implementation
**Priority**: Critical  
**Timeline**: 2-3 weeks  
**Status**: IN PROGRESS

#### Next Immediate Tasks:
1. **User Management System** - Complete authentication and authorization with comprehensive tests
2. **Document Control Testing** - Implement full TDD test suite for document control module
3. **Risk Management Implementation** - Basic ISO 14971 compliance features
4. **Integration Testing** - End-to-end workflow validation

### Test-Driven Development
Following TDD principles with FIRST criteria:
- **Fast**: Tests execute in <10 seconds
- **Isolated**: No dependencies between tests
- **Repeatable**: Consistent results across environments
- **Self-validating**: Clear pass/fail outcomes
- **Timely**: Written before/during implementation

## 🧪 Testing

Run the test suite (note: some modules still in development):

```bash
# Run library tests
cargo test --lib

# Run specific test modules
cargo test audit
cargo test config
cargo test document_control

# Run with coverage (when available)
cargo test --all-features
```

## 📋 Compliance Features

### ✅ Implemented Compliance Features

#### FDA-Compliant Audit Logging
- Comprehensive tracing system with structured JSON logging
- Automatic file rotation with configurable retention policies
- Non-blocking I/O for high-performance audit trails
- Tamper-evident audit records with timestamps and thread information

#### Document Control Foundation
- Version control with approval workflows
- Document templates and standardization
- Change control processes
- Electronic signature integration points

### 🔄 In Progress Compliance Features

#### User Management (21 CFR Part 11)
- Role-based access control implementation
- Electronic signature capabilities
- Session management with timeout
- Multi-factor authentication support

### 📋 Planned Compliance Features

#### Risk Management (ISO 14971)
- Risk identification and analysis
- Risk evaluation and acceptability criteria
- Risk control measures tracking
- Residual risk assessment
- Post-market surveillance integration

#### Requirements Traceability
- Bidirectional traceability links
- Impact analysis for requirement changes
- Traceability report generation
- Integration with test management

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests in watch mode
cargo watch -x "test --lib"

# Generate coverage report (when available)
cargo tarpaulin --out html
```

### Development Workflow
1. **Write Tests First**: Follow TDD principles with comprehensive test coverage
2. **Implement Features**: Use SOLID principles and clean architecture
3. **Validate Compliance**: Ensure regulatory requirements are met
4. **Document Changes**: Update relevant documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Product Requirements Document](PRD.md) - Detailed requirements and roadmap
- [Development Checklist](DEVELOPMENT_CHECKLIST.md) - Current development status and tasks
- [FDA Logging Implementation](LOGGING_IMPLEMENTATION_SUMMARY.md) - Comprehensive logging system details
- [Issue Tracker](https://github.com/ryancinsight/OxiQMS/issues)

## 📞 Support

For questions, issues, or support:
- Create an [issue](https://github.com/ryancinsight/OxiQMS/issues)
- Email: support@oxiqms.com
- Documentation: [docs.oxiqms.com](https://docs.oxiqms.com)

---

**Note**: This software is designed for medical device development and includes features for regulatory compliance. Always consult with regulatory experts and conduct appropriate validation for your specific use case.

**Current Status**: Core modules in active development. FDA-compliant audit logging system implemented and operational. User management and testing infrastructure in progress.
