import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Guest Discussion Message Model
class GuestDiscussionMessage {
  final String id;
  final String invoiceId;
  final String senderType; // 'seller' or 'buyer'
  final String message;
  final DateTime createdAt;

  GuestDiscussionMessage({
    required this.id,
    required this.invoiceId,
    required this.senderType,
    required this.message,
    required this.createdAt,
  });

  factory GuestDiscussionMessage.fromJson(Map<String, dynamic> json) {
    return GuestDiscussionMessage(
      id: json['id'],
      invoiceId: json['invoice_id'],
      senderType: json['sender_type'],
      message: json['message'],
      createdAt: DateTime.parse(json['created_at']),
    );
  }

  bool get isSeller => senderType.toLowerCase() == 'seller';
  bool get isBuyer => senderType.toLowerCase() == 'buyer';
}

// Guest Discussion Response Model
class GuestDiscussionResponse {
  final List<GuestDiscussionMessage> messages;

  GuestDiscussionResponse({required this.messages});

  factory GuestDiscussionResponse.fromJson(Map<String, dynamic> json) {
    final messagesList = json['messages'] as List;
    return GuestDiscussionResponse(
      messages: messagesList.map((e) => GuestDiscussionMessage.fromJson(e)).toList(),
    );
  }

  int get count => messages.length;
  bool get isEmpty => messages.isEmpty;
}

// Guest Invoice Model
class GuestInvoice {
  final String id;
  final String invoiceNumber;
  final String status;
  final String clientName;
  final String? clientEmail;
  final String? clientPhone;
  final DateTime issueDate;
  final DateTime dueDate;
  final double subtotal;
  final double taxAmount;
  final double discountAmount;
  final double totalAmount;
  final double amountPaid;
  final double balanceDue;
  final List<GuestInvoiceItem> items;
  final String? notes;
  final String? terms;
  final String? guestPaymentToken;
  final DateTime? viewedAt;
  final DateTime? sentAt;
  final bool allowPartialPayment;
  final double? minPaymentAmount;
  final int partialPaymentCount;

  GuestInvoice({
    required this.id,
    required this.invoiceNumber,
    required this.status,
    required this.clientName,
    this.clientEmail,
    this.clientPhone,
    required this.issueDate,
    required this.dueDate,
    required this.subtotal,
    required this.taxAmount,
    required this.discountAmount,
    required this.totalAmount,
    required this.amountPaid,
    required this.balanceDue,
    required this.items,
    this.notes,
    this.terms,
    this.guestPaymentToken,
    this.viewedAt,
    this.sentAt,
    this.allowPartialPayment = true,
    this.minPaymentAmount,
    this.partialPaymentCount = 0,
  });

  factory GuestInvoice.fromJson(Map<String, dynamic> json) {
    return GuestInvoice(
      id: json['id'],
      invoiceNumber: json['invoice_number'],
      status: json['status'],
      clientName: json['client_name'],
      clientEmail: json['client_email'],
      clientPhone: json['client_phone'],
      issueDate: DateTime.parse(json['issue_date']),
      dueDate: DateTime.parse(json['due_date']),
      subtotal: json['subtotal']?.toDouble() ?? 0.0,
      taxAmount: json['tax_amount']?.toDouble() ?? 0.0,
      discountAmount: json['discount_amount']?.toDouble() ?? 0.0,
      totalAmount: json['total_amount']?.toDouble() ?? 0.0,
      amountPaid: json['amount_paid']?.toDouble() ?? 0.0,
      balanceDue: json['balance_due']?.toDouble() ?? 0.0,
      items: (json['items'] as List).map((e) => GuestInvoiceItem.fromJson(e)).toList(),
      notes: json['notes'],
      terms: json['terms'],
      guestPaymentToken: json['guest_payment_token'],
      viewedAt: json['viewed_at'] != null ? DateTime.parse(json['viewed_at']) : null,
      sentAt: json['sent_at'] != null ? DateTime.parse(json['sent_at']) : null,
      allowPartialPayment: json['allow_partial_payment'] ?? true,
      minPaymentAmount: json['min_payment_amount']?.toDouble(),
      partialPaymentCount: json['partial_payment_count'] ?? 0,
    );
  }

  bool get isOverdue => dueDate.isBefore(DateTime.now()) && status != 'paid' && status != 'cancelled';
  bool get isViewed => viewedAt != null;
  bool get isPartial => status.toLowerCase() == 'partial';
}

// Guest Invoice Item Model
class GuestInvoiceItem {
  final String description;
  final double quantity;
  final double unitPrice;
  final double taxRate;
  final double taxAmount;
  final double total;

  GuestInvoiceItem({
    required this.description,
    required this.quantity,
    required this.unitPrice,
    required this.taxRate,
    required this.taxAmount,
    required this.total,
  });

  factory GuestInvoiceItem.fromJson(Map<String, dynamic> json) {
    return GuestInvoiceItem(
      description: json['description'],
      quantity: json['quantity']?.toDouble() ?? 0.0,
      unitPrice: json['unit_price']?.toDouble() ?? 0.0,
      taxRate: json['tax_rate']?.toDouble() ?? 0.0,
      taxAmount: json['tax_amount']?.toDouble() ?? 0.0,
      total: json['total']?.toDouble() ?? 0.0,
    );
  }
}

// Guest Payment History Model
class GuestPaymentHistory {
  final String invoiceNumber;
  final double amount;
  final DateTime paidAt;
  final String status;

  GuestPaymentHistory({
    required this.invoiceNumber,
    required this.amount,
    required this.paidAt,
    required this.status,
  });

  factory GuestPaymentHistory.fromJson(Map<String, dynamic> json) {
    return GuestPaymentHistory(
      invoiceNumber: json['invoice_number'],
      amount: json['amount']?.toDouble() ?? 0.0,
      paidAt: DateTime.parse(json['paid_at']),
      status: json['status'],
    );
  }
}

// Guest State
class GuestState {
  final bool isLoading;
  final String? error;
  final String? successMessage;
  final GuestInvoice? invoice;
  final List<GuestPaymentHistory> paymentHistory;
  final bool paymentProcessed;

  GuestState({
    this.isLoading = false,
    this.error,
    this.successMessage,
    this.invoice,
    this.paymentHistory = const [],
    this.paymentProcessed = false,
  });

  GuestState copyWith({
    bool? isLoading,
    String? error,
    String? successMessage,
    GuestInvoice? invoice,
    List<GuestPaymentHistory>? paymentHistory,
    bool? paymentProcessed,
  }) {
    return GuestState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      successMessage: successMessage,
      invoice: invoice ?? this.invoice,
      paymentHistory: paymentHistory ?? this.paymentHistory,
      paymentProcessed: paymentProcessed ?? this.paymentProcessed,
    );
  }
}

// Guest Notifier
class GuestNotifier extends StateNotifier<GuestState> {
  final ApiClient _apiClient;

  GuestNotifier(this._apiClient) : super(GuestState());

  // Load guest invoice by token
  Future<bool> loadGuestInvoice(String token) async {
    state = state.copyWith(isLoading: true, error: null, invoice: null);

    try {
      final response = await _apiClient.getGuestInvoice(token);
      final invoice = GuestInvoice.fromJson(response.data);

      state = state.copyWith(
        isLoading: false,
        invoice: invoice,
      );

      // Mark as viewed (for tracking)
      await markInvoiceViewed(token);

      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load invoice',
      );
      return false;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return false;
    }
  }

  // Process guest payment
  Future<bool> processPayment(String token, Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null, paymentProcessed: false);

    try {
      final response = await _apiClient.processGuestPayment(token, data);

      state = state.copyWith(
        isLoading: false,
        successMessage: 'Payment processed successfully!',
        paymentProcessed: true,
      );

      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Payment failed',
      );
      return false;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return false;
    }
  }

  // Get guest payment history
  Future<bool> getPaymentHistory(String? email, String? phone) async {
    state = state.copyWith(isLoading: true, error: null, paymentHistory: []);

    try {
      final response = await _apiClient.getGuestPaymentHistory({
        'email': email,
        'phone': phone,
      });

      final List<dynamic> data = response.data['payments'] ?? [];
      final history = data.map((json) => GuestPaymentHistory.fromJson(json)).toList();

      state = state.copyWith(
        isLoading: false,
        paymentHistory: history,
      );

      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load history',
      );
      return false;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return false;
    }
  }

  // Mark invoice as viewed
  Future<bool> markInvoiceViewed(String token) async {
    try {
      await _apiClient.markGuestInvoiceViewed(token);
      return true;
    } catch (e) {
      return false;
    }
  }

  // Send guest payment link via WhatsApp
  Future<bool> sendGuestPaymentLink(String token) async {
    try {
      await _apiClient.sendGuestPaymentLink(token);
      return true;
    } catch (e) {
      return false;
    }
  }

  // Clear error
  void clearError() {
    state = state.copyWith(error: null);
  }

  // Clear success message
  void clearSuccess() {
    state = state.copyWith(successMessage: null, paymentProcessed: false);
  }

  // Get discussion messages for guest
  Future<GuestDiscussionResponse?> getDiscussionMessages(String token) async {
    try {
      final response = await _apiClient.getGuestDiscussionMessages(token);
      return GuestDiscussionResponse.fromJson(response.data);
    } catch (e) {
      return null;
    }
  }

  // Add discussion message as guest
  Future<GuestDiscussionMessage?> addDiscussionMessage(String token, String message) async {
    try {
      final response = await _apiClient.addGuestDiscussionMessage(token, {'message': message});
      return GuestDiscussionMessage.fromJson(response.data);
    } catch (e) {
      return null;
    }
  }
}

// Provider
final guestProvider = StateNotifierProvider<GuestNotifier, GuestState>((ref) {
  return GuestNotifier(ApiClient());
});
