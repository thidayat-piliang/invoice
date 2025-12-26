import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/settings_provider.dart';

class NotificationSettingsScreen extends ConsumerStatefulWidget {
  const NotificationSettingsScreen({super.key});

  @override
  ConsumerState<NotificationSettingsScreen> createState() => _NotificationSettingsScreenState();
}

class _NotificationSettingsScreenState extends ConsumerState<NotificationSettingsScreen> {
  bool _emailPaymentReceived = false;
  bool _emailInvoicePaid = false;
  bool _emailPaymentReminder = false;
  bool _pushPaymentReceived = false;
  bool _pushOverdue = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final settings = ref.read(settingsProvider).notifications;
      if (settings != null) {
        setState(() {
          _emailPaymentReceived = settings.emailPaymentReceived;
          _emailInvoicePaid = settings.emailInvoicePaid;
          _emailPaymentReminder = settings.emailPaymentReminder;
          _pushPaymentReceived = settings.pushPaymentReceived;
          _pushOverdue = settings.pushOverdue;
        });
      }
    });
  }

  Future<void> _saveSettings() async {
    final settings = NotificationSettings(
      emailPaymentReceived: _emailPaymentReceived,
      emailInvoicePaid: _emailInvoicePaid,
      emailPaymentReminder: _emailPaymentReminder,
      pushPaymentReceived: _pushPaymentReceived,
      pushOverdue: _pushOverdue,
    );

    final success = await ref.read(settingsProvider.notifier).updateNotificationSettings(settings);

    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Settings saved successfully'),
          backgroundColor: Colors.green,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Notification Settings'),
        actions: [
          IconButton(
            icon: const Icon(Icons.save),
            onPressed: _saveSettings,
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Email Notifications',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 12),
            Card(
              child: Column(
                children: [
                  _buildSwitchTile(
                    'Payment Received',
                    'Notify when a payment is received',
                    _emailPaymentReceived,
                    (value) => setState(() => _emailPaymentReceived = value!),
                  ),
                  const Divider(height: 1),
                  _buildSwitchTile(
                    'Invoice Paid',
                    'Notify when an invoice is fully paid',
                    _emailInvoicePaid,
                    (value) => setState(() => _emailInvoicePaid = value!),
                  ),
                  const Divider(height: 1),
                  _buildSwitchTile(
                    'Payment Reminder',
                    'Send reminders for pending invoices',
                    _emailPaymentReminder,
                    (value) => setState(() => _emailPaymentReminder = value!),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            const Text(
              'Push Notifications',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 12),
            Card(
              child: Column(
                children: [
                  _buildSwitchTile(
                    'Payment Received',
                    'Push notification when payment is received',
                    _pushPaymentReceived,
                    (value) => setState(() => _pushPaymentReceived = value!),
                  ),
                  const Divider(height: 1),
                  _buildSwitchTile(
                    'Overdue Invoices',
                    'Push notification for overdue invoices',
                    _pushOverdue,
                    (value) => setState(() => _pushOverdue = value!),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              height: 50,
              child: ElevatedButton(
                onPressed: _saveSettings,
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.orange,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(8),
                  ),
                ),
                child: const Text(
                  'Save Changes',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSwitchTile(String title, String subtitle, bool value, ValueChanged<bool?> onChanged) {
    return SwitchListTile(
      title: Text(title, style: const TextStyle(fontWeight: FontWeight.w600)),
      subtitle: Text(subtitle),
      value: value,
      onChanged: onChanged,
      activeColor: Colors.orange,
    );
  }
}
