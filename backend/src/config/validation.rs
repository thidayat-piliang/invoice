#![allow(dead_code)]

use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("Invalid value for environment variable {0}: {1}")]
    InvalidValue(String, String),

    #[error("Invalid port number: {0}")]
    InvalidPort(String),
}

/// Environment variable validation result
pub struct ValidationResult {
    pub missing_vars: Vec<String>,
    pub invalid_vars: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.missing_vars.is_empty() && self.invalid_vars.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validates all required environment variables
pub fn validate_env() -> Result<ValidationResult, ConfigError> {
    let mut result = ValidationResult {
        missing_vars: Vec::new(),
        invalid_vars: Vec::new(),
        warnings: Vec::new(),
    };

    // Required variables
    let required_vars = vec![
        "DATABASE_URL",
        "JWT_SECRET",
    ];

    // Optional but recommended variables
    let recommended_vars = vec![
        "REDIS_URL",
        "CORS_ORIGIN",
    ];

    // Service-specific variables
    let service_vars = vec![
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USER",
        "SMTP_PASS",
        "FROM_EMAIL",
        "STRIPE_SECRET_KEY",
        "PAYPAL_CLIENT_ID",
        "PAYPAL_CLIENT_SECRET",
        "FIREBASE_API_KEY",
        "FCM_SERVER_KEY",
    ];

    // Check required variables
    for var in required_vars {
        match env::var(var) {
            Ok(_) => {}
            Err(_) => result.missing_vars.push(var.to_string()),
        }
    }

    // Check recommended variables
    for var in recommended_vars {
        if env::var(var).is_err() {
            result.warnings.push(format!(
                "Recommended variable {} is not set. Using default value.", var
            ));
        }
    }

    // Check service variables and provide warnings
    for var in service_vars {
        if env::var(var).is_err() {
            result.warnings.push(format!(
                "Service variable {} is not set. {} functionality will be disabled.", var, var
            ));
        }
    }

    // Validate PORT if set
    if let Ok(port) = env::var("PORT") {
        if let Ok(port_num) = port.parse::<u16>() {
            if port_num == 0 {
                result.invalid_vars.push((
                    "PORT".to_string(),
                    "Port must be between 1 and 65535".to_string(),
                ));
            }
        } else {
            result.invalid_vars.push((
                "PORT".to_string(),
                "Port must be a valid number".to_string(),
            ));
        }
    }

    // Validate SMTP_PORT if set
    if let Ok(port) = env::var("SMTP_PORT") {
        if let Ok(port_num) = port.parse::<u16>() {
            if port_num == 0 {
                result.invalid_vars.push((
                    "SMTP_PORT".to_string(),
                    "SMTP port must be between 1 and 65535".to_string(),
                ));
            }
        } else {
            result.invalid_vars.push((
                "SMTP_PORT".to_string(),
                "SMTP port must be a valid number".to_string(),
            ));
        }
    }

    // Validate email addresses if set
    if let Ok(email) = env::var("FROM_EMAIL") {
        if !email.contains('@') {
            result.invalid_vars.push((
                "FROM_EMAIL".to_string(),
                "Invalid email format".to_string(),
            ));
        }
    }

    // Validate CORS origin format
    if let Ok(origin) = env::var("CORS_ORIGIN") {
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            result.warnings.push(format!(
                "CORS_ORIGIN should start with http:// or https://, got: {}", origin
            ));
        }
    }

    // Validate file upload directory if set
    if let Ok(dir) = env::var("FILE_UPLOAD_DIR") {
        // Check if path is absolute or relative
        if !dir.starts_with('/') && !dir.starts_with("./") && !dir.starts_with("../") {
            result.warnings.push(format!(
                "FILE_UPLOAD_DIR should be a clear path. Current: {}", dir
            ));
        }
    }

    // Validate max file size if set
    if let Ok(size) = env::var("MAX_FILE_SIZE") {
        if let Ok(size_bytes) = size.parse::<usize>() {
            if size_bytes == 0 || size_bytes > 100 * 1024 * 1024 { // Max 100MB
                result.warnings.push(format!(
                    "MAX_FILE_SIZE should be between 1 and 104857600 bytes (100MB). Got: {}", size_bytes
                ));
            }
        } else {
            result.invalid_vars.push((
                "MAX_FILE_SIZE".to_string(),
                "Must be a valid number in bytes".to_string(),
            ));
        }
    }

    Ok(result)
}

/// Validates and provides configuration summary
pub fn print_config_summary() {
    println!("=== FlashBill Configuration Summary ===");
    println!();

    // Database
    if let Ok(db_url) = env::var("DATABASE_URL") {
        let masked = mask_url(&db_url);
        println!("✅ Database: {}", masked);
    } else {
        println!("❌ Database: DATABASE_URL not set");
    }

    // Redis
    if let Ok(redis_url) = env::var("REDIS_URL") {
        println!("✅ Redis: {}", redis_url);
    } else {
        println!("⚠️  Redis: Not configured (using in-memory cache)");
    }

    // Email
    if env::var("SMTP_HOST").is_ok() {
        println!("✅ Email: SMTP configured");
    } else {
        println!("⚠️  Email: Not configured (emails will be skipped)");
    }

    // Payment Gateways
    let mut gateways = Vec::new();
    if env::var("STRIPE_SECRET_KEY").is_ok() {
        gateways.push("Stripe");
    }
    if env::var("PAYPAL_CLIENT_ID").is_ok() {
        gateways.push("PayPal");
    }
    if !gateways.is_empty() {
        println!("✅ Payment Gateways: {}", gateways.join(", "));
    } else {
        println!("⚠️  Payment Gateways: Not configured");
    }

    // Firebase
    if env::var("FIREBASE_API_KEY").is_ok() {
        println!("✅ Firebase: Configured");
    } else {
        println!("⚠️  Firebase: Not configured (push notifications disabled)");
    }

    // Security
    if env::var("JWT_SECRET").is_ok() {
        println!("✅ JWT Secret: Configured");
    } else {
        println!("❌ JWT Secret: Not set");
    }

    println!();
    println!("Server will start on: http://0.0.0.0:{}", env::var("PORT").unwrap_or_else(|_| "3000".to_string()));
    println!("========================================");
}

/// Mask sensitive URL (hide password)
fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url.find(':') {
            if colon_pos < at_pos {
                return format!("{}://***:***@{}", &url[..colon_pos], &url[at_pos+1..]);
            }
        }
    }
    url.to_string()
}

/// Get required env var with default
pub fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get required env var or panic
pub fn get_required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("Required environment variable {} is not set", key)
    })
}

/// Check if running in test mode
pub fn is_test_mode() -> bool {
    env::var("TEST_MODE").is_ok() || env::var("SKIP_EMAIL").is_ok()
}

/// Check if email queue should be skipped
#[allow(dead_code)]
pub fn should_skip_queue() -> bool {
    env::var("SKIP_QUEUE").is_ok()
}
