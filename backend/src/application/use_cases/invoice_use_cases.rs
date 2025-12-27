use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::invoice_dto::*;
use crate::domain::models::{CreateInvoice, UpdateInvoice, CreatePayment, InvoiceListFilter, SenderType};
use crate::domain::services::{InvoiceService, InvoiceError};

/// Use case: Create a new invoice
///
/// This orchestrates the business logic for creating an invoice:
/// 1. Validates input
/// 2. Creates invoice via service
/// 3. Returns DTO to API layer
pub struct CreateInvoiceUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl CreateInvoiceUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, command: CreateInvoiceCommand) -> Result<InvoiceCreatedDto, InvoiceError> {
        // Convert command to domain model
        let create_invoice = CreateInvoice {
            client_id: command.client_id,
            issue_date: command.issue_date,
            due_date: command.due_date,
            items: command.items.into_iter().map(|item| crate::domain::models::CreateInvoiceItem {
                description: item.description,
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate: item.tax_rate,
            }).collect(),
            notes: command.notes,
            terms: command.terms,
            discount_amount: command.discount_amount,
            tax_included: command.tax_included,
            send_immediately: command.send_immediately,
            tax_label: None,  // Will be set by service based on default tax
            tax_id: None,    // Will be set by service based on default tax
            allow_partial_payment: command.allow_partial_payment,
            min_payment_amount: command.min_payment_amount,
        };

        // Execute business logic via service
        let invoice = self.invoice_service.create_invoice(user_id, create_invoice).await?;

        // Convert to output DTO
        let status_str = invoice.status.to_string();
        Ok(InvoiceCreatedDto {
            id: invoice.id,
            invoice_number: invoice.invoice_number.clone(),
            status: invoice.status,
            subtotal: invoice.subtotal,
            tax_amount: invoice.tax_amount,
            total_amount: invoice.total_amount,
            tax_label: invoice.tax_label,
            message: if status_str == "sent" {
                "Invoice created and sent successfully".to_string()
            } else {
                "Invoice created as draft".to_string()
            },
        })
    }
}

/// Use case: Get invoice details
pub struct GetInvoiceUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl GetInvoiceUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<InvoiceDto, InvoiceError> {
        let invoice = self.invoice_service.get_invoice(user_id, invoice_id).await?;

        Ok(InvoiceDto {
            id: invoice.id,
            invoice_number: invoice.invoice_number,
            status: invoice.status,
            client_id: invoice.client_id,
            client_name: invoice.client_name,
            client_email: invoice.client_email,
            client_phone: invoice.client_phone,
            client_address: invoice.client_address,
            issue_date: invoice.issue_date,
            due_date: invoice.due_date,
            subtotal: invoice.subtotal,
            tax_amount: invoice.tax_amount,
            discount_amount: invoice.discount_amount,
            total_amount: invoice.total_amount,
            amount_paid: invoice.amount_paid,
            balance_due: invoice.balance_due,
            items: invoice.items,
            notes: invoice.notes,
            terms: invoice.terms,
            tax_calculation: invoice.tax_calculation,
            tax_included: invoice.tax_included,
            tax_label: invoice.tax_label,
            tax_id: invoice.tax_id,
            pdf_url: invoice.pdf_url,
            receipt_image_url: invoice.receipt_image_url,
            sent_at: invoice.sent_at,
            viewed_at: invoice.viewed_at,
            paid_at: invoice.paid_at,
            reminder_sent_count: invoice.reminder_sent_count,
            last_reminder_sent: invoice.last_reminder_sent,
            notification_sent_at: invoice.notification_sent_at,
            whatsapp_sent_at: invoice.whatsapp_sent_at,
            guest_payment_token: invoice.guest_payment_token,
            allow_partial_payment: invoice.allow_partial_payment,
            min_payment_amount: invoice.min_payment_amount,
            partial_payment_count: invoice.partial_payment_count,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
        })
    }
}

/// Use case: List invoices with filtering
pub struct ListInvoicesUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl ListInvoicesUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, query: InvoiceListQuery) -> Result<Vec<InvoiceSummaryDto>, InvoiceError> {
        let filter = InvoiceListFilter {
            status: query.status.and_then(|s| match s.as_str() {
                "draft" => Some(crate::domain::models::InvoiceStatus::Draft),
                "sent" => Some(crate::domain::models::InvoiceStatus::Sent),
                "viewed" => Some(crate::domain::models::InvoiceStatus::Viewed),
                "partial" => Some(crate::domain::models::InvoiceStatus::Partial),
                "paid" => Some(crate::domain::models::InvoiceStatus::Paid),
                "overdue" => Some(crate::domain::models::InvoiceStatus::Overdue),
                "cancelled" => Some(crate::domain::models::InvoiceStatus::Cancelled),
                _ => None,
            }),
            client_id: query.client_id,
            date_from: query.date_from,
            date_to: query.date_to,
            search: query.search,
            limit: query.limit,
            offset: query.offset,
        };

        let invoices = self.invoice_service.list_invoices(user_id, filter).await?;

        Ok(invoices.into_iter().map(|inv| InvoiceSummaryDto {
            id: inv.id,
            invoice_number: inv.invoice_number,
            status: inv.status,
            client_name: inv.client_name,
            client_email: inv.client_email,
            issue_date: inv.issue_date,
            due_date: inv.due_date,
            total_amount: inv.total_amount,
            balance_due: inv.balance_due,
            days_until_due: inv.days_until_due,
            is_overdue: inv.is_overdue,
            created_at: inv.created_at,
        }).collect())
    }
}

/// Use case: Update invoice
pub struct UpdateInvoiceUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl UpdateInvoiceUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid, command: UpdateInvoiceCommand) -> Result<InvoiceDto, InvoiceError> {
        let update = UpdateInvoice {
            client_id: command.client_id,
            issue_date: command.issue_date,
            due_date: command.due_date,
            items: command.items.map(|items| items.into_iter().map(|item| crate::domain::models::CreateInvoiceItem {
                description: item.description,
                quantity: item.quantity,
                unit_price: item.unit_price,
                tax_rate: item.tax_rate,
            }).collect()),
            notes: command.notes,
            terms: command.terms,
            discount_amount: command.discount_amount,
            tax_included: command.tax_included,
            allow_partial_payment: command.allow_partial_payment,
            min_payment_amount: command.min_payment_amount,
        };

        let invoice = self.invoice_service.update_invoice(user_id, invoice_id, update).await?;

        Ok(InvoiceDto {
            id: invoice.id,
            invoice_number: invoice.invoice_number,
            status: invoice.status,
            client_id: invoice.client_id,
            client_name: invoice.client_name,
            client_email: invoice.client_email,
            client_phone: invoice.client_phone,
            client_address: invoice.client_address,
            issue_date: invoice.issue_date,
            due_date: invoice.due_date,
            subtotal: invoice.subtotal,
            tax_amount: invoice.tax_amount,
            discount_amount: invoice.discount_amount,
            total_amount: invoice.total_amount,
            amount_paid: invoice.amount_paid,
            balance_due: invoice.balance_due,
            items: invoice.items,
            notes: invoice.notes,
            terms: invoice.terms,
            tax_calculation: invoice.tax_calculation,
            tax_included: invoice.tax_included,
            tax_label: invoice.tax_label,
            tax_id: invoice.tax_id,
            pdf_url: invoice.pdf_url,
            receipt_image_url: invoice.receipt_image_url,
            sent_at: invoice.sent_at,
            viewed_at: invoice.viewed_at,
            paid_at: invoice.paid_at,
            reminder_sent_count: invoice.reminder_sent_count,
            last_reminder_sent: invoice.last_reminder_sent,
            notification_sent_at: invoice.notification_sent_at,
            whatsapp_sent_at: invoice.whatsapp_sent_at,
            guest_payment_token: invoice.guest_payment_token,
            allow_partial_payment: invoice.allow_partial_payment,
            min_payment_amount: invoice.min_payment_amount,
            partial_payment_count: invoice.partial_payment_count,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
        })
    }
}

/// Use case: Delete invoice
pub struct DeleteInvoiceUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl DeleteInvoiceUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<(), InvoiceError> {
        self.invoice_service.delete_invoice(user_id, invoice_id).await
    }
}

/// Use case: Record payment for invoice
pub struct RecordPaymentUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl RecordPaymentUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid, command: RecordPaymentCommand) -> Result<PaymentRecordedDto, InvoiceError> {
        let payment = CreatePayment {
            invoice_id,
            amount: command.amount,
            payment_method: match command.payment_method.as_str() {
                "stripe" => crate::domain::models::PaymentMethod::Stripe,
                "paypal" => crate::domain::models::PaymentMethod::PayPal,
                "check" => crate::domain::models::PaymentMethod::Check,
                "cash" => crate::domain::models::PaymentMethod::Cash,
                "bank_transfer" => crate::domain::models::PaymentMethod::BankTransfer,
                _ => crate::domain::models::PaymentMethod::Stripe,
            },
            gateway: None,
            gateway_payment_id: None,
            gateway_fee: None,
            paid_by: None,
            notes: command.notes,
        };

        let invoice = self.invoice_service.record_payment(user_id, invoice_id, payment).await?;

        Ok(PaymentRecordedDto {
            invoice_id: invoice.id,
            invoice_number: invoice.invoice_number,
            amount_paid: invoice.amount_paid,
            new_balance: invoice.balance_due,
            status: invoice.status,
            message: "Payment recorded successfully".to_string(),
        })
    }
}

/// Use case: Send invoice to client
pub struct SendInvoiceUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl SendInvoiceUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid, command: SendInvoiceCommand) -> Result<(), InvoiceError> {
        self.invoice_service.send_invoice(user_id, invoice_id, command.email).await
    }
}

/// Use case: Get invoice PDF
pub struct GetInvoicePdfUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl GetInvoicePdfUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<Vec<u8>, InvoiceError> {
        self.invoice_service.get_invoice_pdf(user_id, invoice_id).await
    }
}

/// Use case: Send invoice reminder
pub struct SendReminderUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl SendReminderUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<(), InvoiceError> {
        self.invoice_service.send_reminder(user_id, invoice_id).await
    }
}

/// Use case: Send invoice via WhatsApp
pub struct SendInvoiceWhatsappUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl SendInvoiceWhatsappUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<(), InvoiceError> {
        self.invoice_service.send_invoice_whatsapp(user_id, invoice_id).await
    }
}

/// Use case: Mark invoice as viewed (read receipt)
pub struct MarkInvoiceViewedUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl MarkInvoiceViewedUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, invoice_id: Uuid) -> Result<InvoiceDto, InvoiceError> {
        let invoice = self.invoice_service.mark_as_viewed(invoice_id).await?;

        Ok(InvoiceDto {
            id: invoice.id,
            invoice_number: invoice.invoice_number,
            status: invoice.status,
            client_id: invoice.client_id,
            client_name: invoice.client_name,
            client_email: invoice.client_email,
            client_phone: invoice.client_phone,
            client_address: invoice.client_address,
            issue_date: invoice.issue_date,
            due_date: invoice.due_date,
            subtotal: invoice.subtotal,
            tax_amount: invoice.tax_amount,
            discount_amount: invoice.discount_amount,
            total_amount: invoice.total_amount,
            amount_paid: invoice.amount_paid,
            balance_due: invoice.balance_due,
            items: invoice.items,
            notes: invoice.notes,
            terms: invoice.terms,
            tax_calculation: invoice.tax_calculation,
            tax_included: invoice.tax_included,
            tax_label: invoice.tax_label,
            tax_id: invoice.tax_id,
            pdf_url: invoice.pdf_url,
            receipt_image_url: invoice.receipt_image_url,
            sent_at: invoice.sent_at,
            viewed_at: invoice.viewed_at,
            paid_at: invoice.paid_at,
            reminder_sent_count: invoice.reminder_sent_count,
            last_reminder_sent: invoice.last_reminder_sent,
            notification_sent_at: invoice.notification_sent_at,
            whatsapp_sent_at: invoice.whatsapp_sent_at,
            guest_payment_token: invoice.guest_payment_token,
            allow_partial_payment: invoice.allow_partial_payment,
            min_payment_amount: invoice.min_payment_amount,
            partial_payment_count: invoice.partial_payment_count,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
        })
    }
}

/// Use case: Send payment confirmation
pub struct SendPaymentConfirmationUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl SendPaymentConfirmationUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    pub async fn execute(&self, user_id: Uuid, invoice_id: Uuid) -> Result<(), InvoiceError> {
        self.invoice_service.send_payment_confirmation(user_id, invoice_id).await
    }
}

/// Use case: Add discussion message (seller or buyer)
pub struct AddDiscussionMessageUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl AddDiscussionMessageUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    /// Add a discussion message as a seller
    pub async fn execute_as_seller(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
        command: AddDiscussionMessageCommand,
    ) -> Result<DiscussionMessageDto, InvoiceError> {
        let discussion = self.invoice_service
            .add_discussion_message(user_id, invoice_id, SenderType::Seller, command.message)
            .await?;

        Ok(DiscussionMessageDto {
            id: discussion.id,
            invoice_id: discussion.invoice_id,
            sender_type: discussion.sender_type.to_string(),
            message: discussion.message,
            created_at: discussion.created_at,
        })
    }

    /// Add a discussion message as a buyer (guest access)
    pub async fn execute_as_guest(
        &self,
        invoice_id: Uuid,
        command: AddDiscussionMessageCommand,
    ) -> Result<DiscussionMessageDto, InvoiceError> {
        let discussion = self.invoice_service
            .add_discussion_message_guest(invoice_id, command.message)
            .await?;

        Ok(DiscussionMessageDto {
            id: discussion.id,
            invoice_id: discussion.invoice_id,
            sender_type: discussion.sender_type.to_string(),
            message: discussion.message,
            created_at: discussion.created_at,
        })
    }
}

/// Use case: Get discussion messages
pub struct GetDiscussionMessagesUseCase {
    invoice_service: Arc<InvoiceService>,
}

impl GetDiscussionMessagesUseCase {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }

    /// Get all discussion messages for an invoice (seller access)
    pub async fn execute(
        &self,
        user_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<DiscussionResponseDto, InvoiceError> {
        let messages = self.invoice_service
            .get_discussion_messages(user_id, invoice_id)
            .await?;

        let dtos = messages.into_iter().map(|msg| DiscussionMessageDto {
            id: msg.id,
            invoice_id: msg.invoice_id,
            sender_type: msg.sender_type.to_string(),
            message: msg.message,
            created_at: msg.created_at,
        }).collect();

        Ok(DiscussionResponseDto { messages: dtos })
    }

    /// Get all discussion messages for an invoice (guest access)
    pub async fn execute_as_guest(
        &self,
        invoice_id: Uuid,
    ) -> Result<DiscussionResponseDto, InvoiceError> {
        let messages = self.invoice_service
            .get_discussion_messages_guest(invoice_id)
            .await?;

        let dtos = messages.into_iter().map(|msg| DiscussionMessageDto {
            id: msg.id,
            invoice_id: msg.invoice_id,
            sender_type: msg.sender_type.to_string(),
            message: msg.message,
            created_at: msg.created_at,
        }).collect();

        Ok(DiscussionResponseDto { messages: dtos })
    }
}
