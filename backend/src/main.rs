mod api;
mod domain;
mod infrastructure;
mod config;
mod application;

use axum::{
    http::{Method, HeaderValue},
    routing::get,
    Extension, Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    set_header::SetResponseHeaderLayer,
    limit::RequestBodyLimitLayer,
    timeout::RequestBodyTimeoutLayer,
};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::routes::{auth, invoices, reports, settings, clients, payments, expenses, metrics, files, tax, paypal};
use crate::domain::services::{InvoiceService, AuthService, EmailService, EmailConfig, PdfService, ReportService, SettingsService, ClientService, PaymentService, ExpenseService, RedisService, MetricsService, FileService, NotificationService, TaxService, PaymentGatewayService, MonitoringService, EmailQueueService};
use crate::application::use_cases::*;
use crate::infrastructure::repositories::{InvoiceRepository, ClientRepository, UserRepository, ReportRepositoryImpl, PaymentRepository, ExpenseRepository, TaxRepositoryImpl};
use crate::domain::repositories::TaxRepository;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "flashbill_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate environment variables and print configuration summary
    tracing::info!("🔍 Validating configuration...");
    crate::config::Config::validate_and_print();
    tracing::info!("✅ Configuration validated successfully\n");

    // Load configuration
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/flashbill".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Initialize Redis (optional - will gracefully fallback if not available)
    let redis_service = match RedisService::new(&redis_url) {
        Ok(redis) => {
            tracing::info!("✅ Redis connected successfully");
            Some(Arc::new(redis))
        }
        Err(e) => {
            tracing::warn!("⚠️ Redis not available: {}. Caching and rate limiting will be in-memory only.", e);
            None
        }
    };

    // Initialize database connection
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Initialize repositories (Infrastructure layer)
    let tax_repo_impl = TaxRepositoryImpl::new(db_pool.clone());
    let tax_repo: Arc<dyn TaxRepository + Send + Sync> = Arc::new(tax_repo_impl);

    // Initialize tax service (needed for invoice repository)
    let tax_service = Arc::new(TaxService::new(tax_repo.clone()));
    tracing::info!("✅ Tax service initialized");

    // Initialize payment gateway service (PayPal and Stripe)
    let payment_gateway_service = match PaymentGatewayService::new() {
        Ok(service) => {
            let gateways = service.get_available_gateways();
            if gateways.is_empty() {
                tracing::warn!("⚠️ Payment gateways not configured. Set STRIPE_SECRET_KEY and/or PAYPAL_CLIENT_ID/PAYPAL_CLIENT_SECRET");
            } else {
                tracing::info!("✅ Payment gateway service initialized: {:?}", gateways);
            }
            Arc::new(service)
        }
        Err(e) => {
            tracing::warn!("⚠️ Payment gateway service initialization failed: {}", e);
            Arc::new(PaymentGatewayService::default())
        }
    };

    // Initialize invoice repository with tax service
    let invoice_repo = InvoiceRepository::new(db_pool.clone(), tax_service.clone());
    let client_repo = ClientRepository::new(db_pool.clone());
    let user_repo = UserRepository::new(db_pool.clone());
    let report_repo = ReportRepositoryImpl::new(db_pool.clone());
    let payment_repo = PaymentRepository::new(db_pool.clone());
    let expense_repo = ExpenseRepository::new(db_pool.clone());

    // Initialize services (Domain layer)
    let metrics_service = Arc::new(MetricsService::new().expect("Failed to initialize metrics service"));
    tracing::info!("✅ Metrics service initialized");

    // Initialize file service
    let file_upload_dir = std::env::var("FILE_UPLOAD_DIR")
        .unwrap_or_else(|_| "./uploads".to_string());
    let max_file_size = std::env::var("MAX_FILE_SIZE")
        .unwrap_or_else(|_| "10485760".to_string()) // 10MB default
        .parse()
        .unwrap_or(10485760);

    let file_service = match FileService::new(&file_upload_dir, max_file_size) {
        Ok(service) => {
            tracing::info!("✅ File service initialized (upload dir: {})", file_upload_dir);
            Arc::new(service)
        }
        Err(e) => {
            tracing::error!("❌ Failed to initialize file service: {}", e);
            panic!("File service initialization failed");
        }
    };

    let pdf_service = PdfService::new();
    let email_service = Arc::new(EmailService::new(EmailConfig {
        smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
        smtp_port: std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587),
        username: std::env::var("SMTP_USER").unwrap_or_else(|_| "user".to_string()),
        password: std::env::var("SMTP_PASS").unwrap_or_else(|_| "pass".to_string()),
        from_email: std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@flashbill.com".to_string()),
        from_name: std::env::var("FROM_NAME").unwrap_or_else(|_| "FlashBill".to_string()),
    }));

    let notification_service = Arc::new(NotificationService::new().expect("Failed to initialize notification service"));
    tracing::info!("✅ Notification service initialized");

    // Initialize monitoring service
    let monitoring_service = Arc::new(MonitoringService::new());
    tracing::info!("✅ Monitoring service initialized");

    // Initialize email queue service (if Redis available)
    let email_queue_service = if let Some(redis) = &redis_service {
        let queue = Arc::new(EmailQueueService::new(redis.clone(), email_service.clone()));
        tracing::info!("✅ Email queue service initialized");
        Some(queue)
    } else {
        tracing::warn!("⚠️ Email queue disabled (Redis not available)");
        None
    };

    // Initialize services with repositories and other services
    // Clone invoice_repo before moving it into invoice_service
    let invoice_repo_for_payment = invoice_repo.clone();
    let invoice_repo_for_tax = invoice_repo.clone();
    let invoice_service = Arc::new(InvoiceService::new(
        invoice_repo,
        client_repo.clone(),
        user_repo.clone(),
        pdf_service,
        email_service.clone(),
    ));
    let auth_service = Arc::new(AuthService::new(user_repo.clone(), email_service.clone(), jwt_secret));
    let report_service = match &redis_service {
        Some(redis) => Arc::new(ReportService::with_redis(Arc::new(report_repo), redis.clone())),
        None => Arc::new(ReportService::new(Arc::new(report_repo))),
    };
    let settings_service = Arc::new(SettingsService::new(user_repo.clone()));
    let client_service = Arc::new(ClientService::new(Arc::new(client_repo.clone())));
    let payment_service = Arc::new(PaymentService::new(
        Arc::new(payment_repo.clone()),
        Arc::new(invoice_repo_for_payment),
        Arc::new(client_repo.clone()),
        Arc::new(user_repo.clone()),
        email_service.clone(),
    ));
    let expense_service = Arc::new(ExpenseService::new(Arc::new(expense_repo.clone())));

    // Initialize application use cases (Application layer - glue code)
    let create_invoice_uc = Arc::new(CreateInvoiceUseCase::new(invoice_service.clone()));
    let get_invoice_uc = Arc::new(GetInvoiceUseCase::new(invoice_service.clone()));
    let list_invoices_uc = Arc::new(ListInvoicesUseCase::new(invoice_service.clone()));
    let update_invoice_uc = Arc::new(UpdateInvoiceUseCase::new(invoice_service.clone()));
    let delete_invoice_uc = Arc::new(DeleteInvoiceUseCase::new(invoice_service.clone()));
    let record_payment_uc = Arc::new(RecordPaymentUseCase::new(invoice_service.clone()));
    let send_invoice_uc = Arc::new(SendInvoiceUseCase::new(invoice_service.clone()));
    let get_pdf_uc = Arc::new(GetInvoicePdfUseCase::new(invoice_service.clone()));
    let send_reminder_uc = Arc::new(SendReminderUseCase::new(invoice_service.clone()));

    let register_uc = Arc::new(RegisterUserUseCase::new(auth_service.clone()));
    let login_uc = Arc::new(LoginUserUseCase::new(auth_service.clone()));
    let refresh_token_uc = Arc::new(RefreshTokenUseCase::new(auth_service.clone()));
    let forgot_password_uc = Arc::new(ForgotPasswordUseCase::new(auth_service.clone()));
    let reset_password_uc = Arc::new(ResetPasswordUseCase::new(auth_service.clone()));
    let verify_email_uc = Arc::new(VerifyEmailUseCase::new(auth_service.clone()));
    let get_current_user_uc = Arc::new(GetCurrentUserUseCase::new(auth_service.clone()));
    let update_profile_uc = Arc::new(UpdateProfileUseCase::new(auth_service.clone()));

    // Report use cases
    let get_overview_stats_uc = Arc::new(GetOverviewStatsUseCase::new(report_service.clone()));
    let get_income_report_uc = Arc::new(GetIncomeReportUseCase::new(report_service.clone()));
    let get_expenses_report_uc = Arc::new(GetExpensesReportUseCase::new(report_service.clone()));
    let get_tax_report_uc = Arc::new(GetTaxReportUseCase::new(report_service.clone()));
    let get_aging_report_uc = Arc::new(GetAgingReportUseCase::new(report_service.clone()));
    let export_report_uc = Arc::new(ExportReportUseCase::new(report_service.clone()));

    // Settings use cases
    let get_business_settings_uc = Arc::new(GetBusinessSettingsUseCase::new(settings_service.clone()));
    let update_business_settings_uc = Arc::new(UpdateBusinessSettingsUseCase::new(settings_service.clone()));
    let get_notification_settings_uc = Arc::new(GetNotificationSettingsUseCase::new(settings_service.clone()));
    let update_notification_settings_uc = Arc::new(UpdateNotificationSettingsUseCase::new(settings_service.clone()));
    let get_invoice_settings_uc = Arc::new(GetInvoiceSettingsUseCase::new(settings_service.clone()));
    let update_invoice_settings_uc = Arc::new(UpdateInvoiceSettingsUseCase::new(settings_service.clone()));

    // Client use cases
    let create_client_uc = Arc::new(CreateClientUseCase::new(client_service.clone()));
    let get_client_uc = Arc::new(GetClientUseCase::new(client_service.clone()));
    let list_clients_uc = Arc::new(ListClientsUseCase::new(client_service.clone()));
    let update_client_uc = Arc::new(UpdateClientUseCase::new(client_service.clone()));
    let delete_client_uc = Arc::new(DeleteClientUseCase::new(client_service.clone()));
    let get_client_invoices_uc = Arc::new(GetClientInvoicesUseCase::new(client_service.clone()));
    let get_client_stats_uc = Arc::new(GetClientStatsUseCase::new(client_service.clone()));

    // Payment use cases
    let create_payment_uc = Arc::new(CreatePaymentUseCase::new(payment_service.clone()));
    let get_payment_uc = Arc::new(GetPaymentUseCase::new(payment_service.clone()));
    let list_payments_uc = Arc::new(ListPaymentsUseCase::new(payment_service.clone()));
    let refund_payment_uc = Arc::new(RefundPaymentUseCase::new(payment_service.clone()));
    let get_payment_stats_uc = Arc::new(GetPaymentStatsUseCase::new(payment_service.clone()));
    let get_payment_methods_uc = Arc::new(GetPaymentMethodsUseCase::new());

    // Expense use cases
    let create_expense_uc = Arc::new(CreateExpenseUseCase::new(expense_service.clone()));
    let get_expense_uc = Arc::new(GetExpenseUseCase::new(expense_service.clone()));
    let list_expenses_uc = Arc::new(ListExpensesUseCase::new(expense_service.clone()));
    let update_expense_uc = Arc::new(UpdateExpenseUseCase::new(expense_service.clone()));
    let delete_expense_uc = Arc::new(DeleteExpenseUseCase::new(expense_service.clone()));
    let get_expense_stats_uc = Arc::new(GetExpenseStatsUseCase::new(expense_service.clone()));

    // Tax use cases
    let create_tax_setting_uc = Arc::new(CreateTaxSettingUseCase::new(tax_service.clone()));
    let get_org_tax_settings_uc = Arc::new(GetOrganizationTaxSettingsUseCase::new(tax_service.clone()));
    let get_default_tax_uc = Arc::new(GetDefaultTaxUseCase::new(tax_service.clone()));
    let update_tax_setting_uc = Arc::new(UpdateTaxSettingUseCase::new(tax_service.clone()));
    let delete_tax_setting_uc = Arc::new(DeleteTaxSettingUseCase::new(tax_service.clone()));
    let calculate_tax_uc = Arc::new(CalculateTaxUseCase::new(tax_service.clone()));
    let get_tax_summary_uc = Arc::new(GetTaxSummaryUseCase::new(tax_service.clone()));
    let validate_tax_id_uc = Arc::new(ValidateTaxIdUseCase::new(tax_service.clone()));

    // Tax state for routes (needs invoice_repo for tax summary)
    let tax_state = tax::TaxState {
        create_tax_setting: create_tax_setting_uc,
        get_tax_settings: get_org_tax_settings_uc,
        get_default_tax: get_default_tax_uc,
        update_tax_setting: update_tax_setting_uc,
        delete_tax_setting: delete_tax_setting_uc,
        calculate_tax: calculate_tax_uc,
        get_tax_summary: get_tax_summary_uc,
        validate_tax_id: validate_tax_id_uc,
        invoice_repo: Arc::new(invoice_repo_for_tax),
    };

    // Create main router with security layers
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .nest("/api/v1", Router::new()
            .nest("/auth", auth::create_router(
                register_uc,
                login_uc,
                refresh_token_uc,
                forgot_password_uc,
                reset_password_uc,
                verify_email_uc,
                get_current_user_uc,
                update_profile_uc,
            ))
            .nest("/invoices", invoices::create_router(
                create_invoice_uc,
                get_invoice_uc,
                list_invoices_uc,
                update_invoice_uc,
                delete_invoice_uc,
                record_payment_uc,
                send_invoice_uc,
                get_pdf_uc,
                send_reminder_uc,
            ))
            .nest("/reports", reports::create_router(
                get_overview_stats_uc,
                get_income_report_uc,
                get_expenses_report_uc,
                get_tax_report_uc,
                get_aging_report_uc,
                export_report_uc,
            ))
            .nest("/settings", settings::create_router(
                get_business_settings_uc,
                update_business_settings_uc,
                get_notification_settings_uc,
                update_notification_settings_uc,
                get_invoice_settings_uc,
                update_invoice_settings_uc,
            ))
            .nest("/clients", clients::create_router(
                create_client_uc,
                get_client_uc,
                list_clients_uc,
                update_client_uc,
                delete_client_uc,
                get_client_invoices_uc,
                get_client_stats_uc,
            ))
            .nest("/payments", payments::create_router(
                create_payment_uc,
                get_payment_uc,
                list_payments_uc,
                refund_payment_uc,
                get_payment_stats_uc,
                get_payment_methods_uc,
            ))
            .nest("/paypal", paypal::create_paypal_router(payment_gateway_service.clone()))
            .nest("/expenses", expenses::create_router(
                create_expense_uc,
                get_expense_uc,
                list_expenses_uc,
                update_expense_uc,
                delete_expense_uc,
                get_expense_stats_uc,
            ))
            .nest("/files", files::create_router(file_service.clone()))
            .nest("/settings/tax", tax::create_settings_router(tax_state.clone()))
            .nest("/tax", tax::create_operations_router(tax_state))
        )
        // Metrics endpoint (public, no auth required)
        .nest("/metrics", metrics::create_router(
            metrics_service.clone(),
            monitoring_service.clone(),
            redis_service.clone(),
            Some(db_pool.clone()),
        ))
        .layer(Extension(auth_service))  // Add auth service to extensions for AuthUser extractor
        // Security: CORS configuration
        .layer(
            CorsLayer::new()
                .allow_origin(
                    std::env::var("CORS_ORIGIN")
                        .unwrap_or_else(|_| "http://localhost:3000".to_string())
                        .parse::<HeaderValue>()
                        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000"))
                )
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                    axum::http::header::ORIGIN,
                ])
                .allow_credentials(true)
                .max_age(Duration::from_secs(3600)),
        )
        // Security: Request body size limit (10MB max)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        // Security: Request timeout (30 seconds)
        .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(30)))
        // Security: Add security headers to responses
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        // Logging/Tracing
        .layer(TraceLayer::new_for_http());

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("🚀 FlashBill API server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ready_check() -> &'static str {
    "READY"
}
