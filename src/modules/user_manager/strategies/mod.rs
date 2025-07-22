// Strategy Pattern - Pluggable authentication and authorization strategies
// Allows different authentication methods and authorization policies

pub mod password_auth_strategy;
pub mod role_auth_strategy;
pub mod session_strategy;

pub use password_auth_strategy::PasswordAuthenticationStrategy;
pub use role_auth_strategy::RoleBasedAuthorizationStrategy;
pub use session_strategy::DefaultSessionStrategy;
