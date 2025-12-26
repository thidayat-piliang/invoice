import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Models
class Client {
  final String id;
  final String name;
  final String? email;
  final String? phone;
  final String? companyName;
  final double totalInvoiced;
  final double totalPaid;
  final double outstandingBalance;
  final DateTime createdAt;

  Client({
    required this.id,
    required this.name,
    this.email,
    this.phone,
    this.companyName,
    this.totalInvoiced = 0.0,
    this.totalPaid = 0.0,
    this.outstandingBalance = 0.0,
    required this.createdAt,
  });

  factory Client.fromJson(Map<String, dynamic> json) {
    return Client(
      id: json['id'],
      name: json['name'],
      email: json['email'],
      phone: json['phone'],
      companyName: json['company_name'],
      totalInvoiced: json['total_invoiced']?.toDouble() ?? 0.0,
      totalPaid: json['total_paid']?.toDouble() ?? 0.0,
      outstandingBalance: json['outstanding_balance']?.toDouble() ?? 0.0,
      createdAt: DateTime.parse(json['created_at']),
    );
  }
}

class ClientDetail extends Client {
  final List<Map<String, dynamic>> recentInvoices;

  ClientDetail({
    required super.id,
    required super.name,
    super.email,
    super.phone,
    super.companyName,
    super.totalInvoiced,
    super.totalPaid,
    super.outstandingBalance,
    required super.createdAt,
    required this.recentInvoices,
  });

  factory ClientDetail.fromJson(Map<String, dynamic> json) {
    return ClientDetail(
      id: json['id'],
      name: json['name'],
      email: json['email'],
      phone: json['phone'],
      companyName: json['company_name'],
      totalInvoiced: json['total_invoiced']?.toDouble() ?? 0.0,
      totalPaid: json['total_paid']?.toDouble() ?? 0.0,
      outstandingBalance: json['outstanding_balance']?.toDouble() ?? 0.0,
      createdAt: DateTime.parse(json['created_at']),
      recentInvoices: List<Map<String, dynamic>>.from(json['recent_invoices'] ?? []),
    );
  }
}

// State
class ClientState {
  final bool isLoading;
  final String? error;
  final List<Client> clients;
  final ClientDetail? clientDetail;

  ClientState({
    this.isLoading = false,
    this.error,
    this.clients = const [],
    this.clientDetail,
  });

  ClientState copyWith({
    bool? isLoading,
    String? error,
    List<Client>? clients,
    ClientDetail? clientDetail,
  }) {
    return ClientState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      clients: clients ?? this.clients,
      clientDetail: clientDetail ?? this.clientDetail,
    );
  }
}

// Notifier
class ClientNotifier extends StateNotifier<ClientState> {
  final ApiClient _apiClient;

  ClientNotifier(this._apiClient) : super(ClientState());

  Future<void> loadClients({String? search}) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final response = await _apiClient.getClients(search: search);
      final List<Client> clients = (response.data as List)
          .map((e) => Client.fromJson(e))
          .toList();

      state = state.copyWith(isLoading: false, clients: clients);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load clients',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  Future<bool> createClient(Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.createClient(data);
      state = state.copyWith(isLoading: false);
      await loadClients();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to create client',
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

  Future<bool> updateClient(String id, Map<String, dynamic> data) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateClient(id, data);
      state = state.copyWith(isLoading: false);
      await loadClients();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update client',
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

  Future<bool> deleteClient(String id) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.deleteClient(id);
      state = state.copyWith(isLoading: false);
      await loadClients();
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to delete client',
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

  Future<void> loadClientDetail(String id) async {
    state = state.copyWith(isLoading: true, error: null, clientDetail: null);

    try {
      final response = await _apiClient.getClient(id);
      final clientDetail = ClientDetail.fromJson(response.data);

      state = state.copyWith(isLoading: false, clientDetail: clientDetail);
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load client',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: 'An unexpected error occurred',
      );
    }
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final clientProvider = StateNotifierProvider<ClientNotifier, ClientState>((ref) {
  return ClientNotifier(ApiClient());
});
