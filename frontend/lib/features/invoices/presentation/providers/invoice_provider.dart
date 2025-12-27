import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Discussion Message Model
class DiscussionMessage {
  final String id;
  final String invoiceId;
  final String senderType; // 'seller' or 'buyer'
  final String message;
  final DateTime createdAt;

  DiscussionMessage({
    required this.id,
    required this.invoiceId,
    required this.senderType,
    required this.message,
    required this.createdAt,
  });

  factory DiscussionMessage.fromJson(Map<String, dynamic> json) {
    return DiscussionMessage(
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

// Discussion Response Model
class DiscussionResponse {
  final List<DiscussionMessage> messages;

  DiscussionResponse({required this.messages});

  factory DiscussionResponse.fromJson(Map<String, dynamic> json) {
    final messagesList = json['messages'] as List;
    return DiscussionResponse(
      messages: messagesList.map((e) => DiscussionMessage.fromJson(e)).toList(),
    );
  }

  int get count => messages.length;
  bool get isEmpty => messages.isEmpty;
}

// Invoice Item Model
class InvoiceItem {
  final String description;
  final double quantity;
  final double unitPrice;
  final double amount;

  InvoiceItem({
    required this.description,
    required this.quantity,
    required this.unitPrice,
    required this.amount,
  });

  factory InvoiceItem.fromJson(Map<String, dynamic> json) {
    return InvoiceItem(
      description: json['description'],
      quantity: json['quantity']?.toDouble() ?? 0.0,
      unitPrice: json['unit_price']?.toDouble() ?? 0.0,
      amount: json['amount']?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'description': description,
      'quantity': quantity,
      'unit_price': unitPrice,
      'amount': amount,
    };
  }
}

// Invoice Model
class Invoice {
  final String id;
  final String invoiceNumber;
  final String status;
  final String clientName;
  final DateTime issueDate;
  final DateTime dueDate;
  final double totalAmount;
  final double balanceDue;
  final double amountPaid;
  final bool isOverdue;
  final DateTime? viewedAt;
  final DateTime? sentAt;
  final String? guestPaymentToken;
  final bool allowPartialPayment;
  final double? minPaymentAmount;
  final int partialPaymentCount;

  Invoice({
    required this.id,
    required this.invoiceNumber,
    required this.status,
    required this.clientName,
    required this.issueDate,
    required this.dueDate,
    required this.totalAmount,
    required this.balanceDue,
    required this.amountPaid,
    required this.isOverdue,
    this.viewedAt,
    this.sentAt,
    this.guestPaymentToken,
    this.allowPartialPayment = true,
    this.minPaymentAmount,
    this.partialPaymentCount = 0,
  });

  factory Invoice.fromJson(Map<String, dynamic> json) {
    return Invoice(
      id: json['id'],
      invoiceNumber: json['invoice_number'],
      status: json['status'],
      clientName: json['client_name'],
      issueDate: DateTime.parse(json['issue_date']),
      dueDate: DateTime.parse(json['due_date']),
      totalAmount: json['total_amount']?.toDouble() ?? 0.0,
      balanceDue: json['balance_due']?.toDouble() ?? 0.0,
      amountPaid: json['amount_paid']?.toDouble() ?? 0.0,
      isOverdue: json['is_overdue'] ?? false,
      viewedAt: json['viewed_at'] != null ? DateTime.parse(json['viewed_at']) : null,
      sentAt: json['sent_at'] != null ? DateTime.parse(json['sent_at']) : null,
      guestPaymentToken: json['guest_payment_token'],
      allowPartialPayment: json['allow_partial_payment'] ?? true,
      minPaymentAmount: json['min_payment_amount']?.toDouble(),
      partialPaymentCount: json['partial_payment_count'] ?? 0,
    );
  }

  bool get isViewed => viewedAt != null;
  bool get hasGuestLink => guestPaymentToken != null;
  bool get isPartial => status.toLowerCase() == 'partial';
}

// Invoice Detail Model
class InvoiceDetail {
  final String id;
  final String invoiceNumber;
  final String status;
  final String clientId;
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
  final List<InvoiceItem> items;
  final String? notes;
  final String? terms;
  final DateTime? viewedAt;
  final DateTime? sentAt;
  final String? guestPaymentToken;
  final bool allowPartialPayment;
  final double? minPaymentAmount;
  final int partialPaymentCount;

  InvoiceDetail({
    required this.id,
    required this.invoiceNumber,
    required this.status,
    required this.clientId,
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
    this.viewedAt,
    this.sentAt,
    this.guestPaymentToken,
    this.allowPartialPayment = true,
    this.minPaymentAmount,
    this.partialPaymentCount = 0,
  });

  factory InvoiceDetail.fromJson(Map<String, dynamic> json) {
    return InvoiceDetail(
      id: json['id'],
      invoiceNumber: json['invoice_number'],
      status: json['status'],
      clientId: json['client_id'],
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
      items: (json['items'] as List).map((e) => InvoiceItem.fromJson(e)).toList(),
      notes: json['notes'],
      terms: json['terms'],
      viewedAt: json['viewed_at'] != null ? DateTime.parse(json['viewed_at']) : null,
      sentAt: json['sent_at'] != null ? DateTime.parse(json['sent_at']) : null,
      guestPaymentToken: json['guest_payment_token'],
      allowPartialPayment: json['allow_partial_payment'] ?? true,
      minPaymentAmount: json['min_payment_amount']?.toDouble(),
      partialPaymentCount: json['partial_payment_count'] ?? 0,
    );
  }

  bool get isViewed => viewedAt != null;
  bool get isPartial => status.toLowerCase() == 'partial';
  bool get canAcceptPartialPayment => allowPartialPayment && balanceDue > 0;
}

// Invoice State
class InvoiceState {
  final bool isLoading;
  final String? error;
  final List<Invoice> invoices;
  final InvoiceDetail? selectedInvoice;

  InvoiceState({
    this.isLoading = false,
    this.error,
    this.invoices = const [],
    this.selectedInvoice,
  });

  InvoiceState copyWith({
    bool? isLoading,
    String? error,
    List<Invoice>? invoices,
    InvoiceDetail? selectedInvoice,
  }) {
    return InvoiceState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      invoices: invoices ?? this.invoices,
      selectedInvoice: selectedInvoice ?? this.selectedInvoice,
    );
  }

  List<Invoice> get overdueInvoices => invoices.where((i) => i.isOverdue).toList();
  double get totalOutstanding => invoices.fold(0.0, (sum, i) => sum + i.balanceDue);
}

// Invoice Notifier
class InvoiceNotifier extends StateNotifier<InvoiceState> {
  final ApiClient _apiClient;

  InvoiceNotifier(this._apiClient) : super(InvoiceState());

  Future<void> loadInvoices({String? status, String? search}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getInvoices(status: status, search: search);
      final List<dynamic> data = response.data;

      final invoices = data.map((json) => Invoice.fromJson(json)).toList();

      state = state.copyWith(
        isLoading: false,
        invoices: invoices,
      );
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load invoices',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  Future<void> loadInvoice(String id) async {
    state = state.copyWith(isLoading: true, error: null, selectedInvoice: null);

    try {
      final response = await _apiClient.getInvoice(id);
      final invoice = InvoiceDetail.fromJson(response.data);

      state = state.copyWith(
        isLoading: false,
        selectedInvoice: invoice,
      );
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load invoice',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  Future<bool> createInvoice(Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.createInvoice(data);
      final invoice = Invoice.fromJson(response.data);

      state = state.copyWith(
        isLoading: false,
        invoices: [invoice, ...state.invoices],
      );

      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to create invoice',
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

  Future<bool> updateInvoice(String id, Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateInvoice(id, data);
      state = state.copyWith(isLoading: false);
      await loadInvoices();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update invoice',
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

  Future<bool> deleteInvoice(String id) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.deleteInvoice(id);
      state = state.copyWith(
        isLoading: false,
        invoices: state.invoices.where((i) => i.id != id).toList(),
      );
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to delete invoice',
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

  Future<bool> sendInvoice(String id, Map<String, dynamic> data) async {
    try {
      await _apiClient.sendInvoice(id, data);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> sendReminder(String id, Map<String, dynamic> data) async {
    try {
      await _apiClient.sendReminder(id, data);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> recordPayment(String id, Map<String, dynamic> data) async {
    try {
      await _apiClient.recordPayment(id, data);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<List<int>?> getInvoicePdf(String id) async {
    try {
      final response = await _apiClient.getInvoicePdf(id);
      return response.data as List<int>;
    } catch (e) {
      return null;
    }
  }

  Future<bool> sendInvoiceWhatsapp(String id) async {
    try {
      await _apiClient.sendInvoiceWhatsapp(id);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> markInvoiceViewed(String id) async {
    try {
      await _apiClient.markInvoiceViewed(id);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> sendPaymentConfirmation(String id, Map<String, dynamic> data) async {
    try {
      await _apiClient.sendPaymentConfirmation(id, data);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<DiscussionResponse?> getDiscussionMessages(String id) async {
    try {
      final response = await _apiClient.getDiscussionMessages(id);
      return DiscussionResponse.fromJson(response.data);
    } catch (e) {
      return null;
    }
  }

  Future<DiscussionMessage?> addDiscussionMessage(String id, String message) async {
    try {
      final response = await _apiClient.addDiscussionMessage(id, {'message': message});
      return DiscussionMessage.fromJson(response.data);
    } catch (e) {
      return null;
    }
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final invoiceProvider = StateNotifierProvider<InvoiceNotifier, InvoiceState>((ref) {
  return InvoiceNotifier(ApiClient());
});
