import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Models
class OverviewStats {
  final double totalRevenue;
  final double totalExpenses;
  final double netProfit;
  final double outstandingBalance;
  final int totalInvoices;
  final int paidInvoices;
  final int overdueInvoices;

  OverviewStats({
    required this.totalRevenue,
    required this.totalExpenses,
    required this.netProfit,
    required this.outstandingBalance,
    required this.totalInvoices,
    required this.paidInvoices,
    required this.overdueInvoices,
  });

  factory OverviewStats.fromJson(Map<String, dynamic> json) {
    return OverviewStats(
      totalRevenue: json['total_revenue']?.toDouble() ?? 0.0,
      totalExpenses: json['total_expenses']?.toDouble() ?? 0.0,
      netProfit: json['net_profit']?.toDouble() ?? 0.0,
      outstandingBalance: json['outstanding_balance']?.toDouble() ?? 0.0,
      totalInvoices: json['total_invoices'] ?? 0,
      paidInvoices: json['paid_invoices'] ?? 0,
      overdueInvoices: json['overdue_invoices'] ?? 0,
    );
  }
}

class IncomeReport {
  final double totalIncome;
  final List<Map<String, dynamic>> monthlyData;
  final List<Map<String, dynamic>> topInvoices;

  IncomeReport({
    required this.totalIncome,
    required this.monthlyData,
    required this.topInvoices,
  });

  factory IncomeReport.fromJson(Map<String, dynamic> json) {
    return IncomeReport(
      totalIncome: json['total_income']?.toDouble() ?? 0.0,
      monthlyData: List<Map<String, dynamic>>.from(json['monthly_data'] ?? []),
      topInvoices: List<Map<String, dynamic>>.from(json['top_invoices'] ?? []),
    );
  }
}

class ExpenseReport {
  final double totalExpenses;
  final List<Map<String, dynamic>> monthlyData;
  final List<Map<String, dynamic>> byCategory;

  ExpenseReport({
    required this.totalExpenses,
    required this.monthlyData,
    required this.byCategory,
  });

  factory ExpenseReport.fromJson(Map<String, dynamic> json) {
    return ExpenseReport(
      totalExpenses: json['total_expenses']?.toDouble() ?? 0.0,
      monthlyData: List<Map<String, dynamic>>.from(json['monthly_data'] ?? []),
      byCategory: List<Map<String, dynamic>>.from(json['by_category'] ?? []),
    );
  }
}

class TaxReport {
  final double totalTaxableIncome;
  final double totalTaxDeductible;
  final double estimatedTax;
  final List<Map<String, dynamic>> taxByMonth;

  TaxReport({
    required this.totalTaxableIncome,
    required this.totalTaxDeductible,
    required this.estimatedTax,
    required this.taxByMonth,
  });

  factory TaxReport.fromJson(Map<String, dynamic> json) {
    return TaxReport(
      totalTaxableIncome: json['total_taxable_income']?.toDouble() ?? 0.0,
      totalTaxDeductible: json['total_tax_deductible']?.toDouble() ?? 0.0,
      estimatedTax: json['estimated_tax']?.toDouble() ?? 0.0,
      taxByMonth: List<Map<String, dynamic>>.from(json['tax_by_month'] ?? []),
    );
  }
}

class AgingReport {
  final double current;
  final double days1_30;
  final double days31_60;
  final double days61_90;
  final double over90;
  final List<Map<String, dynamic>> overdueInvoices;

  AgingReport({
    required this.current,
    required this.days1_30,
    required this.days31_60,
    required this.days61_90,
    required this.over90,
    required this.overdueInvoices,
  });

  factory AgingReport.fromJson(Map<String, dynamic> json) {
    return AgingReport(
      current: json['current']?.toDouble() ?? 0.0,
      days1_30: json['days_1_30']?.toDouble() ?? 0.0,
      days31_60: json['days_31_60']?.toDouble() ?? 0.0,
      days61_90: json['days_61_90']?.toDouble() ?? 0.0,
      over90: json['over_90']?.toDouble() ?? 0.0,
      overdueInvoices: List<Map<String, dynamic>>.from(json['overdue_invoices'] ?? []),
    );
  }
}

// State
class ReportState {
  final bool isLoading;
  final String? error;
  final OverviewStats? overview;
  final IncomeReport? income;
  final ExpenseReport? expense;
  final TaxReport? tax;
  final AgingReport? aging;

  ReportState({
    this.isLoading = false,
    this.error,
    this.overview,
    this.income,
    this.expense,
    this.tax,
    this.aging,
  });

  ReportState copyWith({
    bool? isLoading,
    String? error,
    OverviewStats? overview,
    IncomeReport? income,
    ExpenseReport? expense,
    TaxReport? tax,
    AgingReport? aging,
  }) {
    return ReportState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      overview: overview ?? this.overview,
      income: income ?? this.income,
      expense: expense ?? this.expense,
      tax: tax ?? this.tax,
      aging: aging ?? this.aging,
    );
  }
}

// Notifier
class ReportNotifier extends StateNotifier<ReportState> {
  final ApiClient _apiClient;

  ReportNotifier(this._apiClient) : super(ReportState());

  Future<void> loadOverview() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getOverviewStats();
      final overview = OverviewStats.fromJson(response.data);
      state = state.copyWith(isLoading: false, overview: overview);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load overview',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<void> loadIncomeReport({String? startDate, String? endDate}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getIncomeReport(startDate: startDate, endDate: endDate);
      final income = IncomeReport.fromJson(response.data);
      state = state.copyWith(isLoading: false, income: income);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load income report',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<void> loadExpenseReport({String? startDate, String? endDate}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getExpenseReport(startDate: startDate, endDate: endDate);
      final expense = ExpenseReport.fromJson(response.data);
      state = state.copyWith(isLoading: false, expense: expense);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load expense report',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<void> loadTaxReport({String? startDate, String? endDate}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getTaxReport(startDate: startDate, endDate: endDate);
      final tax = TaxReport.fromJson(response.data);
      state = state.copyWith(isLoading: false, tax: tax);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load tax report',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<void> loadAgingReport() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getAgingReport();
      final aging = AgingReport.fromJson(response.data);
      state = state.copyWith(isLoading: false, aging: aging);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load aging report',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<bool> exportReport(String format, String type) async {
    try {
      await _apiClient.exportReport(format, type);
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
final reportProvider = StateNotifierProvider<ReportNotifier, ReportState>((ref) {
  return ReportNotifier(ApiClient());
});
