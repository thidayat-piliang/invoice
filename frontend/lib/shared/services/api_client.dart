import 'package:dio/dio.dart';
import 'package:pretty_dio_logger/pretty_dio_logger.dart';
import 'local_storage.dart';

class ApiClient {
  static final ApiClient _instance = ApiClient._internal();
  factory ApiClient() => _instance;

  late Dio dio;
  final LocalStorage _storage = LocalStorage();

  ApiClient._internal() {
    final options = BaseOptions(
      baseUrl: 'http://localhost:3000/api/v1',
      connectTimeout: const Duration(seconds: 30),
      receiveTimeout: const Duration(seconds: 30),
      headers: {
        'Content-Type': 'application/json',
      },
    );

    dio = Dio(options);

    // Add logging interceptor
    dio.interceptors.add(
      PrettyDioLogger(
        requestHeader: true,
        requestBody: true,
        responseBody: true,
        responseHeader: false,
        error: true,
        compact: true,
      ),
    );

    // Add auth interceptor
    dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) async {
          final token = await _storage.getToken();
          if (token != null) {
            options.headers['Authorization'] = 'Bearer $token';
          }
          return handler.next(options);
        },
        onError: (error, handler) async {
          if (error.response?.statusCode == 401) {
            await _storage.clearAuth();
          }
          return handler.next(error);
        },
      ),
    );
  }

  // Auth endpoints
  Future<Response> login(String email, String password) async {
    return dio.post('/auth/login', data: {
      'email': email,
      'password': password,
    });
  }

  Future<Response> register(String email, String password, String? companyName, String? phone, String? businessType) async {
    return dio.post('/auth/register', data: {
      'email': email,
      'password': password,
      if (companyName != null) 'company_name': companyName,
      if (phone != null) 'phone': phone,
      if (businessType != null) 'business_type': businessType,
    });
  }

  Future<Response> forgotPassword(String email) async {
    return dio.post('/auth/forgot-password', data: {'email': email});
  }

  Future<Response> resetPassword(String token, String newPassword) async {
    return dio.post('/auth/reset-password', data: {
      'token': token,
      'new_password': newPassword,
    });
  }

  Future<Response> verifyEmail(String token) async {
    return dio.post('/auth/verify-email', data: {'token': token});
  }

  Future<Response> getCurrentUser() async {
    return dio.get('/auth/me');
  }

  Future<Response> updateProfile(Map<String, dynamic> data) async {
    return dio.put('/auth/profile', data: data);
  }

  // Invoice endpoints
  Future<Response> getInvoices({String? status, String? search}) async {
    final params = <String, dynamic>{};
    if (status != null) params['status'] = status;
    if (search != null) params['search'] = search;
    return dio.get('/invoices', queryParameters: params);
  }

  Future<Response> getInvoice(String id) async {
    return dio.get('/invoices/$id');
  }

  Future<Response> createInvoice(Map<String, dynamic> data) async {
    return dio.post('/invoices', data: data);
  }

  Future<Response> updateInvoice(String id, Map<String, dynamic> data) async {
    return dio.put('/invoices/$id', data: data);
  }

  Future<Response> deleteInvoice(String id) async {
    return dio.delete('/invoices/$id');
  }

  Future<Response> sendInvoice(String id, Map<String, dynamic> data) async {
    return dio.post('/invoices/$id/send', data: data);
  }

  Future<Response> getInvoicePdf(String id) async {
    return dio.get('/invoices/$id/pdf', options: Options(
      responseType: ResponseType.bytes,
    ));
  }

  Future<Response> sendReminder(String id, Map<String, dynamic> data) async {
    return dio.post('/invoices/$id/reminder', data: data);
  }

  Future<Response> recordPayment(String id, Map<String, dynamic> data) async {
    return dio.post('/invoices/$id/payments', data: data);
  }

  // Client endpoints
  Future<Response> getClients({String? search}) async {
    final params = <String, dynamic>{};
    if (search != null) params['search'] = search;
    return dio.get('/clients', queryParameters: params);
  }

  Future<Response> getClient(String id) async {
    return dio.get('/clients/$id');
  }

  Future<Response> getClientInvoices(String id) async {
    return dio.get('/clients/$id/invoices');
  }

  Future<Response> getClientStats(String id) async {
    return dio.get('/clients/$id/stats');
  }

  Future<Response> createClient(Map<String, dynamic> data) async {
    return dio.post('/clients', data: data);
  }

  Future<Response> updateClient(String id, Map<String, dynamic> data) async {
    return dio.put('/clients/$id', data: data);
  }

  Future<Response> deleteClient(String id) async {
    return dio.delete('/clients/$id');
  }

  // Payment endpoints
  Future<Response> getPayments({String? status, String? paymentMethod}) async {
    final params = <String, dynamic>{};
    if (status != null) params['status'] = status;
    if (paymentMethod != null) params['payment_method'] = paymentMethod;
    return dio.get('/payments', queryParameters: params);
  }

  Future<Response> getPayment(String id) async {
    return dio.get('/payments/$id');
  }

  Future<Response> createPayment(Map<String, dynamic> data) async {
    return dio.post('/payments', data: data);
  }

  Future<Response> refundPayment(String id, Map<String, dynamic> data) async {
    return dio.post('/payments/$id/refund', data: data);
  }

  Future<Response> getPaymentStats() async {
    return dio.get('/payments/stats');
  }

  Future<Response> getPaymentMethods() async {
    return dio.get('/payments/methods');
  }

  // Expense endpoints
  Future<Response> getExpenses({String? category, String? search}) async {
    final params = <String, dynamic>{};
    if (category != null) params['category'] = category;
    if (search != null) params['search'] = search;
    return dio.get('/expenses', queryParameters: params);
  }

  Future<Response> getExpense(String id) async {
    return dio.get('/expenses/$id');
  }

  Future<Response> createExpense(Map<String, dynamic> data) async {
    return dio.post('/expenses', data: data);
  }

  Future<Response> updateExpense(String id, Map<String, dynamic> data) async {
    return dio.put('/expenses/$id', data: data);
  }

  Future<Response> deleteExpense(String id) async {
    return dio.delete('/expenses/$id');
  }

  Future<Response> getExpenseStats() async {
    return dio.get('/expenses/stats');
  }

  // Dashboard endpoints
  Future<Response> getDashboardOverview() async {
    return dio.get('/dashboard/overview');
  }

  Future<Response> getRecentInvoices() async {
    return dio.get('/dashboard/recent-invoices');
  }

  Future<Response> getRecentPayments() async {
    return dio.get('/dashboard/recent-payments');
  }

  // Report endpoints
  Future<Response> getOverviewStats() async {
    return dio.get('/reports/overview');
  }

  Future<Response> getIncomeReport({String? startDate, String? endDate}) async {
    final params = <String, dynamic>{};
    if (startDate != null) params['start_date'] = startDate;
    if (endDate != null) params['end_date'] = endDate;
    return dio.get('/reports/income', queryParameters: params);
  }

  Future<Response> getExpenseReport({String? startDate, String? endDate}) async {
    final params = <String, dynamic>{};
    if (startDate != null) params['start_date'] = startDate;
    if (endDate != null) params['end_date'] = endDate;
    return dio.get('/reports/expenses', queryParameters: params);
  }

  Future<Response> getTaxReport({String? startDate, String? endDate}) async {
    final params = <String, dynamic>{};
    if (startDate != null) params['start_date'] = startDate;
    if (endDate != null) params['end_date'] = endDate;
    return dio.get('/reports/tax', queryParameters: params);
  }

  Future<Response> getAgingReport() async {
    return dio.get('/reports/aging');
  }

  Future<Response> exportReport(String format, String type) async {
    return dio.get('/reports/export/$type', queryParameters: {'format': format});
  }

  // Settings endpoints
  Future<Response> getBusinessSettings() async {
    return dio.get('/settings/business');
  }

  Future<Response> updateBusinessSettings(Map<String, dynamic> data) async {
    return dio.put('/settings/business', data: data);
  }

  Future<Response> getTaxSettings() async {
    return dio.get('/settings/tax');
  }

  Future<Response> updateTaxSettings(Map<String, dynamic> data) async {
    return dio.put('/settings/tax', data: data);
  }

  Future<Response> getNotificationSettings() async {
    return dio.get('/settings/notifications');
  }

  Future<Response> updateNotificationSettings(Map<String, dynamic> data) async {
    return dio.put('/settings/notifications', data: data);
  }

  Future<Response> getInvoiceSettings() async {
    return dio.get('/settings/invoice');
  }

  Future<Response> updateInvoiceSettings(Map<String, dynamic> data) async {
    return dio.put('/settings/invoice', data: data);
  }
}
