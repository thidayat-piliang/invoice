import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/payment_provider.dart';
import '../../../invoices/presentation/providers/invoice_provider.dart';

class CreatePaymentScreen extends ConsumerStatefulWidget {
  const CreatePaymentScreen({super.key});

  @override
  ConsumerState<CreatePaymentScreen> createState() => _CreatePaymentScreenState();
}

class _CreatePaymentScreenState extends ConsumerState<CreatePaymentScreen> {
  final _formKey = GlobalKey<FormState>();
  String? _selectedInvoiceId;
  String? _selectedInvoiceNumber;
  double? _selectedInvoiceAmount;
  final _amountController = TextEditingController();
  final _methodController = TextEditingController(text: 'Cash');
  final _notesController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(invoiceProvider.notifier).loadInvoices(status: 'unpaid');
    });
  }

  @override
  void dispose() {
    _amountController.dispose();
    _methodController.dispose();
    _notesController.dispose();
    super.dispose();
  }

  void _showInvoiceSelector() {
    final invoiceState = ref.read(invoiceProvider);

    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        height: 400,
        child: Column(
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Select Invoice',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
            ),
            Expanded(
              child: invoiceState.invoices.isEmpty
                  ? const Center(
                      child: Text(
                        'No unpaid invoices available',
                        textAlign: TextAlign.center,
                        style: TextStyle(color: Colors.grey),
                      ),
                    )
                  : ListView.builder(
                      itemCount: invoiceState.invoices.length,
                      itemBuilder: (context, index) {
                        final invoice = invoiceState.invoices[index];
                        return ListTile(
                          title: Text(invoice.invoiceNumber),
                          subtitle: Text(
                            '${invoice.clientName} - \$${invoice.balanceDue.toStringAsFixed(2)}',
                          ),
                          onTap: () {
                            setState(() {
                              _selectedInvoiceId = invoice.id;
                              _selectedInvoiceNumber = invoice.invoiceNumber;
                              _selectedInvoiceAmount = invoice.balanceDue;
                              _amountController.text = invoice.balanceDue.toStringAsFixed(2);
                            });
                            context.pop();
                          },
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _createPayment() async {
    if (!_formKey.currentState!.validate()) return;
    if (_selectedInvoiceId == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please select an invoice'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final amount = double.tryParse(_amountController.text) ?? 0.0;

    if (amount <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Amount must be greater than 0'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    if (_selectedInvoiceAmount != null && amount > _selectedInvoiceAmount!) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Amount exceeds balance due'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final data = {
      'invoice_id': _selectedInvoiceId,
      'amount': amount,
      'payment_method': _methodController.text,
      'notes': _notesController.text.isEmpty ? null : _notesController.text,
    };

    final success = await ref.read(paymentProvider.notifier).createPayment(data);

    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Payment recorded successfully!'),
          backgroundColor: Colors.green,
        ),
      );
      context.go('/payments');
    } else if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ref.read(paymentProvider).error ?? 'Failed to record payment'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final paymentState = ref.watch(paymentProvider);
    final invoiceState = ref.watch(invoiceProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Record Payment'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/payments'),
        ),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Invoice Selection
              const Text(
                'Invoice',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              InkWell(
                onTap: _showInvoiceSelector,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 16),
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.grey.shade300),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        _selectedInvoiceNumber ?? 'Select an invoice',
                        style: TextStyle(
                          color: _selectedInvoiceNumber != null ? Colors.black : Colors.grey,
                        ),
                      ),
                      const Icon(Icons.arrow_drop_down),
                    ],
                  ),
                ),
              ),
              if (invoiceState.invoices.isEmpty) ...[
                const SizedBox(height: 8),
                Row(
                  children: [
                    const Text(
                      'No unpaid invoices. ',
                      style: TextStyle(color: Colors.grey, fontSize: 12),
                    ),
                    GestureDetector(
                      onTap: () => context.go('/invoices'),
                      child: Text(
                        'View invoices',
                        style: TextStyle(
                          color: Colors.blue.shade700,
                          fontWeight: FontWeight.w600,
                          fontSize: 12,
                        ),
                      ),
                    ),
                  ],
                ),
              ],
              const SizedBox(height: 24),

              // Payment Amount
              const Text(
                'Payment Details',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Amount *',
                controller: _amountController,
                keyboardType: TextInputType.number,
                prefixText: '\$',
                required: true,
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Payment Method *',
                controller: _methodController,
                hint: 'Cash, Credit Card, Bank Transfer, etc.',
                required: true,
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Notes (Optional)',
                controller: _notesController,
                hint: 'Additional notes',
                maxLines: 3,
              ),
              const SizedBox(height: 32),

              // Actions
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton(
                      onPressed: () => context.go('/payments'),
                      style: OutlinedButton.styleFrom(
                        minimumSize: const Size(double.infinity, 48),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(8),
                        ),
                      ),
                      child: const Text('Cancel'),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: ElevatedButton(
                      onPressed: paymentState.isLoading ? null : _createPayment,
                      style: ElevatedButton.styleFrom(
                        minimumSize: const Size(double.infinity, 48),
                        backgroundColor: Colors.green,
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(8),
                        ),
                      ),
                      child: paymentState.isLoading
                          ? const SizedBox(
                              width: 20,
                              height: 20,
                              child: CircularProgressIndicator(
                                strokeWidth: 2,
                                valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                              ),
                            )
                          : const Text(
                              'Record Payment',
                              style: TextStyle(
                                fontSize: 16,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildTextField({
    required String label,
    required TextEditingController controller,
    String? hint,
    String? prefixText,
    TextInputType keyboardType = TextInputType.text,
    int maxLines = 1,
    bool required = false,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w500,
          ),
        ),
        const SizedBox(height: 4),
        TextFormField(
          controller: controller,
          decoration: InputDecoration(
            hintText: hint,
            prefixText: prefixText,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
            contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
          ),
          keyboardType: keyboardType,
          maxLines: maxLines,
          validator: (value) {
            if (required && (value == null || value.isEmpty)) {
              return 'This field is required';
            }
            if (label.contains('Amount')) {
              if (value != null && value.isNotEmpty && double.tryParse(value) == null) {
                return 'Invalid amount';
              }
            }
            return null;
          },
        ),
      ],
    );
  }
}
