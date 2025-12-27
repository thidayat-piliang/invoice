import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/invoice_provider.dart';
import '../../../clients/presentation/providers/client_provider.dart';

class CreateInvoiceScreen extends ConsumerStatefulWidget {
  const CreateInvoiceScreen({super.key});

  @override
  ConsumerState<CreateInvoiceScreen> createState() => _CreateInvoiceScreenState();
}

class _CreateInvoiceScreenState extends ConsumerState<CreateInvoiceScreen> {
  final _formKey = GlobalKey<FormState>();
  String? _selectedClientId;
  String? _selectedClientName;
  final _descriptionController = TextEditingController();
  final _quantityController = TextEditingController(text: '1');
  final _priceController = TextEditingController();
  final _notesController = TextEditingController();
  final _termsController = TextEditingController(text: 'Payment due within 30 days');
  final _minPaymentController = TextEditingController();

  DateTime _issueDate = DateTime.now();
  DateTime _dueDate = DateTime.now().add(const Duration(days: 30));
  bool _allowPartialPayment = true;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(clientProvider.notifier).loadClients();
    });
  }

  @override
  void dispose() {
    _descriptionController.dispose();
    _quantityController.dispose();
    _priceController.dispose();
    _notesController.dispose();
    _termsController.dispose();
    _minPaymentController.dispose();
    super.dispose();
  }

  Future<void> _selectDate(bool isDueDate) async {
    final picked = await showDatePicker(
      context: context,
      initialDate: isDueDate ? _dueDate : _issueDate,
      firstDate: DateTime.now(),
      lastDate: DateTime.now().add(const Duration(days: 365)),
    );

    if (picked != null) {
      setState(() {
        if (isDueDate) {
          _dueDate = picked;
        } else {
          _issueDate = picked;
        }
      });
    }
  }

  void _showClientSelector() {
    final clientState = ref.read(clientProvider);

    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        height: 400,
        child: Column(
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Select Client',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
            ),
            Expanded(
              child: clientState.clients.isEmpty
                  ? const Center(
                      child: Text(
                        'No clients available\nPlease create a client first',
                        textAlign: TextAlign.center,
                        style: TextStyle(color: Colors.grey),
                      ),
                    )
                  : ListView.builder(
                      itemCount: clientState.clients.length,
                      itemBuilder: (context, index) {
                        final client = clientState.clients[index];
                        return ListTile(
                          title: Text(client.name),
                          subtitle: Text(client.email ?? 'No email'),
                          onTap: () {
                            setState(() {
                              _selectedClientId = client.id;
                              _selectedClientName = client.name;
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

  Future<void> _createInvoice() async {
    if (!_formKey.currentState!.validate()) return;
    if (_selectedClientId == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please select a client'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final quantity = double.tryParse(_quantityController.text) ?? 1.0;
    final price = double.tryParse(_priceController.text) ?? 0.0;
    final minPayment = _minPaymentController.text.isNotEmpty
        ? double.tryParse(_minPaymentController.text)
        : null;

    final data = {
      'client_id': _selectedClientId,
      'issue_date': _issueDate.toIso8601String().split('T')[0],
      'due_date': _dueDate.toIso8601String().split('T')[0],
      'items': [
        {
          'description': _descriptionController.text,
          'quantity': quantity,
          'unit_price': price,
          'tax_rate': 0.0,
        }
      ],
      'notes': _notesController.text,
      'terms': _termsController.text,
      'discount_amount': 0.0,
      'tax_included': false,
      'send_immediately': false,
      'allow_partial_payment': _allowPartialPayment,
      if (minPayment != null) 'min_payment_amount': minPayment,
    };

    final success = await ref.read(invoiceProvider.notifier).createInvoice(data);

    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invoice created successfully!'),
          backgroundColor: Colors.green,
        ),
      );
      context.go('/invoices');
    } else if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ref.read(invoiceProvider).error ?? 'Failed to create invoice'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final invoiceState = ref.watch(invoiceProvider);
    final clientState = ref.watch(clientProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Create Invoice'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/invoices'),
        ),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Client Selection
              const Text(
                'Client',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              InkWell(
                onTap: _showClientSelector,
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
                        _selectedClientName ?? 'Select a client',
                        style: TextStyle(
                          color: _selectedClientName != null ? Colors.black : Colors.grey,
                        ),
                      ),
                      const Icon(Icons.arrow_drop_down),
                    ],
                  ),
                ),
              ),
              if (clientState.clients.isEmpty) ...[
                const SizedBox(height: 8),
                Row(
                  children: [
                    const Text(
                      'No clients available. ',
                      style: TextStyle(color: Colors.grey, fontSize: 12),
                    ),
                    GestureDetector(
                      onTap: () => context.go('/clients/create'),
                      child: Text(
                        'Create a client first',
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

              // Dates
              const Text(
                'Dates',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: _buildDateField(
                      label: 'Issue Date',
                      date: _issueDate,
                      onTap: () => _selectDate(false),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildDateField(
                      label: 'Due Date',
                      date: _dueDate,
                      onTap: () => _selectDate(true),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 24),

              // Items
              const Text(
                'Invoice Items',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Description *',
                controller: _descriptionController,
                hint: 'Product or service description',
                maxLines: 2,
                required: true,
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: _buildTextField(
                      label: 'Quantity *',
                      controller: _quantityController,
                      keyboardType: TextInputType.number,
                      required: true,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildTextField(
                      label: 'Unit Price *',
                      controller: _priceController,
                      keyboardType: TextInputType.number,
                      hint: '\$0.00',
                      required: true,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 24),

              // Notes & Terms
              const Text(
                'Additional Info',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Notes (Optional)',
                controller: _notesController,
                hint: 'Thank you for your business!',
                maxLines: 3,
              ),
              const SizedBox(height: 12),
              _buildTextField(
                label: 'Terms (Optional)',
                controller: _termsController,
                hint: 'Payment terms',
                maxLines: 2,
              ),
              const SizedBox(height: 24),

              // Partial Payment Settings
              const Text(
                'Partial Payment Settings',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Checkbox(
                    value: _allowPartialPayment,
                    onChanged: (value) {
                      setState(() {
                        _allowPartialPayment = value ?? true;
                      });
                    },
                  ),
                  const Text('Allow partial payments'),
                ],
              ),
              const SizedBox(height: 8),
              if (_allowPartialPayment) ...[
                _buildTextField(
                  label: 'Minimum Payment Amount (Optional)',
                  controller: _minPaymentController,
                  keyboardType: TextInputType.number,
                  hint: '\$0.00 (leave empty for no minimum)',
                ),
                const SizedBox(height: 8),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.blue.shade50,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.blue.shade200),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.info_outline, color: Colors.blue.shade700, size: 20),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          'When enabled, buyers can pay any amount above the minimum (if set) until the full amount is paid.',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.blue.shade700,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
              const SizedBox(height: 32),

              // Actions
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton(
                      onPressed: () => context.go('/invoices'),
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
                      onPressed: invoiceState.isLoading ? null : _createInvoice,
                      style: ElevatedButton.styleFrom(
                        minimumSize: const Size(double.infinity, 48),
                        backgroundColor: Colors.blue,
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(8),
                        ),
                      ),
                      child: invoiceState.isLoading
                          ? const SizedBox(
                              width: 20,
                              height: 20,
                              child: CircularProgressIndicator(
                                strokeWidth: 2,
                                valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                              ),
                            )
                          : const Text(
                              'Create',
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

  Widget _buildDateField({
    required String label,
    required DateTime date,
    required VoidCallback onTap,
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
        InkWell(
          onTap: onTap,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.grey.shade300),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('${date.month}/${date.day}/${date.year}'),
                const Icon(Icons.calendar_today, size: 18),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildTextField({
    required String label,
    required TextEditingController controller,
    String? hint,
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
            if (label.contains('Price') || label.contains('Quantity')) {
              if (value != null && value.isNotEmpty && double.tryParse(value) == null) {
                return 'Invalid number';
              }
            }
            return null;
          },
        ),
      ],
    );
  }
}
