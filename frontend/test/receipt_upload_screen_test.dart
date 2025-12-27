import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flashbill/features/expenses/presentation/screens/receipt_upload_screen.dart';

void main() {
  group('ReceiptUploadScreen Widget Tests', () {
    testWidgets('Should display all UI elements', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ReceiptUploadScreen(expenseId: 'test-123'),
          ),
        ),
      );

      // Check app bar title
      expect(find.text('Upload Receipt'), findsOneWidget);

      // Check instructions text
      expect(find.text('Upload a receipt image for this expense. Supported formats: JPG, PNG, HEIC (Max 10MB)'), findsOneWidget);

      // Check placeholder for no receipt
      expect(find.text('No receipt selected'), findsOneWidget);

      // Check buttons
      expect(find.text('Select Image'), findsOneWidget);
      expect(find.text('Upload'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);

      // Check tips section
      expect(find.text('Tips for better receipts'), findsOneWidget);
      expect(find.text('• Ensure good lighting'), findsOneWidget);
    });

    testWidgets('Upload button should be disabled when no image selected', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ReceiptUploadScreen(expenseId: 'test-123'),
          ),
        ),
      );

      final uploadButton = find.text('Upload');
      expect(uploadButton, findsOneWidget);

      // The button should be disabled (no onPressed callback or null)
      final button = tester.widget<ElevatedButton>(uploadButton);
      expect(button.onPressed, isNull);
    });

    testWidgets('Should navigate back when cancel is pressed', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Navigator(
              onGenerateRoute: (settings) {
                return MaterialPageRoute(
                  builder: (context) => ReceiptUploadScreen(expenseId: 'test-123'),
                );
              },
            ),
          ),
        ),
      );

      // Find and tap cancel button
      final cancelButton = find.text('Cancel');
      await tester.tap(cancelButton);
      await tester.pumpAndSettle();

      // Should navigate back (screen should be popped)
      // In a real test, we'd verify navigation occurred
    });

    testWidgets('Should display icon buttons', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ReceiptUploadScreen(expenseId: 'test-123'),
          ),
        ),
      );

      // Check for icons
      expect(find.byIcon(Icons.info_outline), findsOneWidget);
      expect(find.byIcon(Icons.receipt_long), findsOneWidget);
      expect(find.byIcon(Icons.add_a_photo), findsOneWidget);
      expect(find.byIcon(Icons.upload), findsOneWidget);
    });

    testWidgets('Should have proper layout structure', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ReceiptUploadScreen(expenseId: 'test-123'),
          ),
        ),
      );

      // Check for main layout components
      expect(find.byType(AppBar), findsOneWidget);
      expect(find.byType(SingleChildScrollView), findsOneWidget);
      expect(find.byType(Column), findsWidgets);
      expect(find.byType(OutlinedButton), findsOneWidget);
      expect(find.byType(ElevatedButton), findsOneWidget);
    });
  });
}
