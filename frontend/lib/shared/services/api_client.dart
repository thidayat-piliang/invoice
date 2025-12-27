import 'package:dio/dio.dart';
import 'package:pretty_dio_logger/pretty_dio_logger.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'local_storage.dart';

/// Custom exception for API errors with user-friendly messages
class ApiException implements Exception {
  final String message;
  final int? statusCode;
  final String? errorCode;

  ApiException(this.message, {this.statusCode, this.errorCode});

  @override
  String toString() => 'ApiException: $message (Code: $statusCode)';
}

/// Retry configuration
class RetryConfig {
  final int maxRetries;
  final Duration initialDelay;
  final double backoffMultiplier;
  final List<int> retryStatusCodes;

  const RetryConfig({
    this.maxRetries = 3,
    this.initialDelay = const Duration(seconds: 1),
    this.backoffMultiplier = 2.0,
    this.retryStatusCodes = const [408, 429, 500, 502, 503, 504],
  });
}

/// API Client with retry logic and enhanced error handling
class ApiClient {
  static final ApiClient _instance = ApiClient._internal();
  factory ApiClient() => _instance;

  late Dio dio;
  final LocalStorage _storage = LocalStorage();
  final RetryConfig retryConfig;

  ApiClient._internal({this.retryConfig = const RetryConfig()}) {
    final options = BaseOptions(
      baseUrl: _getBaseUrl(),
      connectTimeout: const Duration(seconds: 30),
      receiveTimeout: const Duration(seconds: 30),
      headers: {
        'Content-Type': 'application/json',
      },
    );

    dio = Dio(options);

    // Add logging interceptor (only in debug mode)
    assert(() {
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
      return true;
    }());

    // Add retry interceptor
    dio.interceptors.add(_createRetryInterceptor());

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

  String _getBaseUrl() {
    // Try to get from environment, fallback to localhost
    const fromEnv = String.fromEnvironment('API_BASE_URL');
    return fromEnv.isNotEmpty ? fromEnv : 'http://localhost:3000/api/v1';
  }

  /// Create retry interceptor with exponential backoff
  InterceptorsWrapper _createRetryInterceptor() {
    return InterceptorsWrapper(
      onError: (error, handler) async {
        // Check if we should retry
        if (!_shouldRetry(error)) {
          return handler.next(error);
        }

        // Check network connectivity
        final connectivity = await Connectivity().checkConnectivity();
        if (connectivity == ConnectivityResult.none) {
          return handler.next(
            DioException(
              requestOptions: error.requestOptions,
              type: DioExceptionType.connectionError,
              message: 'No internet connection',
              error: ApiException('No internet connection'),
            ),
          );
        }

        // Attempt retry
        final retryCount = error.requestOptions.extra['retryCount'] ?? 0;
        if (retryCount >= retryConfig.maxRetries) {
          return handler.next(error);
        }

        // Calculate delay with exponential backoff
        final delay = retryConfig.initialDelay *
            retryConfig.backoffMultiplier.pow(retryCount).toInt();

        // Wait before retry
        await Future.delayed(delay);

        // Update retry count
        error.requestOptions.extra['retryCount'] = retryCount + 1;

        try {
          // Retry the request
          final retryResponse = await dio.fetch(error.requestOptions);
          return handler.resolve(retryResponse);
        } catch (e) {
          return handler.next(error);
        }
      },
    );
  }

  bool _shouldRetry(DioException error) {
    if (error.type == DioExceptionType.connectionTimeout ||
        error.type == DioExceptionType.sendTimeout ||
        error.type == DioExceptionType.receiveTimeout ||
        error.type == DioExceptionType.connectionError) {
      return true;
    }

    if (error.response != null) {
      return retryConfig.retryStatusCodes.contains(error.response!.statusCode);
    }

    return false;
  }

  /// Check network connectivity
  Future<bool> checkConnectivity() async {
    final connectivity = await Connectivity().checkConnectivity();
    return connectivity != ConnectivityResult.none;
  }

  /// Get user-friendly error message
  String getErrorMessage(DioException error) {
    if (error.type == DioExceptionType.connectionTimeout ||
        error.type == DioExceptionType.sendTimeout ||
        error.type == DioExceptionType.receiveTimeout) {
      return 'Connection timeout. Please check your internet connection.';
    }

    if (error.type == DioExceptionType.connectionError) {
      return 'Cannot connect to server. Please try again later.';
    }

    if (error.response == null) {
      return 'An unexpected error occurred. Please try again.';
    }

    final statusCode = error.response!.statusCode;
    final data = error.response!.data;

    // Extract error message from response
    String? errorMessage;
    if (data is Map<String, dynamic>) {
      if (data.containsKey('error')) {
        final errorData = data['error'];
        if (errorData is Map<String, dynamic> && errorData.containsKey('message')) {
          errorMessage = errorData['message'];
        } else if (errorData is String) {
          errorMessage = errorData;
        }
      } else if (data.containsKey('message')) {
        errorMessage = data['message'];
      }
    }

    switch (statusCode) {
      case 400:
        return errorMessage ?? 'Bad request. Please check your input.';
      case 401:
        return 'Authentication required. Please login again.';
      case 403:
        return 'You don\'t have permission to perform this action.';
      case 404:
        return errorMessage ?? 'Resource not found.';
      case 409:
        return errorMessage ?? 'Conflict. The resource already exists.';
      case 422:
        return errorMessage ?? 'Validation error. Please check your input.';
      case 429:
        return 'Too many requests. Please try again later.';
      case 500:
        return 'Server error. Please try again later.';
      case 502:
        return 'Bad gateway. Please try again later.';
      case 503:
        return 'Service unavailable. Please try again later.';
      case 504:
        return 'Gateway timeout. Please try again later.';
      default:
        return errorMessage ?? 'An unexpected error occurred.';
    }
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

  // PayPal endpoints
  Future<Response> createPayPalOrder(Map<String, dynamic> data) async {
    return dio.post('/paypal/create-order', data: data);
  }

  Future<Response> refundPayPalPayment(String orderId, Map<String, dynamic> data) async {
    return dio.post('/paypal/refund/$orderId', data: data);
  }

  Future<Response> getPayPalStatus(String orderId) async {
    return dio.get('/paypal/status/$orderId');
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

  Future<Response> uploadExpenseReceipt(String id, MultipartFile file) async {
    final formData = FormData.fromMap({'receipt': file});
    return dio.post('/expenses/$id/receipt', data: formData);
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

  // Monitoring endpoints
  Future<Response> getHealthStatus() async {
    return dio.get('/metrics/health');
  }

  Future<Response> getReadinessStatus() async {
    return dio.get('/metrics/ready');
  }

  Future<Response> getMonitoringSummary() async {
    return dio.get('/metrics/monitoring/summary');
  }

  Future<Response> getActiveRequests() async {
    return dio.get('/metrics/monitoring/active-requests');
  }

  Future<Response> getRecentErrors() async {
    return dio.get('/metrics/monitoring/errors');
  }
}
