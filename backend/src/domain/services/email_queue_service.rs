#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::services::redis_service::RedisService;
use crate::domain::services::email_service::{EmailService, EmailError};

/// Email job types that can be queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailJobType {
    SendInvoice {
        to_email: String,
        to_name: String,
        invoice_number: String,
        pdf_url: String,
        amount: f64,
        due_date: String,
    },
    SendPaymentReminder {
        to_email: String,
        to_name: String,
        invoice_number: String,
        days_overdue: i64,
        amount_due: f64,
        reminder_type: String,
    },
    SendPaymentConfirmation {
        to_email: String,
        to_name: String,
        invoice_number: String,
        amount: f64,
        payment_method: String,
    },
    SendPasswordReset {
        to_email: String,
        to_name: String,
        reset_token: String,
    },
    SendVerificationEmail {
        to_email: String,
        to_name: String,
        verification_token: String,
    },
    SendInvoiceWithAttachment {
        to_email: String,
        to_name: String,
        invoice_number: String,
        pdf_bytes: Vec<u8>,
        amount: f64,
        due_date: String,
    },
}

/// Email job with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJob {
    pub id: String,
    pub job_type: EmailJobType,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: i64,
    pub scheduled_at: i64,
}

impl EmailJob {
    pub fn new(job_type: EmailJobType) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            job_type,
            retry_count: 0,
            max_retries: 3,
            created_at: now,
            scheduled_at: now, // Process immediately
        }
    }

    pub fn with_delay(job_type: EmailJobType, delay_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            job_type,
            retry_count: 0,
            max_retries: 3,
            created_at: now,
            scheduled_at: now + delay_seconds,
        }
    }

    pub fn should_process(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.scheduled_at
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        // Exponential backoff: 2^retry_count seconds
        let delay = 2_i64.pow(self.retry_count);
        self.scheduled_at = chrono::Utc::now().timestamp() + delay;
    }

    pub fn is_failed(&self) -> bool {
        self.retry_count >= self.max_retries
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmailQueueError {
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Email error: {0}")]
    EmailError(#[from] EmailError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Job processing failed after max retries")]
    MaxRetriesExceeded,
}

/// Email Queue Service - Provides reliable email delivery with retry logic
pub struct EmailQueueService {
    redis: Arc<RedisService>,
    email_service: Arc<EmailService>,
    queue_key: String,
    processing_key: String,
}

impl EmailQueueService {
    pub fn new(redis: Arc<RedisService>, email_service: Arc<EmailService>) -> Self {
        Self {
            redis,
            email_service,
            queue_key: "email_queue:pending".to_string(),
            processing_key: "email_queue:processing".to_string(),
        }
    }

    /// Queue an email job for async processing
    pub async fn enqueue(&self, job: EmailJob) -> Result<(), EmailQueueError> {
        let job_json = serde_json::to_string(&job)
            .map_err(|e| EmailQueueError::SerializationError(e.to_string()))?;

        // Add to pending queue
        self.redis
            .lpush(&self.queue_key, &job_json)
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?;

        tracing::info!(job_id = %job.id, "Email job queued");
        Ok(())
    }

    /// Queue an email with immediate processing
    pub async fn send_immediate(&self, job_type: EmailJobType) -> Result<(), EmailQueueError> {
        let job = EmailJob::new(job_type);
        self.process_job(&job).await
    }

    /// Process the next job in queue
    pub async fn process_next(&self) -> Result<Option<String>, EmailQueueError> {
        // Get next job from queue
        let job_json = match self
            .redis
            .rpop(&self.queue_key)
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?
        {
            Some(json) => json,
            None => return Ok(None), // Queue is empty
        };

        let mut job: EmailJob = serde_json::from_str(&job_json)
            .map_err(|e| EmailQueueError::SerializationError(e.to_string()))?;

        // Check if job should be processed (scheduled time)
        if !job.should_process() {
            // Re-queue for later
            self.redis
                .lpush(&self.queue_key, &job_json)
                .await
                .map_err(|e| EmailQueueError::RedisError(e.to_string()))?;
            return Ok(None);
        }

        // Process the job
        match self.process_job(&job).await {
            Ok(_) => {
                tracing::info!(job_id = %job.id, "Email job processed successfully");
                Ok(Some(job.id))
            }
            Err(e) => {
                job.increment_retry();

                if job.is_failed() {
                    tracing::error!(job_id = %job.id, error = %e, "Email job failed after max retries");
                    // Log to failed jobs for manual review
                    let failed_json = serde_json::to_string(&job)
                        .map_err(|e| EmailQueueError::SerializationError(e.to_string()))?;
                    self.redis
                        .lpush("email_queue:failed", &failed_json)
                        .await
                        .map_err(|e| EmailQueueError::RedisError(e.to_string()))?;
                    return Err(EmailQueueError::MaxRetriesExceeded);
                }

                // Re-queue for retry
                tracing::warn!(job_id = %job.id, retry_count = %job.retry_count, "Email job failed, retrying");
                let retry_json = serde_json::to_string(&job)
                    .map_err(|e| EmailQueueError::SerializationError(e.to_string()))?;
                self.redis
                    .lpush(&self.queue_key, &retry_json)
                    .await
                    .map_err(|e| EmailQueueError::RedisError(e.to_string()))?;

                Err(e)
            }
        }
    }

    /// Process a specific job
    async fn process_job(&self, job: &EmailJob) -> Result<(), EmailQueueError> {
        match &job.job_type {
            EmailJobType::SendInvoice {
                to_email,
                to_name,
                invoice_number,
                pdf_url,
                amount,
                due_date,
            } => {
                self.email_service
                    .send_invoice(to_email, to_name, invoice_number, pdf_url, *amount, due_date)?;
            }
            EmailJobType::SendPaymentReminder {
                to_email,
                to_name,
                invoice_number,
                days_overdue,
                amount_due,
                reminder_type,
            } => {
                self.email_service.send_payment_reminder(
                    to_email,
                    to_name,
                    invoice_number,
                    *days_overdue,
                    *amount_due,
                    reminder_type,
                )?;
            }
            EmailJobType::SendPaymentConfirmation {
                to_email,
                to_name,
                invoice_number,
                amount,
                payment_method,
            } => {
                self.email_service.send_payment_confirmation(
                    to_email,
                    to_name,
                    invoice_number,
                    *amount,
                    payment_method,
                )?;
            }
            EmailJobType::SendPasswordReset {
                to_email,
                to_name,
                reset_token,
            } => {
                self.email_service
                    .send_password_reset(to_email, to_name, reset_token)?;
            }
            EmailJobType::SendVerificationEmail {
                to_email,
                to_name,
                verification_token,
            } => {
                self.email_service
                    .send_verification_email(to_email, to_name, verification_token)?;
            }
            EmailJobType::SendInvoiceWithAttachment {
                to_email,
                to_name,
                invoice_number,
                pdf_bytes,
                amount,
                due_date,
            } => {
                self.email_service.send_invoice_with_attachment(
                    to_email,
                    to_name,
                    invoice_number,
                    pdf_bytes.clone(),
                    *amount,
                    due_date,
                )?;
            }
        }

        Ok(())
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> Result<QueueStats, EmailQueueError> {
        let pending = self
            .redis
            .llen(&self.queue_key)
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?
            .unwrap_or(0);

        let failed = self
            .redis
            .llen("email_queue:failed")
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?
            .unwrap_or(0);

        let processing = self
            .redis
            .llen(&self.processing_key)
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?
            .unwrap_or(0);

        Ok(QueueStats {
            pending,
            processing,
            failed,
        })
    }

    /// Clear failed jobs
    pub async fn clear_failed(&self) -> Result<(), EmailQueueError> {
        self.redis
            .delete("email_queue:failed")
            .await
            .map_err(|e| EmailQueueError::RedisError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct QueueStats {
    pub pending: i64,
    pub processing: i64,
    pub failed: i64,
}
