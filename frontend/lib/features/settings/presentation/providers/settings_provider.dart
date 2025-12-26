import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import '../../../../shared/services/api_client.dart';

// Models
class BusinessSettings {
  final String? companyName;
  final String? phone;
  final Map<String, dynamic>? address;

  BusinessSettings({
    this.companyName,
    this.phone,
    this.address,
  });

  factory BusinessSettings.fromJson(Map<String, dynamic> json) {
    return BusinessSettings(
      companyName: json['company_name'],
      phone: json['phone'],
      address: json['address'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'company_name': companyName,
      'phone': phone,
      'address': address,
    };
  }
}

class TaxSettings {
  final String state;
  final double rate;
  final bool isExempt;

  TaxSettings({
    required this.state,
    required this.rate,
    required this.isExempt,
  });

  factory TaxSettings.fromJson(Map<String, dynamic> json) {
    return TaxSettings(
      state: json['state'],
      rate: json['rate']?.toDouble() ?? 0.0,
      isExempt: json['is_exempt'] ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'state': state,
      'rate': rate,
      'is_exempt': isExempt,
    };
  }
}

class NotificationSettings {
  final bool emailPaymentReceived;
  final bool emailInvoicePaid;
  final bool emailPaymentReminder;
  final bool pushPaymentReceived;
  final bool pushOverdue;

  NotificationSettings({
    required this.emailPaymentReceived,
    required this.emailInvoicePaid,
    required this.emailPaymentReminder,
    required this.pushPaymentReceived,
    required this.pushOverdue,
  });

  factory NotificationSettings.fromJson(Map<String, dynamic> json) {
    return NotificationSettings(
      emailPaymentReceived: json['email_payment_received'] ?? false,
      emailInvoicePaid: json['email_invoice_paid'] ?? false,
      emailPaymentReminder: json['email_payment_reminder'] ?? false,
      pushPaymentReceived: json['push_payment_received'] ?? false,
      pushOverdue: json['push_overdue'] ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'email_payment_received': emailPaymentReceived,
      'email_invoice_paid': emailInvoicePaid,
      'email_payment_reminder': emailPaymentReminder,
      'push_payment_received': pushPaymentReceived,
      'push_overdue': pushOverdue,
    };
  }
}

class InvoiceSettings {
  final String? currency;
  final String? template;
  final String? terms;
  final String? notes;

  InvoiceSettings({
    this.currency,
    this.template,
    this.terms,
    this.notes,
  });

  factory InvoiceSettings.fromJson(Map<String, dynamic> json) {
    return InvoiceSettings(
      currency: json['currency'],
      template: json['template'],
      terms: json['terms'],
      notes: json['notes'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'currency': currency,
      'template': template,
      'terms': terms,
      'notes': notes,
    };
  }
}

// State
class SettingsState {
  final bool isLoading;
  final String? error;
  final BusinessSettings? business;
  final TaxSettings? tax;
  final NotificationSettings? notifications;
  final InvoiceSettings? invoice;

  SettingsState({
    this.isLoading = false,
    this.error,
    this.business,
    this.tax,
    this.notifications,
    this.invoice,
  });

  SettingsState copyWith({
    bool? isLoading,
    String? error,
    BusinessSettings? business,
    TaxSettings? tax,
    NotificationSettings? notifications,
    InvoiceSettings? invoice,
  }) {
    return SettingsState(
      isLoading: isLoading ?? this.isLoading,
      error: error,
      business: business ?? this.business,
      tax: tax ?? this.tax,
      notifications: notifications ?? this.notifications,
      invoice: invoice ?? this.invoice,
    );
  }
}

// Notifier
class SettingsNotifier extends StateNotifier<SettingsState> {
  final ApiClient _apiClient;

  SettingsNotifier(this._apiClient) : super(SettingsState());

  Future<void> loadAllSettings() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final businessResponse = await _apiClient.getBusinessSettings();
      final taxResponse = await _apiClient.getTaxSettings();
      final notificationsResponse = await _apiClient.getNotificationSettings();
      final invoiceResponse = await _apiClient.getInvoiceSettings();

      state = state.copyWith(
        isLoading: false,
        business: BusinessSettings.fromJson(businessResponse.data),
        tax: TaxSettings.fromJson(taxResponse.data),
        notifications: NotificationSettings.fromJson(notificationsResponse.data),
        invoice: InvoiceSettings.fromJson(invoiceResponse.data),
      );
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to load settings',
      );
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
    }
  }

  Future<bool> updateBusinessSettings(BusinessSettings settings) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateBusinessSettings(settings.toJson());
      state = state.copyWith(isLoading: false, business: settings);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update settings',
      );
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
      return false;
    }
  }

  Future<bool> updateTaxSettings(TaxSettings settings) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateTaxSettings(settings.toJson());
      state = state.copyWith(isLoading: false, tax: settings);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update settings',
      );
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
      return false;
    }
  }

  Future<bool> updateNotificationSettings(NotificationSettings settings) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateNotificationSettings(settings.toJson());
      state = state.copyWith(isLoading: false, notifications: settings);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update settings',
      );
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
      return false;
    }
  }

  Future<bool> updateInvoiceSettings(InvoiceSettings settings) async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      await _apiClient.updateInvoiceSettings(settings.toJson());
      state = state.copyWith(isLoading: false, invoice: settings);
      return true;
    } on DioException catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.response?.data['error']['message'] ?? 'Failed to update settings',
      );
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: 'An unexpected error occurred');
      return false;
    }
  }

  void clearError() {
    state = state.copyWith(error: null);
  }
}

// Provider
final settingsProvider = StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
  return SettingsNotifier(ApiClient());
});
