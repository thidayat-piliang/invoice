import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Auth State
class AuthState {
  final bool isLoading;
  final String? error;
  final String? token;
  final String? userId;
  final String? email;

  AuthState({
    this.isLoading = false,
    this.error,
    this.token,
    this.userId,
    this.email,
  });

  AuthState copyWith({
    bool? isLoading,
    String? error,
    String? token,
    String? userId,
    String? email,
  }) {
    return AuthState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      token: token ?? this.token,
      userId: userId ?? this.userId,
      email: email ?? this.email,
    );
  }

  bool get isAuthenticated => token != null;
}

// Auth Notifier
class AuthNotifier extends StateNotifier<AuthState> {
  final ApiClient _apiClient;

  AuthNotifier(this._apiClient) : super(AuthState());

  Future<bool> login(String email, String password) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.login(email, password);
      final data = response.data;

      state = state.copyWith(
        isLoading: false,
        token: data['access_token'],
        userId: data['user']['id'],
        email: data['user']['email'],
      );

      // TODO: Save token to local storage
      // await LocalStorage().saveToken(data['access_token']);

      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Login failed',
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

  Future<bool> register(String email, String password, String? companyName) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.register(email, password, companyName);
      state = state.copyWith(isLoading: false);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Registration failed',
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

  Future<bool> forgotPassword(String email) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.forgotPassword(email);
      state = state.copyWith(isLoading: false);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to send reset email',
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

  void logout() {
    state = AuthState();
    // TODO: Clear token from local storage
    // LocalStorage().clearToken();
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier(ApiClient());
});
