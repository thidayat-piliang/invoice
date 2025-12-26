import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

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
  final bool isOverdue;

  Invoice({
    required this.id,
    required this.invoiceNumber,
    required this.status,
    required this.clientName,
    required this.issueDate,
    required this.dueDate,
    required this.totalAmount,
    required this.balanceDue,
    required this.isOverdue,
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
      isOverdue: json['is_overdue'] ?? false,
    );
  }
}

// Invoice State
class InvoiceState {
  final bool isLoading;
  final String? error;
  final List<Invoice> invoices;
  final Invoice? selectedInvoice;

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
    Invoice? selectedInvoice,
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

  Future<bool> sendInvoice(String id, Map<String, dynamic> data) async {
    try {
      await _apiClient.sendInvoice(id, data);
      return true;
    } catch (e) {
      return false;
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
