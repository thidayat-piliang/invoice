use serde::Deserialize;
use std::env;
use crate::config::validation::{validate_env, print_config_summary};

pub mod validation;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_email: String,
    pub from_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://user:password@localhost:5432/flashbill".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_user: env::var("SMTP_USER").unwrap_or_else(|_| "user".to_string()),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_else(|_| "pass".to_string()),
            from_email: env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@flashbill.com".to_string()),
            from_name: env::var("FROM_NAME").unwrap_or_else(|_| "FlashBill".to_string()),
        }
    }

    /// Validate configuration and print summary
    pub fn validate_and_print() {
        match validate_env() {
            Ok(result) => {
                if !result.is_valid() {
                    eprintln!("❌ Configuration validation failed!");
                    if !result.missing_vars.is_empty() {
                        eprintln!("Missing required variables:");
                        for var in &result.missing_vars {
                            eprintln!("  - {}", var);
                        }
                    }
                    if !result.invalid_vars.is_empty() {
                        eprintln!("Invalid values:");
                        for (var, reason) in &result.invalid_vars {
                            eprintln!("  - {}: {}", var, reason);
                        }
                    }
                    panic!("Configuration validation failed");
                }

                if result.has_warnings() {
                    println!("⚠️  Configuration warnings:");
                    for warning in &result.warnings {
                        println!("  - {}", warning);
                    }
                    println!();
                }

                print_config_summary();
            }
            Err(e) => {
                eprintln!("❌ Configuration validation error: {}", e);
                panic!("Configuration error");
            }
        }
    }
}
