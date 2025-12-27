import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import '../providers/guest_provider.dart';
import '../../../../shared/widgets/buttons/primary_button.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../shared/widgets/discussion_bottom_sheet.dart';
import '../../../../app/theme/app_theme.dart';

/// Guest Checkout Screen - For non-registered users to view and pay invoices
class GuestCheckoutScreen extends ConsumerStatefulWidget {
  final String token;

  const GuestCheckoutScreen({
    super.key,
    required this.token,
  });

  @override
  ConsumerState<GuestCheckoutScreen> createState() => _GuestCheckoutScreenState();
}

class _GuestCheckoutScreenState extends ConsumerState<GuestCheckoutScreen> {
  final _formKey = GlobalKey<FormState>();
  final _emailController = TextEditingController();
  final _phoneController = TextEditingController();
  final _nameController = TextEditingController();
  final _notesController = TextEditingController();
  final _amountController = TextEditingController();
  String _selectedPaymentMethod = 'PayPal';

  @override
  void initState() {
    super.initState();
    // Load invoice on init
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(guestProvider.notifier).loadGuestInvoice(widget.token);
    });
  }

  @override
  void dispose() {
    _emailController.dispose();
    _phoneController.dispose();
    _nameController.dispose();
    _notesController.dispose();
    _amountController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final guestState = ref.watch(guestProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Guest Checkout'),
        backgroundColor: AppTheme.primaryColor,
        foregroundColor: Colors.white,
      ),
      body: guestState.isLoading && guestState.invoice == null
          ? const Center(child: CircularProgressIndicator())
          : guestState.error != null
              ? Center(
                  child: EmptyState(
                    icon: Icons.error_outline,
                    title: 'Error',
                    message: guestState.error!,
                    onRetry: () {
                      ref.read(guestProvider.notifier).loadGuestInvoice(widget.token);
                    },
                  ),
                )
              : guestState.invoice == null
                  ? const Center(
                      child: EmptyState(
                        icon: Icons.receipt_long,
                        title: 'Invoice Not Found',
                        message: 'The invoice link may be invalid or expired.',
                      ),
                    )
                  : _buildContent(guestState),
    );
  }

  Widget _buildContent(GuestState state) {
    final invoice = state.invoice!;
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Invoice Summary Card
          Card(
            elevation: 2,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Invoice #${invoice.invoiceNumber}',
                        style: theme.textTheme.titleLarge?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      _buildStatusBadge(invoice.status),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Client: ${invoice.clientName}',
                    style: theme.textTheme.bodyLarge,
                  ),
                  if (invoice.clientEmail != null)
                    Text(
                      'Email: ${invoice.clientEmail}',
                      style: theme.textTheme.bodyMedium,
                    ),
                  if (invoice.clientPhone != null)
                    Text(
                      'Phone: ${invoice.clientPhone}',
                      style: theme.textTheme.bodyMedium,
                    ),
                  const SizedBox(height: 12),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Issue Date:', style: theme.textTheme.bodyMedium),
                      Text(
                        DateFormat('MMM dd, yyyy').format(invoice.issueDate),
                        style: theme.textTheme.bodyMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Due Date:', style: theme.textTheme.bodyMedium),
                      Text(
                        DateFormat('MMM dd, yyyy').format(invoice.dueDate),
                        style: theme.textTheme.bodyMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                          color: invoice.isOverdue ? Colors.red : null,
                        ),
                      ),
                    ],
                  ),
                  const Divider(height: 24),
                  // Items
                  ...invoice.items.map((item) => Padding(
                        padding: const EdgeInsets.symmetric(vertical: 4),
                        child: Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Expanded(
                              child: Text(
                                item.description,
                                style: theme.textTheme.bodyMedium,
                              ),
                            ),
                            Text(
                              '${item.quantity} x \$${item.unitPrice.toStringAsFixed(2)}',
                              style: theme.textTheme.bodyMedium,
                            ),
                            Text(
                              '\$${item.total.toStringAsFixed(2)}',
                              style: theme.textTheme.bodyMedium?.copyWith(
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ),
                  )),
                  const Divider(height: 24),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Subtotal:', style: theme.textTheme.bodyMedium),
                      Text(
                        '\$${invoice.subtotal.toStringAsFixed(2)}',
                        style: theme.textTheme.bodyMedium,
                      ),
                    ],
                  ),
                  if (invoice.taxAmount > 0)
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text('Tax:', style: theme.textTheme.bodyMedium),
                        Text(
                          '\$${invoice.taxAmount.toStringAsFixed(2)}',
                          style: theme.textTheme.bodyMedium,
                        ),
                      ],
                    ),
                  if (invoice.discountAmount > 0)
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text('Discount:', style: theme.textTheme.bodyMedium),
                        Text(
                          '-\$${invoice.discountAmount.toStringAsFixed(2)}',
                          style: theme.textTheme.bodyMedium?.copyWith(color: Colors.green),
                        ),
                      ],
                    ),
                  const SizedBox(height: 8),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Total Amount:',
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      Text(
                        '\$${invoice.totalAmount.toStringAsFixed(2)}',
                        style: theme.textTheme.titleLarge?.copyWith(
                          fontWeight: FontWeight.bold,
                          color: AppTheme.primaryColor,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),

          const SizedBox(height: 20),

          // Payment Form
          if (invoice.status != 'paid') ...[
            Card(
              elevation: 2,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Form(
                  key: _formKey,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Payment Information',
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 16),
                      // Partial Payment Notice
                      if (invoice.allowPartialPayment) ...[
                        Container(
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: Colors.orange.shade50,
                            borderRadius: BorderRadius.circular(6),
                            border: Border.all(color: Colors.orange.shade200),
                          ),
                          child: Row(
                            children: [
                              Icon(Icons.info_outline, color: Colors.orange.shade700, size: 18),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  'Partial payments allowed${invoice.minPaymentAmount != null ? ' (min: \$${invoice.minPaymentAmount!.toStringAsFixed(2)})' : ''}',
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Colors.orange.shade700,
                                    fontWeight: FontWeight.w600,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 12),
                      ],
                      TextFormField(
                        controller: _nameController,
                        decoration: const InputDecoration(
                          labelText: 'Your Name *',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.person),
                        ),
                        validator: (value) {
                          if (value == null || value.isEmpty) {
                            return 'Please enter your name';
                          }
                          return null;
                        },
                      ),
                      const SizedBox(height: 12),
                      TextFormField(
                        controller: _emailController,
                        decoration: const InputDecoration(
                          labelText: 'Email (for receipt)',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.email),
                        ),
                        keyboardType: TextInputType.emailAddress,
                      ),
                      const SizedBox(height: 12),
                      TextFormField(
                        controller: _phoneController,
                        decoration: const InputDecoration(
                          labelText: 'Phone (for WhatsApp)',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.phone),
                        ),
                        keyboardType: TextInputType.phone,
                      ),
                      const SizedBox(height: 12),
                      // Amount Field (for partial payments)
                      if (invoice.allowPartialPayment) ...[
                        TextFormField(
                          controller: _amountController,
                          decoration: InputDecoration(
                            labelText: 'Payment Amount *',
                            border: const OutlineInputBorder(),
                            prefixIcon: const Icon(Icons.attach_money),
                            helperText: 'Balance due: \$${invoice.balanceDue.toStringAsFixed(2)}',
                          ),
                          keyboardType: TextInputType.number,
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return 'Please enter amount';
                            }
                            final amount = double.tryParse(value);
                            if (amount == null || amount <= 0) {
                              return 'Invalid amount';
                            }
                            if (invoice.minPaymentAmount != null && amount < invoice.minPaymentAmount!) {
                              return 'Minimum: \$${invoice.minPaymentAmount!.toStringAsFixed(2)}';
                            }
                            if (amount > invoice.balanceDue) {
                              return 'Maximum: \$${invoice.balanceDue.toStringAsFixed(2)}';
                            }
                            return null;
                          },
                        ),
                        const SizedBox(height: 12),
                      ] else ...[
                        // Full payment only
                        Container(
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: Colors.blue.shade50,
                            borderRadius: BorderRadius.circular(6),
                            border: Border.all(color: Colors.blue.shade200),
                          ),
                          child: Row(
                            children: [
                              Icon(Icons.lock, color: Colors.blue.shade700, size: 18),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  'Full payment required: \$${invoice.balanceDue.toStringAsFixed(2)}',
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Colors.blue.shade700,
                                    fontWeight: FontWeight.w600,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 12),
                      ],
                      DropdownButtonFormField<String>(
                        value: _selectedPaymentMethod,
                        decoration: const InputDecoration(
                          labelText: 'Payment Method *',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.payment),
                        ),
                        items: const [
                          DropdownMenuItem(value: 'PayPal', child: Text('PayPal')),
                          DropdownMenuItem(value: 'Stripe', child: Text('Stripe')),
                          DropdownMenuItem(value: 'AchDebit', child: Text('ACH Debit')),
                          DropdownMenuItem(value: 'BankTransfer', child: Text('Bank Transfer')),
                        ],
                        onChanged: (value) {
                          setState(() {
                            _selectedPaymentMethod = value!;
                          });
                        },
                        validator: (value) {
                          if (value == null || value.isEmpty) {
                            return 'Please select a payment method';
                          }
                          return null;
                        },
                      ),
                      const SizedBox(height: 12),
                      TextFormField(
                        controller: _notesController,
                        decoration: const InputDecoration(
                          labelText: 'Notes (optional)',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.note),
                        ),
                        maxLines: 2,
                      ),
                    ],
                  ),
                ),
              ),
            ),

            const SizedBox(height: 20),

            // Action Buttons
            PrimaryButton(
              text: 'Pay Now',
              onPressed: state.isLoading ? null : () => _processPayment(),
              isLoading: state.isLoading,
              fullWidth: true,
            ),

            const SizedBox(height: 12),

            // Send Payment Link via WhatsApp
            if (invoice.clientPhone != null)
              OutlinedButton.icon(
                onPressed: state.isLoading
                    ? null
                    : () async {
                        final result = await ref
                            .read(guestProvider.notifier)
                            .sendGuestPaymentLink(widget.token);
                        if (result && mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Payment link sent via WhatsApp!'),
                              backgroundColor: Colors.green,
                            ),
                          );
                        }
                      },
                icon: const Icon(Icons.whatsapp),
                label: const Text('Send Payment Link via WhatsApp'),
                style: OutlinedButton.styleFrom(
                  foregroundColor: Colors.green,
                  side: const BorderSide(color: Colors.green),
                ),
              ),

            const SizedBox(height: 12),

            // Discussion Button
            OutlinedButton.icon(
              onPressed: () {
                showDiscussionBottomSheet(
                  context,
                  invoiceId: invoice.id,
                  guestToken: widget.token,
                  clientName: invoice.clientName,
                );
              },
              icon: const Icon(Icons.chat),
              label: const Text('Discussion / Ask Question'),
              style: OutlinedButton.styleFrom(
                foregroundColor: Colors.blue,
                side: const BorderSide(color: Colors.blue),
              ),
            ),
          ] else ...[
            // Payment Already Completed
            Card(
              color: Colors.green.shade50,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  children: [
                    const Icon(Icons.check_circle, color: Colors.green, size: 48),
                    const SizedBox(height: 8),
                    Text(
                      'Payment Completed',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                        color: Colors.green.shade800,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'This invoice has been paid. Thank you!',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: Colors.green.shade800,
                      ),
                      textAlign: TextAlign.center,
                    ),
                  ],
                ),
              ),
            ),
          ],

          const SizedBox(height: 20),

          // Payment History Section
          if (state.paymentHistory.isNotEmpty) ...[
            Card(
              elevation: 2,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Your Payment History',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Showing last 5 payments',
                      style: theme.textTheme.bodySmall,
                    ),
                    const SizedBox(height: 12),
                    ...state.paymentHistory.take(5).map((payment) => Padding(
                          padding: const EdgeInsets.symmetric(vertical: 6),
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Text(
                                '#${payment.invoiceNumber}',
                                style: theme.textTheme.bodyMedium,
                              ),
                              Text(
                                '\$${payment.amount.toStringAsFixed(2)}',
                                style: theme.textTheme.bodyMedium?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                            ],
                          ),
                    )),
                  ],
                ),
              ),
            ),
          ],

          const SizedBox(height: 20),

          // App Download Promotion
          Card(
            color: AppTheme.primaryColor.withOpacity(0.1),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  const Icon(Icons.download, color: AppTheme.primaryColor, size: 32),
                  const SizedBox(height: 8),
                  Text(
                    'Want to Track All Your Payments?',
                    style: theme.textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                      color: AppTheme.primaryColor,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Download the FlashBill app to:\n• View all payment history\n• Get payment reminders\n• Manage invoices easily',
                    style: theme.textTheme.bodyMedium,
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 12),
                  OutlinedButton(
                    onPressed: () {
                      // TODO: Open app store
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('App download coming soon!')),
                      );
                    },
                    child: const Text('Download App'),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusBadge(String status) {
    Color color;
    IconData icon;

    switch (status.toLowerCase()) {
      case 'paid':
        color = Colors.green;
        icon = Icons.check_circle;
        break;
      case 'overdue':
        color = Colors.red;
        icon = Icons.warning;
        break;
      case 'viewed':
        color = Colors.blue;
        icon = Icons.visibility;
        break;
      case 'sent':
        color = Colors.orange;
        icon = Icons.send;
        break;
      default:
        color = Colors.grey;
        icon = Icons.circle;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 16, color: color),
          const SizedBox(width: 4),
          Text(
            status.toUpperCase(),
            style: TextStyle(
              color: color,
              fontWeight: FontWeight.bold,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _processPayment() async {
    if (!_formKey.currentState!.validate()) {
      return;
    }

    final invoice = ref.read(guestProvider).invoice;
    if (invoice == null) return;

    // Determine amount: use entered amount for partial payments, or full balance
    double amount;
    if (invoice.allowPartialPayment && _amountController.text.isNotEmpty) {
      amount = double.parse(_amountController.text);
    } else {
      amount = invoice.balanceDue;
    }

    final data = {
      'amount': amount,
      'payment_method': _selectedPaymentMethod,
      'customer_email': _emailController.text.trim(),
      'customer_phone': _phoneController.text.trim(),
      'customer_name': _nameController.text.trim(),
      'notes': _notesController.text.trim(),
    };

    final result = await ref.read(guestProvider.notifier).processPayment(widget.token, data);

    if (result && mounted) {
      // Show success dialog
      showDialog(
        context: context,
        barrierDismissible: false,
        builder: (context) => AlertDialog(
          title: const Row(
            children: [
              Icon(Icons.check_circle, color: Colors.green),
              SizedBox(width: 8),
              Text('Payment Successful'),
            ],
          ),
          content: Text(
            amount < invoice.balanceDue
                ? 'Partial payment of \$${amount.toStringAsFixed(2)} has been processed. Remaining balance: \$${(invoice.balanceDue - amount).toStringAsFixed(2)}'
                : 'Your payment has been processed successfully. A confirmation email will be sent shortly.',
          ),
          actions: [
            TextButton(
              onPressed: () {
                Navigator.of(context).pop();
                // Refresh the invoice
                ref.read(guestProvider.notifier).loadGuestInvoice(widget.token);
                // Clear amount field for partial payments
                if (invoice.allowPartialPayment) {
                  _amountController.clear();
                }
              },
              child: const Text('OK'),
            ),
          ],
        ),
      );
    }
  }
}
