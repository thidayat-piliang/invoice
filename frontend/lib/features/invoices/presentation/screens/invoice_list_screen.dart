import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/invoice_provider.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../shared/widgets/search_bar.dart';
import '../../../../shared/widgets/filter_chip.dart';

class InvoiceListScreen extends ConsumerStatefulWidget {
  const InvoiceListScreen({super.key});

  @override
  ConsumerState<InvoiceListScreen> createState() => _InvoiceListScreenState();
}

class _InvoiceListScreenState extends ConsumerState<InvoiceListScreen> {
  final TextEditingController _searchController = TextEditingController();
  String _selectedStatus = 'all';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadInvoices();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  void _loadInvoices() {
    ref.read(invoiceProvider.notifier).loadInvoices(
      status: _selectedStatus == 'all' ? null : _selectedStatus,
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
              'Filter by Status',
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
                _buildFilterChip('draft', 'Draft'),
                _buildFilterChip('sent', 'Sent'),
                _buildFilterChip('viewed', 'Viewed'),
                _buildFilterChip('paid', 'Paid'),
                _buildFilterChip('overdue', 'Overdue'),
                _buildFilterChip('partial', 'Partial'),
              ],
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                onPressed: () {
                  context.pop();
                  _loadInvoices();
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
      isSelected: _selectedStatus == value,
      onTap: () {
        setState(() {
          _selectedStatus = value;
        });
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final invoiceState = ref.watch(invoiceProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Invoices'),
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
              hintText: 'Search invoices...',
              onSearch: (value) => _loadInvoices(),
            ),
          ),
          if (_selectedStatus != 'all')
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Text(
                    'Filter: ${_selectedStatus.toUpperCase()}',
                    style: TextStyle(
                      color: Colors.blue.shade700,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(width: 8),
                  TextButton(
                    onPressed: () {
                      setState(() {
                        _selectedStatus = 'all';
                        _searchController.clear();
                      });
                      _loadInvoices();
                    },
                    child: const Text('Clear'),
                  ),
                ],
              ),
            ),
          Expanded(
            child: invoiceState.isLoading && invoiceState.invoices.isEmpty
                ? const Center(child: CircularProgressIndicator())
                : invoiceState.error != null
                    ? Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
                            const SizedBox(height: 16),
                            Text(
                              invoiceState.error!,
                              style: const TextStyle(color: Colors.red),
                              textAlign: TextAlign.center,
                            ),
                            const SizedBox(height: 16),
                            ElevatedButton(
                              onPressed: _loadInvoices,
                              child: const Text('Retry'),
                            ),
                          ],
                        ),
                      )
                    : invoiceState.invoices.isEmpty
                        ? const EmptyState(
                            icon: Icons.description_outlined,
                            title: 'No Invoices Found',
                            message: 'Try adjusting your filters or create a new invoice',
                          )
                        : RefreshIndicator(
                            onRefresh: () async {
                              _loadInvoices();
                            },
                            child: ListView.builder(
                              padding: const EdgeInsets.all(8),
                              itemCount: invoiceState.invoices.length,
                              itemBuilder: (context, index) {
                                final invoice = invoiceState.invoices[index];
                                return InvoiceCard(invoice: invoice);
                              },
                            ),
                          ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.go('/invoices/create'),
        label: const Text('New Invoice'),
        icon: const Icon(Icons.add),
      ),
    );
  }
}

class InvoiceCard extends StatelessWidget {
  final Invoice invoice;

  const InvoiceCard({super.key, required this.invoice});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: InkWell(
        onTap: () => context.go('/invoices/${invoice.id}'),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      Text(
                        invoice.invoiceNumber,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      if (invoice.isViewed) ...[
                        const SizedBox(width: 6),
                        const Icon(Icons.visibility, size: 14, color: Colors.blue),
                      ],
                      if (invoice.hasGuestLink) ...[
                        const SizedBox(width: 6),
                        const Icon(Icons.link, size: 14, color: Colors.green),
                      ],
                    ],
                  ),
                  _StatusBadge(status: invoice.status),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                invoice.clientName,
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
              if (invoice.sentAt != null) ...[
                const SizedBox(height: 4),
                Row(
                  children: [
                    Icon(Icons.send, size: 12, color: Colors.grey[600]),
                    const SizedBox(width: 4),
                    Text(
                      'Sent: ${_formatDateTime(invoice.sentAt!)}',
                      style: TextStyle(
                        fontSize: 11,
                        color: Colors.grey[600],
                      ),
                    ),
                  ],
                ),
              ],
              if (invoice.isViewed) ...[
                const SizedBox(height: 4),
                Row(
                  children: [
                    Icon(Icons.visibility, size: 12, color: Colors.blue),
                    const SizedBox(width: 4),
                    Text(
                      'Viewed: ${_formatDateTime(invoice.viewedAt!)}',
                      style: const TextStyle(
                        fontSize: 11,
                        color: Colors.blue,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ],
                ),
              ],
              const SizedBox(height: 8),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    'Due ${_formatDate(invoice.dueDate)}',
                    style: TextStyle(
                      fontSize: 12,
                      color: invoice.isOverdue ? Colors.red : Colors.grey[600],
                    ),
                  ),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Text(
                        '\$${invoice.totalAmount.toStringAsFixed(2)}',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.w700,
                          color: Theme.of(context).colorScheme.primary,
                        ),
                      ),
                      if (invoice.isPartial) ...[
                        const SizedBox(height: 2),
                        Text(
                          'Paid: \$${invoice.amountPaid.toStringAsFixed(2)}',
                          style: TextStyle(
                            fontSize: 11,
                            color: Colors.orange.shade700,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                      if (invoice.isPartial && invoice.allowPartialPayment) ...[
                        const SizedBox(height: 2),
                        Text(
                          'Partial payments allowed',
                          style: TextStyle(
                            fontSize: 10,
                            color: Colors.orange.shade600,
                          ),
                        ),
                      ],
                    ],
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.month}/${date.day}/${date.year}';
  }

  String _formatDateTime(DateTime date) {
    return '${date.month}/${date.day}/${date.year} ${date.hour.toString().padLeft(2, '0')}:${date.minute.toString().padLeft(2, '0')}';
  }
}

class _StatusBadge extends StatelessWidget {
  final String status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    Color color;
    switch (status) {
      case 'draft':
        color = Colors.grey;
        break;
      case 'sent':
      case 'viewed':
        color = Colors.blue;
        break;
      case 'paid':
        color = Colors.green;
        break;
      case 'overdue':
        color = Colors.red;
        break;
      case 'partial':
        color = Colors.orange;
        break;
      default:
        color = Colors.grey;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        status.toUpperCase(),
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
