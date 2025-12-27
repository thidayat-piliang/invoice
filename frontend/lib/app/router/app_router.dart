import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../features/auth/presentation/providers/auth_provider.dart';
import '../../features/auth/presentation/screens/login_screen.dart';
import '../../features/auth/presentation/screens/register_screen.dart';
import '../../features/auth/presentation/screens/forgot_password_screen.dart';
import '../../features/dashboard/presentation/dashboard_screen.dart';
import '../../features/invoices/presentation/screens/invoice_list_screen.dart';
import '../../features/invoices/presentation/screens/create_invoice_screen.dart';
import '../../features/invoices/presentation/screens/invoice_detail_screen.dart';
import '../../features/invoices/presentation/screens/pdf_preview_screen.dart';
import '../../features/clients/presentation/screens/client_list_screen.dart';
import '../../features/clients/presentation/screens/client_form_screen.dart';
import '../../features/clients/presentation/screens/client_detail_screen.dart';
import '../../features/payments/presentation/screens/payment_list_screen.dart';
import '../../features/payments/presentation/screens/create_payment_screen.dart';
import '../../features/payments/presentation/screens/paypal_checkout_screen.dart';
import '../../features/payments/presentation/screens/paypal_refund_screen.dart';
import '../../features/expenses/presentation/screens/expense_list_screen.dart';
import '../../features/expenses/presentation/screens/expense_form_screen.dart';
import '../../features/expenses/presentation/screens/receipt_upload_screen.dart';
import '../../features/reports/presentation/screens/reports_screen.dart';
import '../../features/reports/presentation/screens/income_report_screen.dart';
import '../../features/reports/presentation/screens/report_detail_screen.dart';
import '../../features/settings/presentation/screens/settings_screen.dart';
import '../../features/settings/presentation/screens/business_settings_screen.dart';
import '../../features/settings/presentation/screens/tax_settings_screen.dart';
import '../../features/settings/presentation/screens/notification_settings_screen.dart';
import '../../features/settings/presentation/screens/invoice_settings_screen.dart';

// Shell routes for bottom navigation
final _shellRoutes = [
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/dashboard',
        builder: (context, state) => const DashboardScreen(),
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/clients',
        builder: (context, state) => const ClientListScreen(),
        routes: [
          GoRoute(
            path: 'create',
            builder: (context, state) => const ClientFormScreen(),
          ),
          GoRoute(
            path: 'edit/:id',
            builder: (context, state) {
              final clientId = state.pathParameters['id']!;
              return ClientFormScreen(clientId: clientId);
            },
          ),
          GoRoute(
            path: ':id',
            builder: (context, state) {
              final clientId = state.pathParameters['id']!;
              return ClientDetailScreen(clientId: clientId);
            },
          ),
        ],
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/invoices',
        builder: (context, state) => const InvoiceListScreen(),
        routes: [
          GoRoute(
            path: 'create',
            builder: (context, state) => const CreateInvoiceScreen(),
          ),
          GoRoute(
            path: ':id',
            builder: (context, state) {
              final invoiceId = state.pathParameters['id']!;
              return InvoiceDetailScreen(invoiceId: invoiceId);
            },
          ),
          GoRoute(
            path: 'pdf-preview/:id',
            builder: (context, state) {
              final invoiceId = state.pathParameters['id']!;
              return PdfPreviewScreen(invoiceId: invoiceId);
            },
          ),
        ],
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/payments',
        builder: (context, state) => const PaymentListScreen(),
        routes: [
          GoRoute(
            path: 'create',
            builder: (context, state) => const CreatePaymentScreen(),
          ),
          GoRoute(
            path: 'paypal-checkout',
            builder: (context, state) {
              final extra = state.extra as Map<String, dynamic>?;
              return PayPalCheckoutScreen(
                amount: extra?['amount'] ?? 0.0,
                currency: extra?['currency'] ?? 'USD',
                description: extra?['description'] ?? '',
                invoiceId: extra?['invoiceId'],
                customerEmail: extra?['customerEmail'],
              );
            },
          ),
          GoRoute(
            path: 'paypal-refund/:orderId',
            builder: (context, state) {
              final orderId = state.pathParameters['orderId']!;
              final maxAmount = (state.extra as double?) ?? 0.0;
              return PayPalRefundScreen(
                orderId: orderId,
                maxAmount: maxAmount,
              );
            },
          ),
        ],
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/expenses',
        builder: (context, state) => const ExpenseListScreen(),
        routes: [
          GoRoute(
            path: 'create',
            builder: (context, state) => const ExpenseFormScreen(),
          ),
          GoRoute(
            path: 'edit/:id',
            builder: (context, state) {
              final expenseId = state.pathParameters['id']!;
              return ExpenseFormScreen(expenseId: expenseId);
            },
          ),
          GoRoute(
            path: 'receipt/:id',
            builder: (context, state) {
              final expenseId = state.pathParameters['id']!;
              return ReceiptUploadScreen(expenseId: expenseId);
            },
          ),
        ],
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/reports',
        builder: (context, state) => const ReportsScreen(),
        routes: [
          GoRoute(
            path: 'income',
            builder: (context, state) => const IncomeReportScreen(),
          ),
          GoRoute(
            path: ':type',
            builder: (context, state) {
              final type = state.pathParameters['type']!;
              return ReportDetailScreen(reportType: type);
            },
          ),
        ],
      ),
    ],
  ),
  StatefulShellBranch(
    routes: [
      GoRoute(
        path: '/settings',
        builder: (context, state) => const SettingsScreen(),
        routes: [
          GoRoute(
            path: 'business',
            builder: (context, state) => const BusinessSettingsScreen(),
          ),
          GoRoute(
            path: 'tax',
            builder: (context, state) => const TaxSettingsScreen(),
          ),
          GoRoute(
            path: 'notifications',
            builder: (context, state) => const NotificationSettingsScreen(),
          ),
          GoRoute(
            path: 'invoice',
            builder: (context, state) => const InvoiceSettingsScreen(),
          ),
        ],
      ),
    ],
  ),
];

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/auth/login',
    redirect: (context, state) {
      final authState = ref.read(authProvider);
      final isAuthenticated = authState.isAuthenticated;

      if (isAuthenticated && state.location.startsWith('/auth')) {
        return '/dashboard';
      } else if (!isAuthenticated && !state.location.startsWith('/auth')) {
        return '/auth/login';
      }
      return null;
    },
    routes: [
      // Auth routes
      GoRoute(
        path: '/auth/login',
        builder: (context, state) => const LoginScreen(),
      ),
      GoRoute(
        path: '/auth/register',
        builder: (context, state) => const RegisterScreen(),
      ),
      GoRoute(
        path: '/auth/forgot-password',
        builder: (context, state) => const ForgotPasswordScreen(),
      ),

      // Main app with bottom navigation
      StatefulShellRoute(
        branches: _shellRoutes,
        builder: (context, state, navigationShell) {
          return ScaffoldWithNavBar(navigationShell: navigationShell);
        },
      ),
    ],
  );
});

// Scaffold with Bottom Navigation Bar
class ScaffoldWithNavBar extends StatelessWidget {
  final StatefulNavigationShell navigationShell;

  const ScaffoldWithNavBar({
    super.key,
    required this.navigationShell,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: navigationShell,
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: navigationShell.currentIndex,
        onTap: (index) {
          navigationShell.goBranch(index);
        },
        type: BottomNavigationBarType.fixed,
        selectedItemColor: Colors.blue,
        unselectedItemColor: Colors.grey,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.dashboard),
            label: 'Dashboard',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.people),
            label: 'Clients',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.description),
            label: 'Invoices',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.payment),
            label: 'Payments',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.receipt_long),
            label: 'Expenses',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.analytics),
            label: 'Reports',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings),
            label: 'Settings',
          ),
        ],
      ),
    );
  }
}
