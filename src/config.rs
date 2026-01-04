// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Server configuration from environment variables.
//!
//! All configuration is loaded from environment variables with sensible
//! defaults for development. In production, sensitive values like `JWT_SECRET`
//! must be set.
//!
//! # Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `DATABASE_URL` | Yes | - | PostgreSQL connection string |
//! | `JWT_SECRET` | Yes | - | Secret for JWT signing (min 32 chars recommended) |
//! | `COOKIE_NAME` | No | `auth_token` | Name of the auth cookie |
//! | `HOST` | No | `0.0.0.0` | Server bind address |
//! | `PORT` | No | `3000` | Server port |
//! | `MAX_CONNECTIONS` | No | `10` | Database connection pool size |
//! | `ALLOWED_ORIGINS` | No | - | Comma-separated CORS origins |

use std::net::SocketAddr;

/// Server configuration loaded from environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection string.
    pub database_url: String,

    /// JWT signing secret.
    pub jwt_secret: String,

    /// Cookie name for JWT storage.
    pub cookie_name: String,

    /// Server bind address.
    pub host: String,

    /// Server port.
    pub port: u16,

    /// Database connection pool size.
    pub max_connections: u32,

    /// Allowed CORS origins.
    pub allowed_origins: Vec<String>
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if required environment variables (`DATABASE_URL`, `JWT_SECRET`)
    /// are not set.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// dotenvy::dotenv().ok();
    /// let config = Config::from_env();
    /// ```
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            database_url:    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret:      std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            cookie_name:     std::env::var("COOKIE_NAME").unwrap_or_else(|_| "auth_token".into()),
            host:            std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port:            std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(10),
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default()
        }
    }

    /// Returns the socket address for binding.
    ///
    /// # Panics
    ///
    /// Panics if host:port combination is invalid.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid HOST:PORT")
    }

    /// Returns true if CORS should be permissive (development mode).
    #[must_use]
    pub fn is_cors_permissive(&self) -> bool {
        self.allowed_origins.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_addr_formats_correctly() {
        let config = Config {
            database_url:    String::new(),
            jwt_secret:      String::new(),
            cookie_name:     String::new(),
            host:            "127.0.0.1".into(),
            port:            8080,
            max_connections: 5,
            allowed_origins: vec![]
        };

        assert_eq!(config.socket_addr().to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn is_cors_permissive_when_no_origins() {
        let config = Config {
            database_url:    String::new(),
            jwt_secret:      String::new(),
            cookie_name:     String::new(),
            host:            String::new(),
            port:            0,
            max_connections: 0,
            allowed_origins: vec![]
        };

        assert!(config.is_cors_permissive());
    }

    #[test]
    fn is_not_cors_permissive_when_origins_set() {
        let config = Config {
            database_url:    String::new(),
            jwt_secret:      String::new(),
            cookie_name:     String::new(),
            host:            String::new(),
            port:            0,
            max_connections: 0,
            allowed_origins: vec!["https://example.com".into()]
        };

        assert!(!config.is_cors_permissive());
    }
}
