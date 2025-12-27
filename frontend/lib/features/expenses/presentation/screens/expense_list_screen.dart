import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/expense_provider.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../shared/widgets/search_bar.dart';
import '../../../../shared/widgets/filter_chip.dart';
import '../../../../shared/services/image_picker_service.dart';

class ExpenseListScreen extends ConsumerStatefulWidget {
  const ExpenseListScreen({super.key});

  @override
  ConsumerState<ExpenseListScreen> createState() => _ExpenseListScreenState();
}

class _ExpenseListScreenState extends ConsumerState<ExpenseListScreen> {
  final TextEditingController _searchController = TextEditingController();
  String _selectedCategory = 'all';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadExpenses();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  void _loadExpenses() {
    ref.read(expenseProvider.notifier).loadExpenses(
      category: _selectedCategory == 'all' ? null : _selectedCategory,
      search: _searchController.text.isEmpty ? null : _searchController.text,
    );
  }

  void _showFilterDialog() {
    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Filter by Category',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _buildFilterChip('all', 'All'),
                _buildFilterChip('office', 'Office'),
                _buildFilterChip('travel', 'Travel'),
                _buildFilterChip('marketing', 'Marketing'),
                _buildFilterChip('utilities', 'Utilities'),
                _buildFilterChip('other', 'Other'),
              ],
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                onPressed: () {
                  context.pop();
                  _loadExpenses();
                },
                child: const Text('Apply Filters'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildFilterChip(String value, String label) {
    return CustomFilterChip(
      label: label,
      isSelected: _selectedCategory == value,
      onTap: () {
        setState(() {
          _selectedCategory = value;
        });
      },
    );
  }

  void _showDeleteDialog(String expenseId, String description) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Expense'),
        content: Text('Are you sure you want to delete "$description"? This action cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => context.pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () async {
              context.pop();
              final success = await ref.read(expenseProvider.notifier).deleteExpense(expenseId);
              if (success && mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Expense deleted successfully'),
                    backgroundColor: Colors.green,
                  ),
                );
              }
            },
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final expenseState = ref.watch(expenseProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Expenses'),
        actions: [
          IconButton(
            icon: const Icon(Icons.filter_list),
            onPressed: _showFilterDialog,
          ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: CustomSearchBar(
              controller: _searchController,
              hintText: 'Search expenses...',
              onSearch: (value) => _loadExpenses(),
            ),
          ),
          if (_selectedCategory != 'all')
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Text(
                    'Filter: ${_selectedCategory.toUpperCase()}',
                    style: TextStyle(
                      color: Colors.red.shade700,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(width: 8),
                  TextButton(
                    onPressed: () {
                      setState(() {
                        _selectedCategory = 'all';
                        _searchController.clear();
                      });
                      _loadExpenses();
                    },
                    child: const Text('Clear'),
                  ),
                ],
              ),
            ),
          Expanded(
            child: expenseState.isLoading && expenseState.expenses.isEmpty
                ? const Center(child: CircularProgressIndicator())
                : expenseState.error != null
                    ? Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
                            const SizedBox(height: 16),
                            Text(
                              expenseState.error!,
                              style: const TextStyle(color: Colors.red),
                              textAlign: TextAlign.center,
                            ),
                            const SizedBox(height: 16),
                            ElevatedButton(
                              onPressed: _loadExpenses,
                              child: const Text('Retry'),
                            ),
                          ],
                        ),
                      )
                    : expenseState.expenses.isEmpty
                        ? const EmptyState(
                            icon: Icons.receipt_long_outlined,
                            title: 'No Expenses Found',
                            message: 'Try adjusting your filters or add a new expense',
                          )
                        : RefreshIndicator(
                            onRefresh: () async {
                              _loadExpenses();
                            },
                            child: ListView.builder(
                              padding: const EdgeInsets.all(8),
                              itemCount: expenseState.expenses.length,
                              itemBuilder: (context, index) {
                                final expense = expenseState.expenses[index];
                                return _ExpenseCard(
                                  expense: expense,
                                  onEdit: () => context.go('/expenses/edit/${expense.id}'),
                                  onDelete: () => _showDeleteDialog(expense.id, expense.description),
                                );
                              },
                            ),
                          ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.go('/expenses/create'),
        label: const Text('Add Expense'),
        icon: const Icon(Icons.add),
      ),
    );
  }
}

class _ExpenseCard extends ConsumerWidget {
  final Expense expense;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  const _ExpenseCard({
    required this.expense,
    required this.onEdit,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Container(
          width: 48,
          height: 48,
          decoration: BoxDecoration(
            color: Colors.red.shade50,
            borderRadius: BorderRadius.circular(24),
          ),
          child: Center(
            child: Icon(
              Icons.receipt_long,
              color: Colors.red.shade700,
              size: 24,
            ),
          ),
        ),
        title: Text(
          expense.description,
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              expense.category.toUpperCase(),
              style: TextStyle(
                color: Colors.grey[600],
                fontSize: 12,
                fontWeight: FontWeight.w600,
              ),
            ),
            Text(
              _formatDate(expense.date),
              style: TextStyle(color: Colors.grey[600], fontSize: 12),
            ),
          ],
        ),
        trailing: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Text(
              '\$${expense.amount.toStringAsFixed(2)}',
              style: const TextStyle(
                fontWeight: FontWeight.w600,
                fontSize: 16,
                color: Colors.red,
              ),
            ),
            if (expense.isTaxDeductible)
              Text(
                'Tax Deductible',
                style: TextStyle(
                  color: Colors.green.shade700,
                  fontSize: 10,
                  fontWeight: FontWeight.w600,
                ),
              ),
            if (expense.hasReceipt)
              Icon(
                Icons.receipt,
                size: 14,
                color: Colors.blue.shade700,
              ),
          ],
        ),
        onTap: () {
          showModalBottomSheet(
            context: context,
            builder: (context) => _ExpenseOptionsBottomSheet(
              expense: expense,
              onEdit: onEdit,
              onDelete: onDelete,
            ),
          );
        },
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.day}/${date.month}/${date.year}';
  }
}

class _ExpenseOptionsBottomSheet extends ConsumerStatefulWidget {
  final Expense expense;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  const _ExpenseOptionsBottomSheet({
    required this.expense,
    required this.onEdit,
    required this.onDelete,
  });

  @override
  ConsumerState<_ExpenseOptionsBottomSheet> createState() => _ExpenseOptionsBottomSheetState();
}

class _ExpenseOptionsBottomSheetState extends ConsumerState<_ExpenseOptionsBottomSheet> {
  final ImagePickerService _imagePicker = ImagePickerService();
  bool _isUploading = false;

  Future<void> _uploadReceipt() async {
    final file = await _imagePicker.showImageSourceDialog(context);
    if (file == null || !mounted) return;

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

    setState(() => _isUploading = true);

    final size = await _imagePicker.getFileSizeInMB(file);
    final success = await ref
        .read(expenseProvider.notifier)
        .uploadReceipt(widget.expense.id, file.path, size);

    setState(() => _isUploading = false);

    if (mounted) {
      context.pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(success ? 'Receipt uploaded successfully!' : 'Failed to upload receipt'),
          backgroundColor: success ? Colors.green : Colors.red,
        ),
      );
    }
  }

  Future<void> _viewReceipt() async {
    // For now, we'll show a dialog with receipt info
    // In a real app, you might download and display the image
    if (!mounted) return;

    context.pop();
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Receipt viewing would open the image here'),
        backgroundColor: Colors.blue,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'Expense Options',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: () {
                context.pop();
                widget.onEdit();
              },
              icon: const Icon(Icons.edit),
              label: const Text('Edit Expense'),
            ),
          ),
          const SizedBox(height: 8),
          if (widget.expense.hasReceipt)
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: _isUploading ? null : _viewReceipt,
                icon: const Icon(Icons.visibility),
                label: const Text('View Receipt'),
              ),
            ),
          if (widget.expense.hasReceipt) const SizedBox(height: 8),
          SizedBox(
            width: double.infinity,
            child: OutlinedButton.icon(
              onPressed: _isUploading ? null : _uploadReceipt,
              icon: _isUploading
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        valueColor: AlwaysStoppedAnimation<Color>(Colors.blue),
                      ),
                    )
                  : const Icon(Icons.receipt_long),
              label: Text(
                _isUploading
                    ? 'Uploading...'
                    : (widget.expense.hasReceipt ? 'Replace Receipt' : 'Upload Receipt'),
              ),
            ),
          ),
          const SizedBox(height: 8),
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: () {
                context.pop();
                widget.onDelete();
              },
              icon: const Icon(Icons.delete),
              label: const Text('Delete Expense'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            ),
          ),
          const SizedBox(height: 8),
          SizedBox(
            width: double.infinity,
            child: OutlinedButton(
              onPressed: () => context.pop(),
              child: const Text('Close'),
            ),
          ),
        ],
      ),
    );
  }
}
