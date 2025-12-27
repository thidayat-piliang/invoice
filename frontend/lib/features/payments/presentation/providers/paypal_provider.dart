import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

/// PayPal Order Status
enum PayPalOrderStatus {
  created,
  approved,
  completed,
  failed,
  cancelled,
  refunded,
  unknown;

  static PayPalOrderStatus fromString(String status) {
    switch (status.toLowerCase()) {
      case 'created':
        return PayPalOrderStatus.created;
      case 'approved':
        return PayPalOrderStatus.approved;
      case 'completed':
        return PayPalOrderStatus.completed;
      case 'failed':
        return PayPalOrderStatus.failed;
      case 'cancelled':
        return PayPalOrderStatus.cancelled;
      case 'refunded':
        return PayPalOrderStatus.refunded;
      default:
        return PayPalOrderStatus.unknown;
    }
  }

  String get displayText {
    switch (this) {
      case PayPalOrderStatus.created:
        return 'Created';
      case PayPalOrderStatus.approved:
        return 'Approved';
      case PayPalOrderStatus.completed:
        return 'Completed';
      case PayPalOrderStatus.failed:
        return 'Failed';
      case PayPalOrderStatus.cancelled:
        return 'Cancelled';
      case PayPalOrderStatus.refunded:
        return 'Refunded';
      case PayPalOrderStatus.unknown:
        return 'Unknown';
    }
  }

  Color get color {
    switch (this) {
      case PayPalOrderStatus.completed:
        return const Color(0xFF4CAF50);
      case PayPalOrderStatus.approved:
        return const Color(0xFF2196F3);
      case PayPalOrderStatus.created:
        return const Color(0xFFFF9800);
      case PayPalOrderStatus.failed:
      case PayPalOrderStatus.cancelled:
        return const Color(0xFFF44336);
      case PayPalOrderStatus.refunded:
        return const Color(0xFF9E9E9E);
      case PayPalOrderStatus.unknown:
        return const Color(0xFF607D8B);
    }
  }
}

/// PayPal Order Model
class PayPalOrder {
  final String id;
  final String? paypalOrderId;
  final double amount;
  final String currency;
  final PayPalOrderStatus status;
  final String description;
  final DateTime createdAt;
  final DateTime? updatedAt;
  final String? invoiceId;
  final String? customerEmail;

  PayPalOrder({
    required this.id,
    this.paypalOrderId,
    required this.amount,
    required this.currency,
    required this.status,
    required this.description,
    required this.createdAt,
    this.updatedAt,
    this.invoiceId,
    this.customerEmail,
  });

  factory PayPalOrder.fromJson(Map<String, dynamic> json) {
    return PayPalOrder(
      id: json['id'],
      paypalOrderId: json['paypal_order_id'],
      amount: json['amount']?.toDouble() ?? 0.0,
      currency: json['currency'] ?? 'USD',
      status: PayPalOrderStatus.fromString(json['status'] ?? 'unknown'),
      description: json['description'] ?? '',
      createdAt: DateTime.parse(json['created_at']),
      updatedAt: json['updated_at'] != null ? DateTime.parse(json['updated_at']) : null,
      invoiceId: json['invoice_id'],
      customerEmail: json['customer_email'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'paypal_order_id': paypalOrderId,
      'amount': amount,
      'currency': currency,
      'status': status.toString().split('.').last,
      'description': description,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt?.toIso8601String(),
      'invoice_id': invoiceId,
      'customer_email': customerEmail,
    };
  }
}

/// PayPal Refund Model
class PayPalRefund {
  final String id;
  final String orderId;
  final double amount;
  final String currency;
  final String status;
  final DateTime createdAt;
  final String? reason;

  PayPalRefund({
    required this.id,
    required this.orderId,
    required this.amount,
    required this.currency,
    required this.status,
    required this.createdAt,
    this.reason,
  });

  factory PayPalRefund.fromJson(Map<String, dynamic> json) {
    return PayPalRefund(
      id: json['id'],
      orderId: json['order_id'],
      amount: json['amount']?.toDouble() ?? 0.0,
      currency: json['currency'] ?? 'USD',
      status: json['status'] ?? 'pending',
      createdAt: DateTime.parse(json['created_at']),
      reason: json['reason'],
    );
  }
}

/// PayPal State
class PayPalState {
  final bool isLoading;
  final String? error;
  final PayPalOrder? currentOrder;
  final List<PayPalOrder> orders;
  final PayPalRefund? lastRefund;

  PayPalState({
    this.isLoading = false,
    this.error,
    this.currentOrder,
    this.orders = const [],
    this.lastRefund,
  });

  PayPalState copyWith({
    bool? isLoading,
    String? error,
    PayPalOrder? currentOrder,
    List<PayPalOrder>? orders,
    PayPalRefund? lastRefund,
  }) {
    return PayPalState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      currentOrder: currentOrder ?? this.currentOrder,
      orders: orders ?? this.orders,
      lastRefund: lastRefund ?? this.lastRefund,
    );
  }
}

/// PayPal Notifier
class PayPalNotifier extends StateNotifier<PayPalState> {
  final ApiClient _apiClient;

  PayPalNotifier(this._apiClient) : super(PayPalState());

  /// Create a PayPal order for invoice payment
  Future<PayPalOrder?> createOrder({
    required double amount,
    required String currency,
    required String description,
    String? invoiceId,
    String? customerEmail,
  }) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.createPayPalOrder({
        'amount': amount,
        'currency': currency,
        'description': description,
        if (invoiceId != null) 'invoice_id': invoiceId,
        if (customerEmail != null) 'customer_email': customerEmail,
      });

      final order = PayPalOrder.fromJson(response.data);
      state = state.copyWith(
        isLoading: false,
        currentOrder: order,
        orders: [order, ...state.orders],
      );

      return order;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: _getErrorMessage(e),
      );
      return null;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return null;
    }
  }

  /// Refund a PayPal payment
  Future<PayPalRefund?> refundOrder({
    required String orderId,
    required double amount,
    String? reason,
  }) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.refundPayPalPayment(orderId, {
        'amount': amount,
        if (reason != null) 'reason': reason,
      });

      final refund = PayPalRefund.fromJson(response.data);
      state = state.copyWith(
        isLoading: false,
        lastRefund: refund,
      );

      return refund;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: _getErrorMessage(e),
      );
      return null;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return null;
    }
  }

  /// Get PayPal order status
  Future<PayPalOrder?> getOrderStatus(String orderId) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getPayPalStatus(orderId);
      final order = PayPalOrder.fromJson(response.data);

      state = state.copyWith(
        isLoading: false,
        currentOrder: order,
      );

      return order;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: _getErrorMessage(e),
      );
      return null;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
      return null;
    }
  }

  /// Load all PayPal orders
  Future<void> loadOrders() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      // Note: This endpoint would need to be added to the backend
      // For now, we'll return an empty list
      state = state.copyWith(
        isLoading: false,
        orders: [],
      );
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: _getErrorMessage(e),
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  /// Clear error
  void clearError() {
    state = state.copyWith(error: null);
  }

  /// Reset state
  void reset() {
    state = PayPalState();
  }

  String _getErrorMessage(DioException e) {
    return _apiClient.getErrorMessage(e);
  }
}

/// PayPal Provider
final paypalProvider = StateNotifierProvider<PayPalNotifier, PayPalState>((ref) {
  return PayPalNotifier(ApiClient());
});
