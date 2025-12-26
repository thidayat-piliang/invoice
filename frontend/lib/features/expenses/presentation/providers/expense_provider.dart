import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Models
class Expense {
  final String id;
  final String description;
  final double amount;
  final String currency;
  final String category;
  final bool isTaxDeductible;
  final String? notes;
  final DateTime date;
  final DateTime createdAt;

  Expense({
    required this.id,
    required this.description,
    required this.amount,
    required this.currency,
    required this.category,
    required this.isTaxDeductible,
    this.notes,
    required this.date,
    required this.createdAt,
  });

  factory Expense.fromJson(Map<String, dynamic> json) {
    return Expense(
      id: json['id'],
      description: json['description'],
      amount: json['amount']?.toDouble() ?? 0.0,
      currency: json['currency'],
      category: json['category'],
      isTaxDeductible: json['is_tax_deductible'] ?? false,
      notes: json['notes'],
      date: DateTime.parse(json['date']),
      createdAt: DateTime.parse(json['created_at']),
    );
  }
}

class ExpenseStats {
  final int totalExpenses;
  final double totalAmount;
  final double taxDeductibleAmount;
  final Map<String, double> byCategory;

  ExpenseStats({
    required this.totalExpenses,
    required this.totalAmount,
    required this.taxDeductibleAmount,
    required this.byCategory,
  });

  factory ExpenseStats.fromJson(Map<String, dynamic> json) {
    return ExpenseStats(
      totalExpenses: json['total_expenses'],
      totalAmount: json['total_amount']?.toDouble() ?? 0.0,
      taxDeductibleAmount: json['tax_deductible_amount']?.toDouble() ?? 0.0,
      byCategory: Map<String, double>.from(json['by_category'] ?? {}),
    );
  }
}

// State
class ExpenseState {
  final bool isLoading;
  final String? error;
  final List<Expense> expenses;
  final ExpenseStats? stats;

  ExpenseState({
    this.isLoading = false,
    this.error,
    this.expenses = const [],
    this.stats,
  });

  ExpenseState copyWith({
    bool? isLoading,
    String? error,
    List<Expense>? expenses,
    ExpenseStats? stats,
  }) {
    return ExpenseState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      expenses: expenses ?? this.expenses,
      stats: stats ?? this.stats,
    );
  }
}

// Notifier
class ExpenseNotifier extends StateNotifier<ExpenseState> {
  final ApiClient _apiClient;

  ExpenseNotifier(this._apiClient) : super(ExpenseState());

  Future<void> loadExpenses({String? category, String? search}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getExpenses(category: category, search: search);
      final List<Expense> expenses = (response.data as List)
          .map((e) => Expense.fromJson(e))
          .toList();

      state = state.copyWith(isLoading: false, expenses: expenses);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load expenses',
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
      final response = await _apiClient.getExpenseStats();
      final stats = ExpenseStats.fromJson(response.data);
      state = state.copyWith(stats: stats);
    } catch (e) {
      // Non-fatal
    }
  }

  Future<bool> createExpense(Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.createExpense(data);
      state = state.copyWith(isLoading: false);
      await loadExpenses();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to create expense',
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

  Future<bool> updateExpense(String id, Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateExpense(id, data);
      state = state.copyWith(isLoading: false);
      await loadExpenses();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update expense',
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

  Future<bool> deleteExpense(String id) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.deleteExpense(id);
      state = state.copyWith(isLoading: false);
      await loadExpenses();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to delete expense',
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
final expenseProvider = StateNotifierProvider<ExpenseNotifier, ExpenseState>((ref) {
  return ExpenseNotifier(ApiClient());
});
