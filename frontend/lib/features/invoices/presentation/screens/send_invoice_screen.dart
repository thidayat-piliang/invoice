import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/invoice_provider.dart';
import '../../../../shared/widgets/empty_state.dart';
import '../../../../app/theme/app_theme.dart';

/// Screen for sending invoices via email and/or WhatsApp
class SendInvoiceScreen extends ConsumerStatefulWidget {
  final String invoiceId;

  const SendInvoiceScreen({super.key, required this.invoiceId});

  @override
  ConsumerState<SendInvoiceScreen> createState() => _SendInvoiceScreenState();
}

class _SendInvoiceScreenState extends ConsumerState<SendInvoiceScreen> {
  final _emailController = TextEditingController();
  final _whatsappController = TextEditingController();
  final _messageController = TextEditingController();
  bool _sendEmail = true;
  bool _sendWhatsApp = false;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadInvoice();
    });
  }

  @override
  void dispose() {
    _emailController.dispose();
    _whatsappController.dispose();
    _messageController.dispose();
    super.dispose();
  }

  Future<void> _loadInvoice() async {
    await ref.read(invoiceProvider.notifier).loadInvoice(widget.invoiceId);
    final invoice = ref.read(invoiceProvider).selectedInvoice;
    if (invoice != null) {
      setState(() {
        _emailController.text = invoice.clientEmail ?? '';
        _whatsappController.text = invoice.clientPhone ?? '';
        _messageController.text = 'Invoice #${invoice.invoiceNumber} is ready for payment. Total: \$${invoice.totalAmount.toStringAsFixed(2)}';
      });
    }
  }

  Future<void> _sendInvoice() async {
    if (!_sendEmail && !_sendWhatsApp) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please select at least one delivery method'),
          backgroundColor: Colors.orange,
        ),
      );
      return;
    }

    setState(() => _isLoading = true);

    final data = {
      'send_email': _sendEmail,
      'send_whatsapp': _sendWhatsApp,
      'email': _emailController.text.trim(),
      'phone': _whatsappController.text.trim(),
      'message': _messageController.text.trim(),
    };

    final success = await ref.read(invoiceProvider.notifier).sendInvoice(widget.invoiceId, data);

    setState(() => _isLoading = false);

    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invoice sent successfully!'),
          backgroundColor: Colors.green,
        ),
      );
      context.go('/invoices/${widget.invoiceId}');
    } else if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ref.read(invoiceProvider).error ?? 'Failed to send invoice'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _sendWhatsApp() async {
    setState(() => _isLoading = true);

    final success = await ref.read(invoiceProvider.notifier).sendInvoiceWhatsapp(widget.invoiceId);

    setState(() => _isLoading = false);

    if (success && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invoice sent via WhatsApp!'),
          backgroundColor: Colors.green,
        ),
      );
      context.go('/invoices/${widget.invoiceId}');
    } else if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to send WhatsApp message'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final invoiceState = ref.watch(invoiceProvider);
    final invoice = invoiceState.selectedInvoice;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Send Invoice'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/invoices/${widget.invoiceId}'),
        ),
      ),
      body: invoiceState.isLoading && invoice == null
          ? const Center(child: CircularProgressIndicator())
          : invoice == null
              ? const EmptyState(
                  icon: Icons.error_outline,
                  title: 'Invoice Not Found',
                  message: 'Unable to load invoice details',
                )
              : SingleChildScrollView(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Invoice Summary
                      Card(
                        child: Padding(
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                'Invoice #${invoice.invoiceNumber}',
                                style: const TextStyle(
                                  fontSize: 18,
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                              const SizedBox(height: 8),
                              Text('Client: ${invoice.clientName}'),
                              Text('Amount: \$${invoice.totalAmount.toStringAsFixed(2)}'),
                              Text('Due Date: ${invoice.dueDate.month}/${invoice.dueDate.day}/${invoice.dueDate.year}'),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(height: 20),

                      // Delivery Methods
                      const Text(
                        'Delivery Methods',
                        style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 12),
                      Card(
                        child: Column(
                          children: [
                            CheckboxListTile(
                              title: const Row(
                                children: [
                                  Icon(Icons.email, color: Colors.blue),
                                  SizedBox(width: 8),
                                  Text('Send via Email'),
                                ],
                              ),
                              value: _sendEmail,
                              onChanged: (value) {
                                setState(() => _sendEmail = value!);
                              },
                            ),
                            if (_sendEmail) ...[
                              Padding(
                                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                                child: TextField(
                                  controller: _emailController,
                                  decoration: const InputDecoration(
                                    labelText: 'Email Address',
                                    border: OutlineInputBorder(),
                                    prefixIcon: Icon(Icons.email_outlined),
                                  ),
                                  keyboardType: TextInputType.emailAddress,
                                ),
                              ),
                            ],
                            const Divider(),
                            CheckboxListTile(
                              title: const Row(
                                children: [
                                  Icon(Icons.whatsapp, color: Colors.green),
                                  SizedBox(width: 8),
                                  Text('Send via WhatsApp'),
                                ],
                              ),
                              value: _sendWhatsApp,
                              onChanged: (value) {
                                setState(() => _sendWhatsApp = value!);
                              },
                            ),
                            if (_sendWhatsApp) ...[
                              Padding(
                                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                                child: TextField(
                                  controller: _whatsappController,
                                  decoration: const InputDecoration(
                                    labelText: 'Phone Number',
                                    border: OutlineInputBorder(),
                                    prefixIcon: Icon(Icons.phone_outlined),
                                    hintText: '+1234567890',
                                  ),
                                  keyboardType: TextInputType.phone,
                                ),
                              ),
                            ],
                          ],
                        ),
                      ),
                      const SizedBox(height: 20),

                      // Custom Message
                      const Text(
                        'Custom Message (Optional)',
                        style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 12),
                      Card(
                        child: Padding(
                          padding: const EdgeInsets.all(16),
                          child: TextField(
                            controller: _messageController,
                            decoration: const InputDecoration(
                              border: OutlineInputBorder(),
                              hintText: 'Enter custom message...',
                            ),
                            maxLines: 3,
                          ),
                        ),
                      ),
                      const SizedBox(height: 20),

                      // Guest Link Info
                      if (invoice.guestPaymentToken != null) ...[
                        Card(
                          color: Colors.green.shade50,
                          child: Padding(
                            padding: const EdgeInsets.all(16),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const Row(
                                  children: [
                                    Icon(Icons.link, color: Colors.green),
                                    SizedBox(width: 8),
                                    Text(
                                      'Guest Payment Link',
                                      style: TextStyle(
                                        fontWeight: FontWeight.bold,
                                        color: Colors.green,
                                      ),
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 8),
                                Text(
                                  'Token: ${invoice.guestPaymentToken}',
                                  style: const TextStyle(fontSize: 12),
                                ),
                                const SizedBox(height: 4),
                                Text(
                                  'Share this link with clients who don\'t have an account',
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Colors.green.shade700,
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                        const SizedBox(height: 20),
                      ],

                      // Action Buttons
                      Row(
                        children: [
                          Expanded(
                            child: OutlinedButton.icon(
                              onPressed: _isLoading ? null : _sendWhatsApp,
                              icon: const Icon(Icons.whatsapp),
                              label: const Text('WhatsApp Only'),
                              style: OutlinedButton.styleFrom(
                                foregroundColor: Colors.green,
                                side: const BorderSide(color: Colors.green),
                              ),
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: ElevatedButton.icon(
                              onPressed: _isLoading ? null : _sendInvoice,
                              icon: const Icon(Icons.send),
                              label: Text(_isLoading ? 'Sending...' : 'Send Invoice'),
                              style: ElevatedButton.styleFrom(
                                backgroundColor: AppTheme.primaryColor,
                                foregroundColor: Colors.white,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
    );
  }
}
