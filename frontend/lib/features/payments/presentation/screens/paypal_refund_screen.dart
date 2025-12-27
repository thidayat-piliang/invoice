import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/paypal_provider.dart';
import '../../../../shared/widgets/buttons/primary_button.dart';
import '../../../../shared/widgets/inputs/app_text_field.dart';

/// PayPal Refund Screen
class PayPalRefundScreen extends ConsumerStatefulWidget {
  final String orderId;
  final double maxAmount;

  const PayPalRefundScreen({
    super.key,
    required this.orderId,
    required this.maxAmount,
  });

  @override
  ConsumerState<PayPalRefundScreen> createState() => _PayPalRefundScreenState();
}

class _PayPalRefundScreenState extends ConsumerState<PayPalRefundScreen> {
  final _formKey = GlobalKey<FormState>();
  final _amountController = TextEditingController();
  final _reasonController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _amountController.text = widget.maxAmount.toStringAsFixed(2);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(paypalProvider.notifier).clearError();
    });
  }

  @override
  void dispose() {
    _amountController.dispose();
    _reasonController.dispose();
    super.dispose();
  }

  Future<void> _handleRefund() async {
    if (!_formKey.currentState!.validate()) return;

    final amount = double.tryParse(_amountController.text);
    if (amount == null || amount <= 0 || amount > widget.maxAmount) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invalid refund amount'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final paypalNotifier = ref.read(paypalProvider.notifier);
    final refund = await paypalNotifier.refundOrder(
      orderId: widget.orderId,
      amount: amount,
      reason: _reasonController.text.isNotEmpty ? _reasonController.text : null,
    );

    if (refund != null && mounted) {
      _showSuccessDialog(refund);
    }
  }

  void _showSuccessDialog(PayPalRefund refund) {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: const Icon(Icons.check_circle, color: Colors.green, size: 48),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text(
              'Refund Successful!',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 12),
            _buildInfoRow('Refund ID', refund.id),
            _buildInfoRow('Amount', '\$${refund.amount.toStringAsFixed(2)} ${refund.currency}'),
            _buildInfoRow('Status', refund.status),
            if (refund.reason != null) ...[
              const SizedBox(height: 8),
              Text(
                'Reason: ${refund.reason}',
                style: const TextStyle(fontSize: 12, color: Colors.grey),
                textAlign: TextAlign.center,
              ),
            ],
          ],
        ),
        actions: [
          TextButton(
            onPressed: () {
              context.go('/payments');
            },
            child: const Text('Done'),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(fontSize: 12, color: Colors.grey)),
          Text(
            value,
            style: const TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final paypalState = ref.watch(paypalProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('PayPal Refund'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/payments'),
        ),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Order Info
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Order Details',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text('Order ID:', style: TextStyle(color: Colors.grey)),
                          Text(
                            widget.orderId,
                            style: const TextStyle(
                              fontWeight: FontWeight.w600,
                              fontSize: 12,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text('Max Refund:', style: TextStyle(color: Colors.grey)),
                          Text(
                            '\$${widget.maxAmount.toStringAsFixed(2)}',
                            style: const TextStyle(
                              fontWeight: FontWeight.bold,
                              color: Color(0xFF4361EE),
                              fontSize: 16,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ),

              const SizedBox(height: 24),

              // Refund Amount
              AppTextField(
                label: 'Refund Amount',
                hint: 'Enter amount to refund',
                controller: _amountController,
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                validator: (value) {
                  if (value == null || value.isEmpty) {
                    return 'Please enter an amount';
                  }
                  final amount = double.tryParse(value);
                  if (amount == null) {
                    return 'Invalid number format';
                  }
                  if (amount <= 0) {
                    return 'Amount must be greater than 0';
                  }
                  if (amount > widget.maxAmount) {
                    return 'Cannot refund more than \$${widget.maxAmount.toStringAsFixed(2)}';
                  }
                  return null;
                },
              ),

              const SizedBox(height: 16),

              // Refund Reason
              AppTextField(
                label: 'Refund Reason (Optional)',
                hint: 'Why are you refunding this payment?',
                controller: _reasonController,
                maxLines: 3,
              ),

              const SizedBox(height: 24),

              // Error Message
              if (paypalState.error != null)
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.red.shade50,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.red.shade200),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.error_outline, color: Colors.red.shade700),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          paypalState.error!,
                          style: TextStyle(color: Colors.red.shade700),
                        ),
                      ),
                      IconButton(
                        icon: Icon(Icons.close, color: Colors.red.shade700, size: 18),
                        onPressed: () => ref.read(paypalProvider.notifier).clearError(),
                      ),
                    ],
                  ),
                ),

              const SizedBox(height: 24),

              // Action Buttons
              if (paypalState.isLoading)
                const Center(child: CircularProgressIndicator())
              else
                Column(
                  children: [
                    PrimaryButton(
                      text: 'Process Refund',
                      onPressed: _handleRefund,
                      icon: Icons.refresh,
                      backgroundColor: Colors.red,
                    ),
                    const SizedBox(height: 12),
                    OutlinedButton.icon(
                      onPressed: () => context.go('/payments'),
                      icon: const Icon(Icons.cancel),
                      label: const Text('Cancel'),
                    ),
                  ],
                ),

              const SizedBox(height: 24),

              // Warning Box
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.orange.shade50,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.orange.shade200),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(Icons.warning_amber, color: Colors.orange.shade700),
                        const SizedBox(width: 8),
                        const Text(
                          'Important',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Colors.orange,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Text(
                      '• Refunds are processed immediately and cannot be undone\n'
                      '• Partial refunds are allowed up to the original payment amount\n'
                      '• The refund will be processed through PayPal\n'
                      '• Transaction fees may not be refundable',
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.orange.shade800,
                        height: 1.5,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
