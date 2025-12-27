import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/expense_provider.dart';
import '../../../../shared/services/image_picker_service.dart';

class ExpenseFormScreen extends ConsumerStatefulWidget {
  final String? expenseId;

  const ExpenseFormScreen({super.key, this.expenseId});

  @override
  ConsumerState<ExpenseFormScreen> createState() => _ExpenseFormScreenState();
}

class _ExpenseFormScreenState extends ConsumerState<ExpenseFormScreen> {
  final _formKey = GlobalKey<FormState>();
  final _descriptionController = TextEditingController();
  final _amountController = TextEditingController();
  final _categoryController = TextEditingController(text: 'Office');
  final _notesController = TextEditingController();
  final ImagePickerService _imagePicker = ImagePickerService();
  bool _isTaxDeductible = false;
  DateTime _expenseDate = DateTime.now();
  File? _selectedReceipt;
  bool _isUploadingReceipt = false;

  bool _isEditMode = false;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _isEditMode = widget.expenseId != null;
  }

  @override
  void dispose() {
    _descriptionController.dispose();
    _amountController.dispose();
    _categoryController.dispose();
    _notesController.dispose();
    super.dispose();
  }

  Future<void> _selectDate() async {
    final picked = await showDatePicker(
      context: context,
      initialDate: _expenseDate,
      firstDate: DateTime.now().subtract(const Duration(days: 365)),
      lastDate: DateTime.now(),
    );

    if (picked != null) {
      setState(() {
        _expenseDate = picked;
      });
    }
  }

  Future<void> _pickReceipt() async {
    final file = await _imagePicker.showImageSourceDialog(context);
    if (file != null) {
      // Validate image
      final isValid = await _imagePicker.validateImage(file);
      if (!isValid && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Invalid image. Please select a valid image file (max 10MB)'),
            backgroundColor: Colors.red,
          ),
        );
        return;
      }

      setState(() {
        _selectedReceipt = file;
      });
    }
  }

  void _showCategorySelector() {
    final categories = ['Office', 'Travel', 'Marketing', 'Utilities', 'Other'];

    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Select Category',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 16),
            ...categories.map((category) => ListTile(
              title: Text(category),
              onTap: () {
                setState(() {
                  _categoryController.text = category;
                });
                context.pop();
              },
            )).toList(),
          ],
        ),
      ),
    );
  }

  Future<void> _submitForm() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isLoading = true);

    final data = {
      'description': _descriptionController.text,
      'amount': double.parse(_amountController.text),
      'currency': 'USD',
      'category': _categoryController.text.toLowerCase(),
      'is_tax_deductible': _isTaxDeductible,
      'notes': _notesController.text.isEmpty ? null : _notesController.text,
      'date': _expenseDate.toIso8601String().split('T')[0],
    };

    bool success;
    String? expenseId;

    if (_isEditMode) {
      expenseId = widget.expenseId;
      success = await ref
          .read(expenseProvider.notifier)
          .updateExpense(expenseId!, data);
    } else {
      // For new expenses, create first then upload receipt
      success = await ref
          .read(expenseProvider.notifier)
          .createExpense(data);

      if (success) {
        // Get the newly created expense ID from the state
        final expenses = ref.read(expenseProvider).expenses;
        if (expenses.isNotEmpty) {
          // Find the expense with matching description and date
          final newExpense = expenses.firstWhere(
            (e) => e.description == data['description'] &&
                   e.date.toIso8601String().split('T')[0] == data['date'],
            orElse: () => expenses.first,
          );
          expenseId = newExpense.id;
        }
      }
    }

    // Upload receipt if one was selected
    if (success && _selectedReceipt != null && expenseId != null) {
      setState(() => _isUploadingReceipt = true);

      final size = await _imagePicker.getFileSizeInMB(_selectedReceipt!);
      final uploadSuccess = await ref
          .read(expenseProvider.notifier)
          .uploadReceipt(expenseId, _selectedReceipt!.path, size);

      setState(() => _isUploadingReceipt = false);

      if (!uploadSuccess && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Expense saved but receipt upload failed'),
            backgroundColor: Colors.orange,
          ),
        );
      }
    }

    setState(() => _isLoading = false);

    if (success && mounted) {
      context.pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(_isEditMode ? 'Expense updated successfully' : 'Expense added successfully'),
          backgroundColor: Colors.green,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_isEditMode ? 'Edit Expense' : 'Add Expense'),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Basic Info
              const Text(
                'Basic Information',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildTextField(
                controller: _descriptionController,
                label: 'Description *',
                hint: 'What was this expense for?',
                required: true,
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: _buildTextField(
                      controller: _amountController,
                      label: 'Amount *',
                      hint: '\$0.00',
                      keyboardType: TextInputType.number,
                      required: true,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildCategoryField(),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              _buildDateField(),
              const SizedBox(height: 24),

              // Additional Info
              const Text(
                'Additional Information',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildTextField(
                controller: _notesController,
                label: 'Notes (Optional)',
                hint: 'Additional details',
                maxLines: 3,
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Checkbox(
                    value: _isTaxDeductible,
                    onChanged: (value) {
                      setState(() {
                        _isTaxDeductible = value ?? false;
                      });
                    },
                  ),
                  const Text('Tax Deductible'),
                ],
              ),
              const SizedBox(height: 24),

              // Receipt Section
              const Text(
                'Receipt (Optional)',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 12),
              _buildReceiptSection(),
              const SizedBox(height: 32),

              // Actions
              SizedBox(
                width: double.infinity,
                height: 50,
                child: ElevatedButton(
                  onPressed: _isLoading ? null : _submitForm,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(8),
                    ),
                  ),
                  child: _isLoading
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                          ),
                        )
                      : Text(
                          _isEditMode ? 'Update Expense' : 'Add Expense',
                          style: const TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String label,
    String? hint,
    bool required = false,
    TextInputType keyboardType = TextInputType.text,
    int maxLines = 1,
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
            if (label.contains('Amount') && value != null && value.isNotEmpty) {
              if (double.tryParse(value) == null) {
                return 'Invalid amount';
              }
            }
            return null;
          },
        ),
      ],
    );
  }

  Widget _buildCategoryField() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Category *',
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w500,
          ),
        ),
        const SizedBox(height: 4),
        InkWell(
          onTap: _showCategorySelector,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.grey.shade300),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(_categoryController.text),
                const Icon(Icons.arrow_drop_down),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildDateField() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Date *',
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w500,
          ),
        ),
        const SizedBox(height: 4),
        InkWell(
          onTap: _selectDate,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.grey.shade300),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('${_expenseDate.day}/${_expenseDate.month}/${_expenseDate.year}'),
                const Icon(Icons.calendar_today, size: 18),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildReceiptSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (_selectedReceipt != null) ...[
          Container(
            height: 200,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: Colors.grey.shade300),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(8),
              child: Image.file(
                _selectedReceipt!,
                fit: BoxFit.cover,
                width: double.infinity,
              ),
            ),
          ),
          const SizedBox(height: 8),
          FutureBuilder<double>(
            future: _imagePicker.getFileSizeInMB(_selectedReceipt!),
            builder: (context, snapshot) {
              if (snapshot.hasData) {
                return Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'File size: ${snapshot.data!.toStringAsFixed(2)} MB',
                      style: const TextStyle(fontSize: 12, color: Colors.grey),
                    ),
                    IconButton(
                      icon: const Icon(Icons.clear, color: Colors.red),
                      onPressed: () {
                        setState(() {
                          _selectedReceipt = null;
                        });
                      },
                      tooltip: 'Remove receipt',
                    ),
                  ],
                );
              }
              return const SizedBox.shrink();
            },
          ),
          const SizedBox(height: 8),
        ],
        SizedBox(
          width: double.infinity,
          child: OutlinedButton.icon(
            onPressed: _isUploadingReceipt ? null : _pickReceipt,
            icon: _isUploadingReceipt
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor: AlwaysStoppedAnimation<Color>(Colors.red),
                    ),
                  )
                : const Icon(Icons.receipt_long),
            label: Text(_selectedReceipt != null ? 'Change Receipt' : 'Add Receipt'),
            style: OutlinedButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 12),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
              ),
            ),
          ),
        ),
        if (_isUploadingReceipt) ...[
          const SizedBox(height: 8),
          const Text(
            'Uploading receipt...',
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
        ],
      ],
    );
  }
}
