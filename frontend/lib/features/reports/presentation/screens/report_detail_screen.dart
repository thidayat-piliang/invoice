import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/report_provider.dart';

class ReportDetailScreen extends ConsumerStatefulWidget {
  final String reportType;

  const ReportDetailScreen({super.key, required this.reportType});

  @override
  ConsumerState<ReportDetailScreen> createState() => _ReportDetailScreenState();
}

class _ReportDetailScreenState extends ConsumerState<ReportDetailScreen> {
  final TextEditingController _startDateController = TextEditingController();
  final TextEditingController _endDateController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadReport();
    });
  }

  @override
  void dispose() {
    _startDateController.dispose();
    _endDateController.dispose();
    super.dispose();
  }

  void _loadReport() {
    final startDate = _startDateController.text.isEmpty ? null : _startDateController.text;
    final endDate = _endDateController.text.isEmpty ? null : _endDateController.text;

    switch (widget.reportType) {
      case 'income':
        ref.read(reportProvider.notifier).loadIncomeReport(startDate: startDate, endDate: endDate);
        break;
      case 'expenses':
        ref.read(reportProvider.notifier).loadExpenseReport(startDate: startDate, endDate: endDate);
        break;
      case 'tax':
        ref.read(reportProvider.notifier).loadTaxReport(startDate: startDate, endDate: endDate);
        break;
      case 'aging':
        ref.read(reportProvider.notifier).loadAgingReport();
        break;
    }
  }

  Future<void> _selectDate(bool isStart) async {
    final picked = await showDatePicker(
      context: context,
      initialDate: DateTime.now(),
      firstDate: DateTime(2020),
      lastDate: DateTime.now(),
    );

    if (picked != null) {
      setState(() {
        final dateStr = '${picked.year}-${picked.month.toString().padLeft(2, '0')}-${picked.day.toString().padLeft(2, '0')}';
        if (isStart) {
          _startDateController.text = dateStr;
        } else {
          _endDateController.text = dateStr;
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final reportState = ref.watch(reportProvider);
    final title = _getTitle();

    return Scaffold(
      appBar: AppBar(
        title: Text(title),
      ),
      body: Column(
        children: [
          if (widget.reportType != 'aging')
            Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _startDateController,
                      decoration: const InputDecoration(
                        labelText: 'Start Date',
                        border: OutlineInputBorder(),
                        suffixIcon: Icon(Icons.calendar_today),
                      ),
                      readOnly: true,
                      onTap: () => _selectDate(true),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: TextField(
                      controller: _endDateController,
                      decoration: const InputDecoration(
                        labelText: 'End Date',
                        border: OutlineInputBorder(),
                        suffixIcon: Icon(Icons.calendar_today),
                      ),
                      readOnly: true,
                      onTap: () => _selectDate(false),
                    ),
                  ),
                  const SizedBox(width: 12),
                  IconButton(
                    icon: const Icon(Icons.refresh),
                    onPressed: _loadReport,
                    tooltip: 'Apply Filters',
                  ),
                ],
              ),
            ),
          Expanded(
            child: reportState.isLoading
                ? const Center(child: CircularProgressIndicator())
                : _buildReportContent(reportState),
          ),
        ],
      ),
    );
  }

  String _getTitle() {
    switch (widget.reportType) {
      case 'income':
        return 'Income Report';
      case 'expenses':
        return 'Expense Report';
      case 'tax':
        return 'Tax Report';
      case 'aging':
        return 'Aging Report';
      default:
        return 'Report';
    }
  }

  Widget _buildReportContent(ReportState state) {
    if (widget.reportType == 'income' && state.income != null) {
      return _buildIncomeReport(state.income!);
    }
    if (widget.reportType == 'expenses' && state.expense != null) {
      return _buildExpenseReport(state.expense!);
    }
    if (widget.reportType == 'tax' && state.tax != null) {
      return _buildTaxReport(state.tax!);
    }
    if (widget.reportType == 'aging' && state.aging != null) {
      return _buildAgingReport(state.aging!);
    }

    return Center(
      child: Text(
        widget.reportType == 'aging'
          ? 'Select date range to view report'
          : 'Select date range to view report',
        style: const TextStyle(color: Colors.grey),
      ),
    );
  }

  Widget _buildIncomeReport(IncomeReport report) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Total Income', style: TextStyle(color: Colors.grey)),
                  const SizedBox(height: 4),
                  Text(
                    '\$${report.totalIncome.toStringAsFixed(2)}',
                    style: const TextStyle(
                      fontSize: 28,
                      fontWeight: FontWeight.w700,
                      color: Colors.green,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          const Text('Top Invoices', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          ...report.topInvoices.map((invoice) => Card(
            margin: const EdgeInsets.only(bottom: 8),
            child: ListTile(
              title: Text(invoice['invoice_number'] ?? 'N/A', style: const TextStyle(fontWeight: FontWeight.w600)),
              subtitle: Text(invoice['client_name'] ?? ''),
              trailing: Text(
                '\$${(invoice['amount'] ?? 0.0).toStringAsFixed(2)}',
                style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.green),
              ),
            ),
          )).toList(),
        ],
      ),
    );
  }

  Widget _buildExpenseReport(ExpenseReport report) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Total Expenses', style: TextStyle(color: Colors.grey)),
                  const SizedBox(height: 4),
                  Text(
                    '\$${report.totalExpenses.toStringAsFixed(2)}',
                    style: const TextStyle(
                      fontSize: 28,
                      fontWeight: FontWeight.w700,
                      color: Colors.red,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          const Text('By Category', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          ...report.byCategory.map((category) => Card(
            margin: const EdgeInsets.only(bottom: 8),
            child: ListTile(
              title: Text(category['category'] ?? 'N/A', style: const TextStyle(fontWeight: FontWeight.w600)),
              trailing: Text(
                '\$${(category['amount'] ?? 0.0).toStringAsFixed(2)}',
                style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.red),
              ),
            ),
          )).toList(),
        ],
      ),
    );
  }

  Widget _buildTaxReport(TaxReport report) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  _buildTaxStat('Taxable Income', report.totalTaxableIncome, Colors.blue),
                  const SizedBox(height: 12),
                  _buildTaxStat('Tax Deductible', report.totalTaxDeductible, Colors.green),
                  const SizedBox(height: 12),
                  _buildTaxStat('Estimated Tax', report.estimatedTax, Colors.orange),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTaxStat(String label, double value, Color color) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(label, style: TextStyle(color: color, fontWeight: FontWeight.w500)),
        Text(
          '\$${value.toStringAsFixed(2)}',
          style: TextStyle(color: color, fontWeight: FontWeight.w700, fontSize: 16),
        ),
      ],
    );
  }

  Widget _buildAgingReport(AgingReport report) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  _buildAgingRow('Current', report.current, Colors.green),
                  const SizedBox(height: 8),
                  _buildAgingRow('1-30 Days', report.days1_30, Colors.blue),
                  const SizedBox(height: 8),
                  _buildAgingRow('31-60 Days', report.days31_60, Colors.orange),
                  const SizedBox(height: 8),
                  _buildAgingRow('61-90 Days', report.days61_90, Colors.deepOrange),
                  const SizedBox(height: 8),
                  _buildAgingRow('Over 90 Days', report.over90, Colors.red),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          const Text('Overdue Invoices', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          ...report.overdueInvoices.map((invoice) => Card(
            margin: const EdgeInsets.only(bottom: 8),
            child: ListTile(
              title: Text(invoice['invoice_number'] ?? 'N/A', style: const TextStyle(fontWeight: FontWeight.w600)),
              subtitle: Text(invoice['client_name'] ?? ''),
              trailing: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Text(
                    '\$${(invoice['amount'] ?? 0.0).toStringAsFixed(2)}',
                    style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.red),
                  ),
                  Text(
                    '${invoice['days_overdue'] ?? 0} days',
                    style: const TextStyle(fontSize: 10, color: Colors.red),
                  ),
                ],
              ),
            ),
          )).toList(),
        ],
      ),
    );
  }

  Widget _buildAgingRow(String label, double value, Color color) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(label, style: TextStyle(color: color, fontWeight: FontWeight.w500)),
        Text(
          '\$${value.toStringAsFixed(2)}',
          style: TextStyle(color: color, fontWeight: FontWeight.w700, fontSize: 16),
        ),
      ],
    );
  }
}
