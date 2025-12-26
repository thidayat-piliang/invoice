import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Dashboard Metrics
class DashboardMetrics {
  final double totalRevenue;
  final double totalOutstanding;
  final int paidInvoices;
  final int overdueInvoices;
  final double totalExpenses;
  final double netProfit;

  DashboardMetrics({
    required this.totalRevenue,
    required this.totalOutstanding,
    required this.paidInvoices,
    required this.overdueInvoices,
    required this.totalExpenses,
    required this.netProfit,
  });

  factory DashboardMetrics.fromJson(Map<String, dynamic> json) {
    return DashboardMetrics(
      totalRevenue: json['total_revenue']?.toDouble() ?? 0.0,
      totalOutstanding: json['total_outstanding']?.toDouble() ?? 0.0,
      paidInvoices: json['paid_invoices'] ?? 0,
      overdueInvoices: json['overdue_invoices'] ?? 0,
      totalExpenses: json['total_expenses']?.toDouble() ?? 0.0,
      netProfit: json['net_profit']?.toDouble() ?? 0.0,
    );
  }
}

// Dashboard State
class DashboardState {
  final bool isLoading;
  final String? error;
  final DashboardMetrics? metrics;

  DashboardState({
    this.isLoading = false,
    this.error,
    this.metrics,
  });

  DashboardState copyWith({
    bool? isLoading,
    String? error,
    DashboardMetrics? metrics,
  }) {
    return DashboardState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      metrics: metrics ?? this.metrics,
    );
  }
}

// Dashboard Notifier
class DashboardNotifier extends StateNotifier<DashboardState> {
  final ApiClient _apiClient;

  DashboardNotifier(this._apiClient) : super(DashboardState());

  Future<void> loadMetrics() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getDashboardOverview();
      final metrics = DashboardMetrics.fromJson(response.data);

      state = state.copyWith(
        isLoading: false,
        metrics: metrics,
      );
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load metrics',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final dashboardProvider = StateNotifierProvider<DashboardNotifier, DashboardState>((ref) {
  return DashboardNotifier(ApiClient());
});
