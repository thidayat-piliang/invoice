use lettre::{
    message::{Mailbox, MultiPart, SinglePart, header::{ContentType, ContentDisposition, Header}},
    transport::smtp::{authentication::Credentials, client::{Tls, TlsParameters}},
    Address, Message, SmtpTransport, Transport,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("SMTP error: {0}")]
    SmtpError(String),

    #[error("Invalid email address")]
    InvalidEmail,

    #[error("Failed to build message")]
    MessageBuildError,
}

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    pub fn send_invoice(
        &self,
        to_email: &str,
        to_name: &str,
        invoice_number: &str,
        pdf_url: &str,
        amount: f64,
        due_date: &str,
    ) -> Result<(), EmailError> {
        let subject = format!("Invoice #{} - ${:.2}", invoice_number, amount);

        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Invoice #{}</h2>
                <p>Hello {},</p>
                <p>You have received an invoice for <strong>${:.2}</strong>.</p>
                <p><strong>Due Date:</strong> {}</p>
                <p>You can view and download your invoice using the link below:</p>
                <p><a href="{}" style="background-color: #4361EE; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">View Invoice</a></p>
                <p>Thank you for your business!</p>
                <hr>
                <p style="font-size: 12px; color: #666;">This is an automated message from FlashBill</p>
            </body>
            </html>
            "#,
            invoice_number, to_name, amount, due_date, pdf_url
        );

        self.send_email(to_email, to_name, &subject, &body)
    }

    pub fn send_payment_reminder(
        &self,
        to_email: &str,
        to_name: &str,
        invoice_number: &str,
        days_overdue: i64,
        amount_due: f64,
        reminder_type: &str,
    ) -> Result<(), EmailError> {
        let subject = match reminder_type {
            "friendly" => format!("Friendly reminder: Invoice #{}", invoice_number),
            "urgent" => format!("URGENT: Invoice #{} is overdue", invoice_number),
            "final_notice" => format!("FINAL NOTICE: Invoice #{}", invoice_number),
            _ => format!("Payment reminder: Invoice #{}", invoice_number),
        };

        let tone = match reminder_type {
            "friendly" => "This is a friendly reminder",
            "urgent" => "This is an urgent reminder",
            "final_notice" => "This is our final notice",
            _ => "This is a reminder",
        };

        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Payment Reminder</h2>
                <p>Hello {},</p>
                <p>{} that invoice <strong>#{}</strong> is <strong>{} days overdue</strong>.</p>
                <p><strong>Amount Due:</strong> ${:.2}</p>
                <p>Please make payment as soon as possible to avoid late fees.</p>
                <p><a href="https://app.flashbill.com/invoices/{}" style="background-color: #F44336; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Pay Now</a></p>
                <p>If you have already paid, please disregard this email.</p>
                <hr>
                <p style="font-size: 12px; color: #666;">FlashBill Automated Reminder</p>
            </body>
            </html>
            "#,
            to_name, tone, invoice_number, days_overdue, amount_due, invoice_number
        );

        self.send_email(to_email, to_name, &subject, &body)
    }

    pub fn send_payment_confirmation(
        &self,
        to_email: &str,
        to_name: &str,
        invoice_number: &str,
        amount: f64,
        payment_method: &str,
    ) -> Result<(), EmailError> {
        let subject = format!("Payment Received - Invoice #{}", invoice_number);

        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Payment Confirmation</h2>
                <p>Hello {},</p>
                <p>We have received your payment of <strong>${:.2}</strong> for invoice <strong>#{}</strong>.</p>
                <p><strong>Payment Method:</strong> {}</p>
                <p>Your invoice has been marked as paid.</p>
                <p>Thank you for your prompt payment!</p>
                <hr>
                <p style="font-size: 12px; color: #666;">FlashBill Payment Confirmation</p>
            </body>
            </html>
            "#,
            to_name, amount, invoice_number, payment_method
        );

        self.send_email(to_email, to_name, &subject, &body)
    }

    pub fn send_password_reset(
        &self,
        to_email: &str,
        to_name: &str,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let subject = "Password Reset Request".to_string();

        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Password Reset</h2>
                <p>Hello {},</p>
                <p>You have requested to reset your password.</p>
                <p>Your reset token is:</p>
                <p style="background-color: #f0f0f0; padding: 10px; font-family: monospace; font-size: 16px;">{}</p>
                <p>This token will expire in 1 hour.</p>
                <p><a href="https://app.flashbill.com/reset-password?token={}" style="background-color: #4361EE; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Reset Password</a></p>
                <p>If you did not request this, please ignore this email.</p>
                <hr>
                <p style="font-size: 12px; color: #666;">FlashBill Security</p>
            </body>
            </html>
            "#,
            to_name, reset_token, reset_token
        );

        self.send_email(to_email, to_name, &subject, &body)
    }

    pub fn send_verification_email(
        &self,
        to_email: &str,
        to_name: &str,
        verification_token: &str,
    ) -> Result<(), EmailError> {
        let subject = "Verify Your Email Address".to_string();

        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Welcome to FlashBill!</h2>
                <p>Hello {},</p>
                <p>Please verify your email address by clicking the button below:</p>
                <p><a href="https://app.flashbill.com/verify-email?token={}" style="background-color: #4361EE; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Verify Email</a></p>
                <p>If the button doesn't work, use this token: {}</p>
                <hr>
                <p style="font-size: 12px; color: #666;">FlashBill Team</p>
            </body>
            </html>
            "#,
            to_name, verification_token, verification_token
        );

        self.send_email(to_email, to_name, &subject, &body)
    }

    pub fn send_email(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), EmailError> {
        let from_mailbox: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|_| EmailError::InvalidEmail)?;

        let to_mailbox: Mailbox = format!("{} <{}>", to_name, to_email)
            .parse()
            .map_err(|_| EmailError::InvalidEmail)?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|_| EmailError::MessageBuildError)?;

        let credentials = Credentials::new(self.config.username.clone(), self.config.password.clone());

        let mailer = SmtpTransport::relay(&self.config.smtp_host)
            .map_err(|e| EmailError::SmtpError(e.to_string()))?
            .port(self.config.smtp_port)
            .credentials(credentials)
            .tls(Tls::Required(
                TlsParameters::new_native(self.config.smtp_host.clone())
                    .map_err(|e| EmailError::SmtpError(e.to_string()))?
            ))
            .build();

        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(EmailError::SmtpError(e.to_string())),
        }
    }

    /// Send invoice with PDF attachment
    pub fn send_invoice_with_attachment(
        &self,
        to_email: &str,
        to_name: &str,
        invoice_number: &str,
        pdf_bytes: Vec<u8>,
        amount: f64,
        due_date: &str,
    ) -> Result<(), EmailError> {
        let subject = format!("Invoice #{} - ${:.2}", invoice_number, amount);

        let html_body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Invoice #{}</h2>
                <p>Hello {},</p>
                <p>You have received an invoice for <strong>${:.2}</strong>.</p>
                <p><strong>Due Date:</strong> {}</p>
                <p>Please find your invoice attached to this email as a PDF.</p>
                <p>Thank you for your business!</p>
                <hr>
                <p style="font-size: 12px; color: #666;">This is an automated message from FlashBill</p>
            </body>
            </html>
            "#,
            invoice_number, to_name, amount, due_date
        );

        let from_mailbox: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|_| EmailError::InvalidEmail)?;

        let to_mailbox: Mailbox = format!("{} <{}>", to_name, to_email)
            .parse()
            .map_err(|_| EmailError::InvalidEmail)?;

        // Create multipart message with HTML body and PDF attachment
        let pdf_filename = format!("invoice_{}.pdf", invoice_number);
        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .multipart(
                MultiPart::mixed()
                    .singlepart(
                        SinglePart::html(html_body)
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::parse("application/pdf").map_err(|_| EmailError::MessageBuildError)?)
                            .header(ContentDisposition::attachment(&pdf_filename))
                            .body(pdf_bytes)
                    )
            )
            .map_err(|_| EmailError::MessageBuildError)?;

        let credentials = Credentials::new(self.config.username.clone(), self.config.password.clone());

        let mailer = SmtpTransport::relay(&self.config.smtp_host)
            .map_err(|e| EmailError::SmtpError(e.to_string()))?
            .port(self.config.smtp_port)
            .credentials(credentials)
            .tls(Tls::Required(
                TlsParameters::new_native(self.config.smtp_host.clone())
                    .map_err(|e| EmailError::SmtpError(e.to_string()))?
            ))
            .build();

        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(EmailError::SmtpError(e.to_string())),
        }
    }
}
