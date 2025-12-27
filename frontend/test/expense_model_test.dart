import 'package:flutter_test/flutter_test.dart';
import 'package:flashbill/features/expenses/presentation/providers/expense_provider.dart';

void main() {
  group('Expense Model Tests', () {
    test('Expense should be created with required fields', () {
      final expense = Expense(
        id: 'test-123',
        description: 'Office Supplies',
        amount: 50.0,
        currency: 'USD',
        category: 'office',
        isTaxDeductible: true,
        date: DateTime(2024, 1, 15),
        createdAt: DateTime(2024, 1, 15),
      );

      expect(expense.id, 'test-123');
      expect(expense.description, 'Office Supplies');
      expect(expense.amount, 50.0);
      expect(expense.currency, 'USD');
      expect(expense.category, 'office');
      expect(expense.isTaxDeductible, true);
    });

    test('Expense should handle optional receipt fields', () {
      final expense = Expense(
        id: 'test-456',
        description: 'Travel Expense',
        amount: 200.0,
        currency: 'USD',
        category: 'travel',
        isTaxDeductible: false,
        receiptPath: '/path/to/receipt.jpg',
        receiptSize: 2.5,
        date: DateTime(2024, 1, 20),
        createdAt: DateTime(2024, 1, 20),
      );

      expect(expense.receiptPath, '/path/to/receipt.jpg');
      expect(expense.receiptSize, 2.5);
      expect(expense.hasReceipt, true);
    });

    test('Expense without receipt should return false for hasReceipt', () {
      final expense = Expense(
        id: 'test-789',
        description: 'Utilities',
        amount: 100.0,
        currency: 'USD',
        category: 'utilities',
        isTaxDeductible: true,
        date: DateTime(2024, 1, 25),
        createdAt: DateTime(2024, 1, 25),
      );

      expect(expense.hasReceipt, false);
    });

    test('Expense should be created from JSON', () {
      final json = {
        'id': 'json-123',
        'description': 'Marketing',
        'amount': 500.0,
        'currency': 'USD',
        'category': 'marketing',
        'is_tax_deductible': true,
        'notes': 'Ad campaign',
        'receipt_path': '/path/to/receipt.png',
        'receipt_size': 1.2,
        'date': '2024-01-30',
        'created_at': '2024-01-30T10:00:00Z',
      };

      final expense = Expense.fromJson(json);

      expect(expense.id, 'json-123');
      expect(expense.description, 'Marketing');
      expect(expense.amount, 500.0);
      expect(expense.notes, 'Ad campaign');
      expect(expense.receiptPath, '/path/to/receipt.png');
      expect(expense.receiptSize, 1.2);
      expect(expense.hasReceipt, true);
    });

    test('Expense should handle missing optional fields in JSON', () {
      final json = {
        'id': 'json-456',
        'description': 'Misc',
        'amount': 25.0,
        'currency': 'USD',
        'category': 'other',
        'is_tax_deductible': false,
        'date': '2024-02-01',
        'created_at': '2024-02-01T12:00:00Z',
      };

      final expense = Expense.fromJson(json);

      expect(expense.notes, isNull);
      expect(expense.receiptPath, isNull);
      expect(expense.receiptSize, isNull);
      expect(expense.hasReceipt, false);
    });

    test('Expense stats should be created from JSON', () {
      final json = {
        'total_expenses': 10,
        'total_amount': 1500.0,
        'tax_deductible_amount': 800.0,
        'by_category': {
          'office': 300.0,
          'travel': 700.0,
          'marketing': 500.0,
        },
      };

      final stats = ExpenseStats.fromJson(json);

      expect(stats.totalExpenses, 10);
      expect(stats.totalAmount, 1500.0);
      expect(stats.taxDeductibleAmount, 800.0);
      expect(stats.byCategory['office'], 300.0);
      expect(stats.byCategory['travel'], 700.0);
    });
  });
}
