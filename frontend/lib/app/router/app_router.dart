import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../features/auth/presentation/screens/login_screen.dart';
import '../../features/auth/presentation/screens/register_screen.dart';
import '../../features/auth/presentation/screens/forgot_password_screen.dart';
import '../../features/dashboard/presentation/dashboard_screen.dart';
import '../../features/invoices/presentation/screens/invoice_list_screen.dart';
import '../../features/invoices/presentation/screens/create_invoice_screen.dart';
import '../../features/invoices/presentation/screens/invoice_detail_screen.dart';
import '../../features/clients/presentation/screens/client_list_screen.dart';
import '../../features/settings/presentation/screens/settings_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/auth/login',
    redirect: (context, state) {
      // TODO: Implement auth state check
      final isAuthenticated = false; // Placeholder

      if (isAuthenticated && state.location.startsWith('/auth')) {
        return '/dashboard';
      } else if (!isAuthenticated && !state.location.startsWith('/auth')) {
        return '/auth/login';
      }
      return null;
    },
    routes: [
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
      GoRoute(
        path: '/dashboard',
        builder: (context, state) => const DashboardScreen(),
      ),
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
        ],
      ),
      GoRoute(
        path: '/clients',
        builder: (context, state) => const ClientListScreen(),
      ),
      GoRoute(
        path: '/settings',
        builder: (context, state) => const SettingsScreen(),
      ),
    ],
  );
});
