// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Authentication middleware and JWT handling.
//!
//! This module provides JWT validation and authentication configuration
//! for the Revelation server.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use masterror::AppError;
use revelation_user::{AuthConfig, Claims, JwtValidator};

/// JWT token manager for decoding and validating tokens.
///
/// Uses HS256 algorithm by default. Tokens are validated for:
/// - Signature validity
/// - Expiration time
///
/// # Example
///
/// ```rust,ignore
/// let manager = JwtManager::new("your-secret-key");
/// let claims = manager.decode("eyJ...")?;
/// ```
pub struct JwtManager {
    decoding_key: DecodingKey,
    validation:   Validation
}

impl JwtManager {
    /// Creates a new JWT manager with the given secret.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key for HS256 signature validation
    #[must_use]
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation
        }
    }
}

impl JwtValidator for JwtManager {
    fn decode(&self, token: &str) -> Result<Claims, AppError> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::unauthorized("Invalid or expired token"))
    }
}

/// Authentication configuration for cookie-based JWT.
///
/// Specifies the cookie name where JWT tokens are stored.
pub struct AppAuthConfig {
    cookie_name: String
}

impl AppAuthConfig {
    /// Creates new auth configuration.
    ///
    /// # Arguments
    ///
    /// * `cookie_name` - Name of the cookie containing the JWT token
    #[must_use]
    pub fn new(cookie_name: impl Into<String>) -> Self {
        Self {
            cookie_name: cookie_name.into()
        }
    }
}

impl AuthConfig for AppAuthConfig {
    fn cookie_name(&self) -> &str {
        &self.cookie_name
    }
}

/// Creates authentication extensions for the router.
///
/// # Arguments
///
/// * `jwt_secret` - Secret key for JWT validation
/// * `cookie_name` - Name of the auth cookie
///
/// # Returns
///
/// Tuple of (JwtValidator, AuthConfig) as Arc<dyn Trait>
#[must_use]
pub fn create_auth_extensions(
    jwt_secret: &str,
    cookie_name: &str
) -> (Arc<dyn JwtValidator>, Arc<dyn AuthConfig>) {
    let jwt: Arc<dyn JwtValidator> = Arc::new(JwtManager::new(jwt_secret));
    let config: Arc<dyn AuthConfig> = Arc::new(AppAuthConfig::new(cookie_name));
    (jwt, config)
}
