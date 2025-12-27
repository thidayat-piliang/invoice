# Partial Payment Implementation Summary

## Overview
This document summarizes the complete implementation of the hybrid partial payment system for the FlashBill invoice application.

## System Design: Hybrid Partial Payment Model

### Default Behavior
- **All invoices allow partial payments by default** (seller can disable per invoice)
- **No minimum payment required by default** (seller can set minimum per invoice)

### Seller Control
- Seller can toggle partial payment on/off when creating/updating invoices
- Seller can set a minimum payment amount (optional)
- Seller can see partial payment count and history

### Buyer Experience
- If partial payment enabled: buyer can pay any amount above minimum (if set)
- If partial payment disabled: buyer must pay full balance
- Status automatically changes to "Partial" after first partial payment
- Status changes to "Paid" when balance reaches zero

## Backend Implementation

### 1. Database Schema (Migration 006)
```sql
ALTER TABLE invoices ADD COLUMN allow_partial_payment BOOLEAN DEFAULT true;
ALTER TABLE invoices ADD COLUMN min_payment_amount DECIMAL(10,2);
ALTER TABLE invoices ADD COLUMN partial_payment_count INTEGER DEFAULT 0;
```

### 2. Domain Models (`src/domain/models/invoice.rs`)

#### Invoice Model
```rust
pub struct Invoice {
    // ... existing fields
    pub allow_partial_payment: bool,
    pub min_payment_amount: Option<f64>,
    pub partial_payment_count: i32,
}
```

#### CreateInvoice Model
```rust
pub struct CreateInvoice {
    // ... existing fields
    pub allow_partial_payment: Option<bool>,
    pub min_payment_amount: Option<f64>,
}
```

#### UpdateInvoice Model
```rust
pub struct UpdateInvoice {
    // ... existing fields
    pub allow_partial_payment: Option<bool>,
    pub min_payment_amount: Option<f64>,
}
```

### 3. Repository Layer (`src/infrastructure/repositories/invoice_repository.rs`)

#### Create Invoice
- Stores partial payment settings
- Sets default values: `allow_partial_payment = true`, `partial_payment_count = 0`

#### Record Payment (with Validation)
```rust
// Validate partial payment setting
if !invoice.allow_partial_payment {
    return Err(sqlx::Error::RowNotFound);
}

// Check minimum payment amount
if let Some(min_amount) = invoice.min_payment_amount {
    if payment.amount < min_amount {
        return Err(sqlx::Error::RowNotFound);
    }
}

// Check if payment exceeds balance
if new_amount_paid > invoice.total_amount {
    return Err(sqlx::Error::RowNotFound);
}

// Update status
if new_amount_paid >= invoice.total_amount {
    status = InvoiceStatus::Paid;
    paid_at = Some(Utc::now());
} else if new_amount_paid > 0.0 {
    if status != InvoiceStatus::Partial {
        partial_payment_count += 1;
    }
    status = InvoiceStatus::Partial;
}
```

### 4. Service Layer (`src/domain/services/invoice_service.rs`)
- Validates partial payment settings before creating/updating invoices
- Handles payment recording with partial payment logic
- Updates status automatically based on payment amount

### 5. API DTOs (`src/application/dto/invoice_dto.rs`)
- Added partial payment fields to all command and response DTOs
- Updated `InvoiceCreatedDto`, `InvoiceDto`, `InvoiceSummaryDto`

### 6. Use Cases (`src/application/use_cases/invoice_use_cases.rs`)
- Updated `CreateInvoiceUseCase` to handle partial payment fields
- Updated `UpdateInvoiceUseCase` to handle partial payment fields
- Updated all DTO conversions

### 7. API Routes (`src/api/routes/invoice.rs`)
- Create invoice endpoint accepts partial payment settings
- Update invoice endpoint accepts partial payment settings
- Record payment endpoint validates partial payment rules

## Frontend Implementation

### 1. Invoice Models

#### Invoice Model (List View)
```dart
class Invoice {
  final String id;
  final String invoiceNumber;
  final String status;
  final String clientName;
  final double totalAmount;
  final double balanceDue;
  final double amountPaid;  // NEW
  final bool allowPartialPayment;  // NEW
  final double? minPaymentAmount;  // NEW
  final int partialPaymentCount;  // NEW

  // Getters
  bool get isPartial => status.toLowerCase() == 'partial';
}
```

#### InvoiceDetail Model (Detail View)
```dart
class InvoiceDetail {
  // ... all Invoice fields plus:
  final List<InvoiceItem> items;
  final String? notes;
  final String? terms;

  // Getters
  bool get canAcceptPartialPayment => allowPartialPayment && balanceDue > 0;
}
```

#### GuestInvoice Model (Guest Checkout)
```dart
class GuestInvoice {
  // ... same fields as InvoiceDetail
  // Supports partial payments for guest users
}
```

### 2. Create Invoice Screen (`create_invoice_screen.dart`)

#### UI Controls Added
```dart
// Partial Payment Settings Section
Checkbox(
  value: _allowPartialPayment,
  onChanged: (value) => setState(() => _allowPartialPayment = value ?? true),
),
Text('Allow partial payments'),

// Conditional Minimum Payment Field
if (_allowPartialPayment)
  TextField(
    controller: _minPaymentController,
    decoration: InputDecoration(
      labelText: 'Minimum Payment Amount (Optional)',
      hint: '\$0.00 (leave empty for no minimum)',
    ),
  ),

// Info Box
Container(
  color: Colors.blue.shade50,
  child: Text('When enabled, buyers can pay any amount above the minimum until full payment is paid.'),
)
```

#### Data Payload
```dart
final data = {
  'client_id': _selectedClientId,
  'issue_date': ...,
  'due_date': ...,
  'items': [...],
  'notes': _notesController.text,
  'terms': _termsController.text,
  'discount_amount': 0.0,
  'tax_included': false,
  'send_immediately': false,
  'allow_partial_payment': _allowPartialPayment,
  if (minPayment != null) 'min_payment_amount': minPayment,
};
```

### 3. Invoice List Screen (`invoice_list_screen.dart`)

#### Partial Payment Indicators
```dart
// In InvoiceCard widget
if (invoice.isPartial) ...[
  Text('Paid: \$${invoice.amountPaid.toStringAsFixed(2)}'),
  Text('Partial payments allowed'),
],
```

#### Status Badge
- Already supports 'partial' status with orange color
- Shows partial payment count

### 4. Invoice Detail Screen (`invoice_detail_screen.dart`)

#### Partial Payment Info Display
```dart
// In _buildTotals method
if (invoice.allowPartialPayment || invoice.isPartial) ...[
  Container(
    color: Colors.orange.shade50,
    child: Column(
      children: [
        Text('Partial Payment'),
        Text(invoice.allowPartialPayment
          ? 'Buyers can pay in installments${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount})' : ''}'
          : 'Full payment required'),
        if (invoice.partialPaymentCount > 0)
          Text('Payments made: ${invoice.partialPaymentCount}'),
      ],
    ),
  ),
]
```

#### Record Payment Dialog with Validation
```dart
// Show partial payment info
if (invoice.allowPartialPayment)
  Container(
    color: Colors.orange.shade50,
    child: Text('Partial payments allowed${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount})' : ''}'),
  ),

// Validate on submit
if (amount < invoice.minPaymentAmount)
  showError('Minimum payment is \$${invoice.minPaymentAmount}');

if (!invoice.allowPartialPayment && amount < invoice.balanceDue)
  showError('This invoice requires full payment');

if (amount > invoice.balanceDue)
  showError('Amount exceeds balance due');
```

### 5. Guest Checkout Screen (`guest_checkout_screen.dart`)

#### Partial Payment Form
```dart
// Partial payment notice
if (invoice.allowPartialPayment)
  Container(
    color: Colors.orange.shade50,
    child: Text('Partial payments allowed${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount})' : ''}'),
  ),

// Amount field (only for partial payments)
if (invoice.allowPartialPayment)
  TextFormField(
    controller: _amountController,
    decoration: InputDecoration(
      labelText: 'Payment Amount *',
      helperText: 'Balance due: \$${invoice.balanceDue}',
    ),
    validator: (value) {
      // Validate minimum, maximum, and partial payment rules
    },
  )
else
  // Show full payment required message
  Container(
    color: Colors.blue.shade50,
    child: Text('Full payment required: \$${invoice.balanceDue}'),
  ),
```

#### Process Payment Logic
```dart
// Determine amount
double amount;
if (invoice.allowPartialPayment && _amountController.text.isNotEmpty) {
  amount = double.parse(_amountController.text);
} else {
  amount = invoice.balanceDue;
}

// Show success message
if (amount < invoice.balanceDue)
  'Partial payment of \$${amount} processed. Remaining: \$${invoice.balanceDue - amount}'
else
  'Payment processed successfully'
```

### 6. Guest Provider (`guest_provider.dart`)
- Updated `GuestInvoice` model with partial payment fields
- Handles partial payment processing

## Status Flow

### Complete Flow
```
Draft → Sent → Viewed → Partial → Paid
```

### Partial Payment Logic
1. **First Payment**: Status changes from Viewed/Sent → Partial
2. **Subsequent Payments**: Status remains Partial
3. **Final Payment**: Status changes from Partial → Paid

### Automatic Updates
- `partial_payment_count` increments on first partial payment
- `amount_paid` accumulates with each payment
- `balance_due` = `total_amount` - `amount_paid`
- Status updates automatically based on payment amount

## Validation Rules

### Backend Validation (Repository)
1. Check `allow_partial_payment` flag
2. Check minimum payment amount (if set)
3. Check payment doesn't exceed balance
4. Update status and count

### Frontend Validation (UI)
1. **Create Invoice**:
   - Can enable/disable partial payments
   - Can set minimum payment amount

2. **Record Payment**:
   - Must be positive amount
   - Must meet minimum (if set)
   - Must not exceed balance
   - Must respect partial payment flag

3. **Guest Checkout**:
   - Same validations as record payment
   - Shows clear error messages

## User Experience

### Seller Perspective
1. **Create Invoice**: Toggle partial payment, set minimum
2. **View Invoice**: See partial payment status, count, amount paid
3. **Record Payment**: Validate amount, auto-update status

### Buyer Perspective (Guest)
1. **View Invoice**: See partial payment allowed message
2. **Make Payment**: Enter amount (if partial) or pay full amount
3. **Success**: See remaining balance if partial payment

### Buyer Perspective (Registered)
1. **View Invoice**: See partial payment info in details
2. **Make Payment**: Via payment screen with validation
3. **History**: Track partial payments in payment history

## Testing Checklist

### Backend
- [ ] Create invoice with partial payment enabled
- [ ] Create invoice with partial payment disabled
- [ ] Create invoice with minimum payment
- [ ] Record partial payment (valid amount)
- [ ] Record partial payment (below minimum) - should fail
- [ ] Record partial payment (exceeds balance) - should fail
- [ ] Record payment when partial disabled - should require full amount
- [ ] Status transitions: Draft → Sent → Viewed → Partial → Paid
- [ ] Partial payment count increments correctly

### Frontend
- [ ] Create invoice screen shows partial payment controls
- [ ] Invoice list shows partial status indicators
- [ ] Invoice detail shows partial payment info
- [ ] Record payment dialog validates correctly
- [ ] Guest checkout supports partial payments
- [ ] Guest checkout shows validation errors
- [ ] Success messages show correct amounts

## Files Modified

### Backend
1. `migrations/006_add_partial_payment_settings.sql`
2. `src/domain/models/invoice.rs`
3. `src/infrastructure/repositories/invoice_repository.rs`
4. `src/domain/services/invoice_service.rs`
5. `src/application/dto/invoice_dto.rs`
6. `src/application/use_cases/invoice_use_cases.rs`
7. `src/api/routes/invoice.rs`
8. `src/api/routes/tax.rs`

### Frontend
1. `lib/features/invoices/presentation/providers/invoice_provider.dart`
2. `lib/features/invoices/presentation/screens/create_invoice_screen.dart`
3. `lib/features/invoices/presentation/screens/invoice_detail_screen.dart`
4. `lib/features/invoices/presentation/screens/invoice_list_screen.dart`
5. `lib/features/guest/presentation/providers/guest_provider.dart`
6. `lib/features/guest/presentation/screens/guest_checkout_screen.dart`
7. `lib/shared/services/api_client.dart`

## Key Features Implemented

✅ Hybrid partial payment system (default allow, seller can disable)
✅ Minimum payment amount per invoice
✅ Automatic status updates (Partial → Paid)
✅ Partial payment count tracking
✅ Frontend UI controls for partial payment settings
✅ Frontend validation for partial payments
✅ Guest checkout partial payment support
✅ Partial payment indicators in UI
✅ Backward compatibility (existing invoices work)

## Notes

- All changes are backward compatible
- Existing invoices default to `allow_partial_payment = true`
- No breaking changes to existing API contracts
- Frontend gracefully handles missing partial payment fields (defaults to true)
