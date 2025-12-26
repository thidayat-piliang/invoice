import 'package:dio/dio.dart';
import 'package:pretty_dio_logger/pretty_dio_logger.dart';

class ApiClient {
  static final ApiClient _instance = ApiClient._internal();
  factory ApiClient() => _instance;

  late Dio dio;

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
          // TODO: Get token from local storage
          // final token = await LocalStorage().getToken();
          // if (token != null) {
          //   options.headers['Authorization'] = 'Bearer $token';
          // }
          return handler.next(options);
        },
        onError: (error, handler) async {
          if (error.response?.statusCode == 401) {
            // TODO: Handle token refresh or logout
            // await LocalStorage().clearToken();
            // context.go('/auth/login');
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

  Future<Response> register(String email, String password, String? companyName) async {
    return dio.post('/auth/register', data: {
      'email': email,
      'password': password,
      'company_name': companyName,
    });
  }

  Future<Response> forgotPassword(String email) async {
    return dio.post('/auth/forgot-password', data: {'email': email});
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

  // Client endpoints
  Future<Response> getClients({String? search}) async {
    final params = <String, dynamic>{};
    if (search != null) params['search'] = search;
    return dio.get('/clients', queryParameters: params);
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

  // Dashboard endpoints
  Future<Response> getDashboardOverview() async {
    return dio.get('/reports/overview');
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
}
