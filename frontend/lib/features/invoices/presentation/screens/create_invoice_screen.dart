import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/invoice_provider.dart';
import '../../../shared/widgets/buttons/primary_button.dart';
import '../../../shared/widgets/inputs/app_text_field.dart';

class CreateInvoiceScreen extends ConsumerStatefulWidget {
  const CreateInvoiceScreen({super.key});

  @override
  ConsumerState<CreateInvoiceScreen> createState() => _CreateInvoiceScreenState();
}

class _CreateInvoiceScreenState extends ConsumerState<CreateInvoiceScreen> {
  final _formKey = GlobalKey<FormState>();
  final _clientNameController = TextEditingController();
  final _clientEmailController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _quantityController = TextEditingController(text: '1');
  final _priceController = TextEditingController();
  final _notesController = TextEditingController();
  final _termsController = TextEditingController(text: 'Payment due within 30 days');

  DateTime _issueDate = DateTime.now();
  DateTime _dueDate = DateTime.now().add(const Duration(days: 30));

  @override
  void dispose() {
    _clientNameController.dispose();
    _clientEmailController.dispose();
    _descriptionController.dispose();
    _quantityController.dispose();
    _priceController.dispose();
    _notesController.dispose();
    _termsController.dispose();
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

  Future<void> _createInvoice() async {
    if (!_formKey.currentState!.validate()) return;

    final quantity = double.tryParse(_quantityController.text) ?? 1.0;
    final price = double.tryParse(_priceController.text) ?? 0.0;
    final taxRate = 0.0; // Default tax rate

    final data = {
      'client_id': '00000000-0000-0000-0000-000000000000', // Placeholder
      'issue_date': _issueDate.toIso8601String().split('T')[0],
      'due_date': _dueDate.toIso8601String().split('T')[0],
      'items': [
        {
          'description': _descriptionController.text,
          'quantity': quantity,
          'unit_price': price,
          'tax_rate': taxRate,
        }
      ],
      'notes': _notesController.text,
      'terms': _termsController.text,
      'tax_included': false,
      'send_immediately': false,
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
              // Client Information
              const Text(
                'Client Information',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              AppTextField(
                label: 'Client Name',
                controller: _clientNameController,
                hint: 'Enter client name',
                validator: (value) {
                  if (value == null || value.isEmpty) {
                    return 'Required';
                  }
                  return null;
                },
              ),
              const SizedBox(height: 12),
              AppTextField(
                label: 'Client Email',
                controller: _clientEmailController,
                keyboardType: TextInputType.emailAddress,
                hint: 'client@example.com',
                validator: (value) {
                  if (value == null || value.isEmpty) {
                    return 'Required';
                  }
                  if (!value.contains('@')) {
                    return 'Invalid email';
                  }
                  return null;
                },
              ),
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
                    child: AppTextField(
                      label: 'Issue Date',
                      controller: TextEditingController(
                        text: '${_issueDate.month}/${_issueDate.day}/${_issueDate.year}',
                      ),
                      readOnly: true,
                      onTap: () => _selectDate(false),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: AppTextField(
                      label: 'Due Date',
                      controller: TextEditingController(
                        text: '${_dueDate.month}/${_dueDate.day}/${_dueDate.year}',
                      ),
                      readOnly: true,
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
              AppTextField(
                label: 'Description',
                controller: _descriptionController,
                hint: 'Product or service description',
                maxLines: 2,
                validator: (value) {
                  if (value == null || value.isEmpty) {
                    return 'Required';
                  }
                  return null;
                },
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: AppTextField(
                      label: 'Quantity',
                      controller: _quantityController,
                      keyboardType: const TextInputType.numberWithOptions(decimal: true),
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return 'Required';
                        }
                        if (double.tryParse(value) == null) {
                          return 'Invalid';
                        }
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: AppTextField(
                      label: 'Unit Price',
                      controller: _priceController,
                      keyboardType: const TextInputType.numberWithOptions(decimal: true),
                      hint: '\$0.00',
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return 'Required';
                        }
                        if (double.tryParse(value) == null) {
                          return 'Invalid';
                        }
                        return null;
                      },
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
              AppTextField(
                label: 'Notes (Optional)',
                controller: _notesController,
                hint: 'Thank you for your business!',
                maxLines: 3,
              ),
              const SizedBox(height: 12),
              AppTextField(
                label: 'Terms (Optional)',
                controller: _termsController,
                hint: 'Payment terms',
                maxLines: 2,
              ),
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
                    child: PrimaryButton(
                      text: 'Create',
                      onPressed: _createInvoice,
                      isLoading: invoiceState.isLoading,
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
}
