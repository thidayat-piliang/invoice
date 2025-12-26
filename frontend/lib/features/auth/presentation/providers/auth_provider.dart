import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';
import '../../../../shared/services/local_storage.dart';

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
  final LocalStorage _storage;

  AuthNotifier(this._apiClient, this._storage) : super(AuthState());

  // Initialize auth state from local storage
  Future<void> initialize() async {
    final token = await _storage.getToken();
    final userId = await _storage.getUserId();
    final email = await _storage.getUserEmail();

    if (token != null && userId != null && email != null) {
      state = state.copyWith(
        token: token,
        userId: userId,
        email: email,
      );
    }
  }

  Future<bool> login(String email, String password) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.login(email, password);
      final data = response.data;

      final token = data['access_token'];
      final userId = data['user']['id'];
      final userEmail = data['user']['email'];

      // Save to local storage
      await _storage.saveToken(token);
      await _storage.saveUserData(userId, userEmail);

      state = state.copyWith(
        isLoading: false,
        token: token,
        userId: userId,
        email: userEmail,
      );

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

  Future<bool> register(String email, String password, String? companyName, String? phone, String? businessType) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.register(email, password, companyName, phone, businessType);
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

  Future<bool> resetPassword(String token, String newPassword) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.resetPassword(token, newPassword);
      state = state.copyWith(isLoading: false);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to reset password',
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

  Future<bool> verifyEmail(String token) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.verifyEmail(token);
      state = state.copyWith(isLoading: false);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to verify email',
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

  Future<void> logout() async {
    await _storage.clearAuth();
    state = AuthState();
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier(ApiClient(), LocalStorage());
});
