import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:dio/dio.dart';
import 'package:flashbill/shared/services/api_client.dart';

void main() {
  group('RetryConfig Tests', () {
    test('Default retry config should have correct values', () {
      const config = RetryConfig();

      expect(config.maxRetries, 3);
      expect(config.initialDelay.inSeconds, 1);
      expect(config.backoffMultiplier, 2.0);
      expect(config.retryStatusCodes, [408, 429, 500, 502, 503, 504]);
    });

    test('Custom retry config should use provided values', () {
      const config = RetryConfig(
        maxRetries: 5,
        initialDelay: Duration(seconds: 2),
        backoffMultiplier: 1.5,
        retryStatusCodes: [500],
      );

      expect(config.maxRetries, 5);
      expect(config.initialDelay.inSeconds, 2);
      expect(config.backoffMultiplier, 1.5);
      expect(config.retryStatusCodes, [500]);
    });
  });

  group('ApiClient Error Handling', () {
    late ApiClient apiClient;

    setUp(() {
      apiClient = ApiClient();
    });

    test('Should return correct error message for timeout', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        type: DioExceptionType.connectionTimeout,
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Connection timeout. Please check your internet connection.');
    });

    test('Should return correct error message for 401', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(statusCode: 401, requestOptions: RequestOptions(path: '/test')),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Authentication required. Please login again.');
    });

    test('Should return correct error message for 403', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(statusCode: 403, requestOptions: RequestOptions(path: '/test')),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'You don\'t have permission to perform this action.');
    });

    test('Should return correct error message for 404', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(statusCode: 404, requestOptions: RequestOptions(path: '/test')),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Resource not found.');
    });

    test('Should return correct error message for 422 with custom message', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(
          statusCode: 422,
          requestOptions: RequestOptions(path: '/test'),
          data: {
            'error': {
              'message': 'Invalid email format',
            },
          },
        ),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Invalid email format');
    });

    test('Should return correct error message for 500', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(statusCode: 500, requestOptions: RequestOptions(path: '/test')),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Server error. Please try again later.');
    });

    test('Should return generic message for unknown status code', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(statusCode: 999, requestOptions: RequestOptions(path: '/test')),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'An unexpected error occurred.');
    });

    test('Should handle response without data gracefully', () {
      final error = DioException(
        requestOptions: RequestOptions(path: '/test'),
        response: Response(
          statusCode: 500,
          requestOptions: RequestOptions(path: '/test'),
        ),
      );

      final message = apiClient.getErrorMessage(error);
      expect(message, 'Server error. Please try again later.');
    });
  });
}
