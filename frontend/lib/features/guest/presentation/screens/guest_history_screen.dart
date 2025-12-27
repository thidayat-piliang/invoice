import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import '../providers/guest_provider.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../app/theme/app_theme.dart';

/// Guest Payment History Screen - View payment history by email or phone
class GuestHistoryScreen extends ConsumerStatefulWidget {
  const GuestHistoryScreen({super.key});

  @override
  ConsumerState<GuestHistoryScreen> createState() => _GuestHistoryScreenState();
}

class _GuestHistoryScreenState extends ConsumerState<GuestHistoryScreen> {
  final _emailController = TextEditingController();
  final _phoneController = TextEditingController();
  bool _useEmail = true;

  @override
  void dispose() {
    _emailController.dispose();
    _phoneController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final guestState = ref.watch(guestProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Payment History'),
        backgroundColor: AppTheme.primaryColor,
        foregroundColor: Colors.white,
      ),
      body: Column(
        children: [
          // Search Form
          Container(
            color: Colors.white,
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                Row(
                  children: [
                    Expanded(
                      child: ChoiceChip(
                        label: const Text('Email'),
                        selected: _useEmail,
                        onSelected: (selected) => setState(() => _useEmail = true),
                      ),
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: ChoiceChip(
                        label: const Text('Phone'),
                        selected: !_useEmail,
                        onSelected: (selected) => setState(() => _useEmail = false),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                _useEmail
                    ? TextField(
                        controller: _emailController,
                        decoration: const InputDecoration(
                          labelText: 'Email Address',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.email),
                          hintText: 'your@email.com',
                        ),
                        keyboardType: TextInputType.emailAddress,
                      )
                    : TextField(
                        controller: _phoneController,
                        decoration: const InputDecoration(
                          labelText: 'Phone Number',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.phone),
                          hintText: '+1234567890',
                        ),
                        keyboardType: TextInputType.phone,
                      ),
                const SizedBox(height: 12),
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton.icon(
                    onPressed: guestState.isLoading ? null : _loadHistory,
                    icon: const Icon(Icons.search),
                    label: const Text('Search History'),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppTheme.primaryColor,
                      foregroundColor: Colors.white,
                    ),
                  ),
                ),
              ],
            ),
          ),

          // Results
          Expanded(
            child: guestState.isLoading
                ? const Center(child: CircularProgressIndicator())
                : guestState.error != null
                    ? Center(
                        child: EmptyState(
                          icon: Icons.error_outline,
                          title: 'Error',
                          message: guestState.error!,
                          onRetry: _loadHistory,
                        ),
                      )
                    : guestState.paymentHistory.isEmpty
                        ? const Center(
                            child: EmptyState(
                              icon: Icons.receipt_long,
                              title: 'No History Found',
                              message: 'Enter your email or phone to view payment history',
                            ),
                          )
                        : _buildHistoryList(guestState),
          ),
        ],
      ),
    );
  }

  Widget _buildHistoryList(GuestState state) {
    final theme = Theme.of(context);

    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: state.paymentHistory.length,
      separatorBuilder: (context, index) => const SizedBox(height: 8),
      itemBuilder: (context, index) {
        final payment = state.paymentHistory[index];
        return Card(
          elevation: 2,
          child: ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            leading: CircleAvatar(
              backgroundColor: AppTheme.primaryColor.withOpacity(0.2),
              child: Icon(Icons.receipt_long, color: AppTheme.primaryColor),
            ),
            title: Text(
              'Invoice #${payment.invoiceNumber}',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            subtitle: Text(
              DateFormat('MMM dd, yyyy').format(payment.paidAt),
              style: theme.textTheme.bodyMedium,
            ),
            trailing: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(
                  '\$${payment.amount.toStringAsFixed(2)}',
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                    color: AppTheme.primaryColor,
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.green.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    payment.status.toUpperCase(),
                    style: const TextStyle(
                      color: Colors.green,
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Future<void> _loadHistory() async {
    String? email;
    String? phone;

    if (_useEmail) {
      email = _emailController.text.trim();
      if (email.isEmpty) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Please enter your email address')),
        );
        return;
      }
    } else {
      phone = _phoneController.text.trim();
      if (phone.isEmpty) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Please enter your phone number')),
        );
        return;
      }
    }

    await ref.read(guestProvider.notifier).getPaymentHistory(email, phone);
  }
}
