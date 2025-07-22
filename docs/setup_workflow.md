# QMS Web-Based Setup Workflow

## Overview

The QMS web-based setup workflow provides a seamless onboarding experience for new users, eliminating the need to manually run `qms init` from the command line. When no QMS project exists, the web server automatically serves a setup wizard that guides users through project initialization.

## Problem Solved

Previously, users would encounter "No QMS project found" errors when accessing the web interface without first running `qms init`. This created a poor user experience and required command-line knowledge. The setup workflow resolves this by:

1. **Automatic Detection**: Detects when no QMS project exists
2. **Web-Based Initialization**: Provides a user-friendly setup wizard
3. **Seamless Transition**: Automatically redirects to the main application after setup
4. **Error Prevention**: Eliminates audit logging failures during server startup

## Architecture

### Project Detection Logic

The system uses the following detection mechanism:

```rust
// New utility functions in src/utils.rs
pub fn qms_project_exists() -> bool
pub fn get_current_project_path_safe() -> Option<std::path::PathBuf>
```

These functions safely check for project existence without throwing errors, allowing the web server to conditionally serve content.

### Web Server Routing

The web server routing logic in `src/web/server.rs` has been modified to:

1. Check if a QMS project exists using `qms_project_exists()`
2. If no project exists:
   - Serve setup page for `/setup` or `/setup.html`
   - Redirect all other requests to `/setup`
   - Make setup API endpoints available at `/api/setup/*`
3. If project exists:
   - Serve normal application content
   - Enable full API functionality

### Setup API Endpoints

Three new API endpoints handle the setup workflow:

#### GET /api/setup/status
Returns the current setup status:
```json
{
  "project_exists": false,
  "setup_required": true
}
```

#### POST /api/setup/validate-directory
Validates a directory for project creation:

**Request:**
```json
{
  "directory": "/path/to/project"
}
```

**Response:**
```json
{
  "valid": true,
  "writable": true,
  "exists": true,
  "empty": true
}
```

#### POST /api/setup/initialize
Initializes the QMS project and creates the admin user:

**Request:**
```json
{
  "directory": "/path/to/project",
  "project_name": "My QMS Project",
  "admin_username": "admin",
  "admin_email": "admin@company.com",
  "admin_password": "securepassword123"
}
```

**Response:**
```json
{
  "success": true,
  "message": "QMS system initialized successfully",
  "project_id": "uuid-v4-project-id",
  "project_path": "/path/to/project/uuid-v4-project-id"
}
```

## User Interface

### Setup Wizard Steps

The setup wizard (`src/web_assets/setup.html`) provides a 3-step process:

1. **Project Location**: Directory selection and project naming
2. **Administrator Account**: User creation with email and password
3. **Initialize System**: Review and confirmation

### Features

- **Progress Indicators**: Visual progress bar and step indicators
- **Real-time Validation**: Directory validation and form validation
- **Error Handling**: Comprehensive error messages and recovery
- **Responsive Design**: Works on desktop and mobile devices
- **Medical Device Compliance**: Maintains FDA 21 CFR Part 820 compliance notices

## Implementation Details

### File Structure

New files added:
- `src/web_assets/setup.html` - Setup wizard HTML template
- `src/web_assets/setup.js` - Client-side JavaScript for setup workflow
- `tests/setup_workflow_test.rs` - Comprehensive integration tests

Modified files:
- `src/utils.rs` - Added project detection utilities
- `src/web/server.rs` - Added setup routing and API endpoints
- `src/web/assets.rs` - Added setup assets to asset manager

### Project Initialization Process

The setup workflow executes the following steps:

1. **Directory Validation**: Checks if directory exists, is writable, and is empty
2. **Project Creation**: Uses `Repository::init_project()` to create project structure
3. **User Creation**: Creates initial administrator user with full permissions
4. **Audit Logging**: Initializes audit system with the new project
5. **Redirect**: Redirects user to main application

### Security Considerations

- **Input Validation**: All user inputs are validated on both client and server
- **Directory Permissions**: Validates write permissions before project creation
- **Password Requirements**: Enforces minimum 8-character passwords
- **CSRF Protection**: Uses existing session management for security
- **Audit Trail**: All setup actions are logged for compliance

## Error Handling

The setup workflow includes comprehensive error handling:

### Client-Side Errors
- Form validation errors with user-friendly messages
- Network connectivity issues
- Invalid directory selections

### Server-Side Errors
- Directory permission failures
- Project creation failures
- User creation conflicts
- Filesystem errors

### Recovery Mechanisms
- Automatic retry for transient errors
- Clear error messages with suggested actions
- Graceful fallback to manual setup instructions

## Testing

The setup workflow includes comprehensive tests in `tests/setup_workflow_test.rs`:

- **Project Detection Tests**: Verify detection logic works correctly
- **Directory Validation Tests**: Test directory validation logic
- **Project Initialization Tests**: Verify complete project setup
- **User Creation Tests**: Test administrator user creation
- **Integration Tests**: End-to-end workflow testing

## Deployment Considerations

### Environment Variables

The system respects the `QMS_PROJECT_PATH` environment variable for project detection:

```bash
export QMS_PROJECT_PATH=/path/to/existing/project
```

### Server Startup

When starting the web server without a project:
1. Server starts successfully (no more initialization errors)
2. Audit logging warnings are suppressed until project exists
3. Setup workflow is automatically available

### Production Deployment

For production deployments:
1. Ensure web server has write permissions to target directories
2. Configure appropriate security headers (already implemented)
3. Consider pre-creating projects for enterprise deployments
4. Monitor setup completion rates and error patterns

## Medical Device Compliance

The setup workflow maintains full medical device compliance:

- **FDA 21 CFR Part 820**: Quality System Regulation compliance
- **ISO 13485**: Medical device quality management systems
- **ISO 14971**: Risk management for medical devices
- **21 CFR Part 11**: Electronic records and signatures

All setup actions are logged in the audit trail for regulatory compliance.

## Future Enhancements

Potential improvements for future versions:

1. **Multi-Project Support**: Allow selection of existing projects
2. **Advanced Configuration**: More detailed project configuration options
3. **Import/Export**: Import existing QMS data during setup
4. **Team Setup**: Multi-user setup with role assignments
5. **Cloud Integration**: Integration with cloud storage providers
6. **Backup Configuration**: Automatic backup setup during initialization

## Troubleshooting

### Common Issues

1. **Permission Denied**: Ensure web server has write access to target directory
2. **Port Conflicts**: Verify port 8080 is available or use `--port` option
3. **Browser Compatibility**: Use modern browsers with JavaScript enabled
4. **Network Issues**: Check firewall settings for local connections

### Debug Mode

Enable debug logging by setting environment variable:
```bash
export QMS_DEBUG=1
```

This provides additional logging for troubleshooting setup issues.

## Developer Integration Guide

### Adding New Setup Steps

To add new steps to the setup wizard:

1. **Update HTML Template**: Add new form section in `setup.html`
2. **Update JavaScript**: Add validation and navigation logic in `setup.js`
3. **Update API**: Extend `/api/setup/initialize` endpoint if needed
4. **Update Tests**: Add test cases for new functionality

### Extending Project Initialization

To customize project initialization:

1. **Modify Repository**: Update `Repository::init_project()` method
2. **Add Configuration**: Extend setup API to accept additional parameters
3. **Update UI**: Add configuration options to setup wizard
4. **Maintain Compatibility**: Ensure backward compatibility with CLI `qms init`

### Custom Validation

To add custom directory or input validation:

```rust
// In src/web/server.rs
fn custom_validation(input: &str) -> bool {
    // Add your validation logic here
    true
}
```

### Integration with External Systems

The setup workflow can be extended to integrate with:

- LDAP/Active Directory for user authentication
- Cloud storage for project data
- Enterprise configuration management
- Automated deployment systems

## API Reference Summary

| Endpoint | Method | Purpose | Authentication |
|----------|--------|---------|----------------|
| `/api/setup/status` | GET | Check setup status | None |
| `/api/setup/validate-directory` | POST | Validate directory | None |
| `/api/setup/initialize` | POST | Initialize QMS | None |

Note: Setup endpoints are only available when no QMS project exists.
