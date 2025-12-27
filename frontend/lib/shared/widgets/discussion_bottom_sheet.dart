import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import '../../../features/invoices/presentation/providers/invoice_provider.dart';
import '../../../features/guest/presentation/providers/guest_provider.dart';

/// Discussion Bottom Sheet Widget
/// Shows a list of messages and allows adding new ones
class DiscussionBottomSheet extends ConsumerStatefulWidget {
  final String invoiceId;
  final String? guestToken; // If provided, use guest mode
  final String clientName;

  const DiscussionBottomSheet({
    super.key,
    required this.invoiceId,
    this.guestToken,
    required this.clientName,
  });

  @override
  ConsumerState<DiscussionBottomSheet> createState() => _DiscussionBottomSheetState();
}

class _DiscussionBottomSheetState extends ConsumerState<DiscussionBottomSheet> {
  final TextEditingController _messageController = TextEditingController();
  bool _isSending = false;
  List<DiscussionMessage> _messages = [];
  List<GuestDiscussionMessage> _guestMessages = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadMessages();
  }

  @override
  void dispose() {
    _messageController.dispose();
    super.dispose();
  }

  Future<void> _loadMessages() async {
    if (widget.guestToken != null) {
      // Guest mode
      final response = await ref.read(guestProvider.notifier).getDiscussionMessages(widget.guestToken!);
      if (response != null) {
        setState(() {
          _guestMessages = response.messages;
          _isLoading = false;
        });
      } else {
        setState(() => _isLoading = false);
      }
    } else {
      // Seller mode
      final response = await ref.read(invoiceProvider.notifier).getDiscussionMessages(widget.invoiceId);
      if (response != null) {
        setState(() {
          _messages = response.messages;
          _isLoading = false;
        });
      } else {
        setState(() => _isLoading = false);
      }
    }
  }

  Future<void> _sendMessage() async {
    if (_messageController.text.trim().isEmpty) return;

    setState(() {
      _isSending = true;
    });

    final message = _messageController.text.trim();

    if (widget.guestToken != null) {
      // Guest mode
      final newMessage = await ref.read(guestProvider.notifier).addDiscussionMessage(widget.guestToken!, message);
      if (newMessage != null) {
        setState(() {
          _guestMessages.add(newMessage);
          _messageController.clear();
        });
      }
    } else {
      // Seller mode
      final newMessage = await ref.read(invoiceProvider.notifier).addDiscussionMessage(widget.invoiceId, message);
      if (newMessage != null) {
        setState(() {
          _messages.add(newMessage);
          _messageController.clear();
        });
      }
    }

    setState(() {
      _isSending = false;
    });
  }

  Widget _buildMessageBubble(DiscussionMessage message) {
    final isSeller = message.isSeller;
    final isBuyer = message.isBuyer;

    return Align(
      alignment: isSeller ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.all(12),
        constraints: const BoxConstraints(maxWidth: 280),
        decoration: BoxDecoration(
          color: isSeller ? Colors.blue.shade50 : Colors.grey.shade200,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSeller ? Colors.blue.shade200 : Colors.grey.shade300,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  isSeller ? 'You' : widget.clientName,
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 12,
                    color: isSeller ? Colors.blue.shade700 : Colors.grey.shade700,
                  ),
                ),
                const SizedBox(width: 4),
                Text(
                  _formatTime(message.createdAt),
                  style: TextStyle(
                    fontSize: 10,
                    color: Colors.grey.shade600,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              message.message,
              style: const TextStyle(fontSize: 14),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildGuestMessageBubble(GuestDiscussionMessage message) {
    final isSeller = message.isSeller;
    final isBuyer = message.isBuyer;

    return Align(
      alignment: isBuyer ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.all(12),
        constraints: const BoxConstraints(maxWidth: 280),
        decoration: BoxDecoration(
          color: isBuyer ? Colors.green.shade50 : Colors.grey.shade200,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isBuyer ? Colors.green.shade200 : Colors.grey.shade300,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  isBuyer ? 'You' : 'Seller',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 12,
                    color: isBuyer ? Colors.green.shade700 : Colors.grey.shade700,
                  ),
                ),
                const SizedBox(width: 4),
                Text(
                  _formatTime(message.createdAt),
                  style: TextStyle(
                    fontSize: 10,
                    color: Colors.grey.shade600,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              message.message,
              style: const TextStyle(fontSize: 14),
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(DateTime dateTime) {
    final now = DateTime.now();
    final difference = now.difference(dateTime);

    if (difference.inMinutes < 1) {
      return 'Just now';
    } else if (difference.inMinutes < 60) {
      return '${difference.inMinutes}m ago';
    } else if (difference.inHours < 24) {
      return '${difference.inHours}h ago';
    } else {
      return DateFormat('MMM d, h:mm a').format(dateTime);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: MediaQuery.of(context).size.height * 0.75,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(20),
          topRight: Radius.circular(20),
        ),
      ),
      child: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(20),
                topRight: Radius.circular(20),
              ),
              border: Border(
                bottom: BorderSide(color: Colors.grey.shade200),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.chat, color: Colors.blue),
                const SizedBox(width: 12),
                Text(
                  'Discussion',
                  style: Theme.of(context).textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
          ),

          // Messages List
          Expanded(
            child: _isLoading
                ? const Center(child: CircularProgressIndicator())
                : (widget.guestToken != null ? _guestMessages.isEmpty : _messages.isEmpty)
                    ? Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.chat_bubble_outline, size: 64, color: Colors.grey.shade400),
                            const SizedBox(height: 16),
                            Text(
                              'No messages yet',
                              style: TextStyle(color: Colors.grey.shade600, fontSize: 16),
                            ),
                            const SizedBox(height: 8),
                            Text(
                              widget.guestToken != null
                                  ? 'Be the first to send a message'
                                  : 'Start the conversation',
                              style: TextStyle(color: Colors.grey.shade500, fontSize: 14),
                            ),
                          ],
                        ),
                      )
                    : ListView.builder(
                        padding: const EdgeInsets.all(16),
                        itemCount: widget.guestToken != null ? _guestMessages.length : _messages.length,
                        itemBuilder: (context, index) {
                          if (widget.guestToken != null) {
                            return _buildGuestMessageBubble(_guestMessages[index]);
                          }
                          return _buildMessageBubble(_messages[index]);
                        },
                      ),
          ),

          // Input Area
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              border: Border(
                top: BorderSide(color: Colors.grey.shade200),
              ),
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _messageController,
                    decoration: InputDecoration(
                      hintText: 'Type a message...',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(24),
                        borderSide: BorderSide(color: Colors.grey.shade300),
                      ),
                      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                      suffixIcon: _isSending
                          ? const Padding(
                              padding: EdgeInsets.all(12),
                              child: SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(strokeWidth: 2),
                              ),
                            )
                          : null,
                    ),
                    maxLines: 3,
                    minLines: 1,
                    onSubmitted: (_) => _sendMessage(),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton(
                  onPressed: _isSending ? null : _sendMessage,
                  icon: const Icon(Icons.send),
                  style: IconButton.styleFrom(
                    backgroundColor: Colors.blue,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.all(12),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Helper function to show the discussion bottom sheet
void showDiscussionBottomSheet(
  BuildContext context, {
  required String invoiceId,
  String? guestToken,
  required String clientName,
}) {
  showModalBottomSheet(
    context: context,
    isScrollControlled: true,
    backgroundColor: Colors.transparent,
    builder: (context) => DiscussionBottomSheet(
      invoiceId: invoiceId,
      guestToken: guestToken,
      clientName: clientName,
    ),
  );
}
