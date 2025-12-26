import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Models
class Payment {
  final String id;
  final String invoiceId;
  final String? invoiceNumber;
  final double amount;
  final String currency;
  final String paymentMethod;
  final String status;
  final String? paidBy;
  final String? notes;
  final DateTime createdAt;

  Payment({
    required this.id,
    required this.invoiceId,
    this.invoiceNumber,
    required this.amount,
    required this.currency,
    required this.paymentMethod,
    required this.status,
    this.paidBy,
    this.notes,
    required this.createdAt,
  });

  factory Payment.fromJson(Map<String, dynamic> json) {
    return Payment(
      id: json['id'],
      invoiceId: json['invoice_id'],
      invoiceNumber: json['invoice_number'],
      amount: json['amount']?.toDouble() ?? 0.0,
      currency: json['currency'],
      paymentMethod: json['payment_method'],
      status: json['status'],
      paidBy: json['paid_by'],
      notes: json['notes'],
      createdAt: DateTime.parse(json['created_at']),
    );
  }
}

class PaymentStats {
  final int totalPayments;
  final double totalAmount;
  final double avgPayment;
  final Map<String, double> byMethod;

  PaymentStats({
    required this.totalPayments,
    required this.totalAmount,
    required this.avgPayment,
    required this.byMethod,
  });

  factory PaymentStats.fromJson(Map<String, dynamic> json) {
    return PaymentStats(
      totalPayments: json['total_payments'],
      totalAmount: json['total_amount']?.toDouble() ?? 0.0,
      avgPayment: json['avg_payment']?.toDouble() ?? 0.0,
      byMethod: Map<String, double>.from(json['by_method'] ?? {}),
    );
  }
}

// State
class PaymentState {
  final bool isLoading;
  final String? error;
  final List<Payment> payments;
  final PaymentStats? stats;

  PaymentState({
    this.isLoading = false,
    this.error,
    this.payments = const [],
    this.stats,
  });

  PaymentState copyWith({
    bool? isLoading,
    String? error,
    List<Payment>? payments,
    PaymentStats? stats,
  }) {
    return PaymentState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      payments: payments ?? this.payments,
      stats: stats ?? this.stats,
    );
  }
}

// Notifier
class PaymentNotifier extends StateNotifier<PaymentState> {
  final ApiClient _apiClient;

  PaymentNotifier(this._apiClient) : super(PaymentState());

  Future<void> loadPayments({String? status, String? paymentMethod}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getPayments(status: status, paymentMethod: paymentMethod);
      final List<Payment> payments = (response.data as List)
          .map((e) => Payment.fromJson(e))
          .toList();

      state = state.copyWith(isLoading: false, payments: payments);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load payments',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  Future<void> loadStats() async {
    try {
      final response = await _apiClient.getPaymentStats();
      final stats = PaymentStats.fromJson(response.data);
      state = state.copyWith(stats: stats);
    } catch (e) {
      // Non-fatal
    }
  }

  Future<bool> createPayment(Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.createPayment(data);
      state = state.copyWith(isLoading: false);
      await loadPayments();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to create payment',
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

  Future<bool> refundPayment(String id, Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.refundPayment(id, data);
      state = state.copyWith(isLoading: false);
      await loadPayments();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to refund payment',
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

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final paymentProvider = StateNotifierProvider<PaymentNotifier, PaymentState>((ref) {
  return PaymentNotifier(ApiClient());
});
