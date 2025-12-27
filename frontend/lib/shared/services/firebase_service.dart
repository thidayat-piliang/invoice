import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_analytics/firebase_analytics.dart';
import 'package:firebase_crashlytics/firebase_crashlytics.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart';

/// Firebase Service for Analytics, Crashlytics, and Messaging
class FirebaseService {
  static FirebaseService? _instance;
  static bool _initialized = false;

  FirebaseAnalytics? _analytics;
  FirebaseCrashlytics? _crashlytics;
  FirebaseMessaging? _messaging;

  FirebaseService._internal();

  factory FirebaseService() {
    _instance ??= FirebaseService._internal();
    return _instance!;
  }

  FirebaseAnalytics? get analytics => _analytics;
  FirebaseCrashlytics? get crashlytics => _crashlytics;
  FirebaseMessaging? get messaging => _messaging;

  /// Initialize Firebase services
  Future<void> initialize() async {
    if (_initialized) return;

    try {
      // Check if Firebase is already initialized
      if (Firebase.apps.isEmpty) {
        // Firebase is not configured - this is expected for development
        // We'll gracefully handle this without crashing
        if (kDebugMode) {
          print('⚠️ Firebase not configured. Running in development mode.');
        }
        _initialized = true;
        return;
      }

      await Firebase.initializeApp();

      _analytics = FirebaseAnalytics.instance;
      _crashlytics = FirebaseCrashlytics.instance;
      _messaging = FirebaseMessaging.instance;

      // Configure Crashlytics
      FlutterError.onError = (errorDetails) {
        if (_crashlytics != null) {
          _crashlytics!.recordFlutterFatalError(errorDetails);
        }
      };

      // Catch async errors
      PlatformDispatcher.instance.onError = (error, stack) {
        if (_crashlytics != null) {
          _crashlytics!.recordError(error, stack, fatal: true);
        }
        return true;
      };

      // Request notification permissions
      if (_messaging != null) {
        await _messaging!.requestPermission(
          alert: true,
          badge: true,
          sound: true,
        );
      }

      _initialized = true;

      if (kDebugMode) {
        print('✅ Firebase initialized successfully');
      }
    } catch (e) {
      // Firebase not configured - continue without it
      if (kDebugMode) {
        print('⚠️ Firebase initialization failed: $e');
        print('   Running without Firebase services');
      }
      _initialized = true;
    }
  }

  /// Log custom analytics event
  Future<void> logEvent(String name, Map<String, dynamic>? parameters) async {
    if (_analytics == null) return;

    try {
      await _analytics!.logEvent(
        name: name,
        parameters: parameters,
      );
    } catch (e) {
      if (kDebugMode) {
        print('Analytics log failed: $e');
      }
    }
  }

  /// Set user properties
  Future<void> setUserProperties({
    String? userId,
    String? email,
    String? name,
  }) async {
    if (_analytics == null) return;

    try {
      if (userId != null) {
        await _analytics!.setUserId(id: userId);
      }
      if (email != null) {
        await _analytics!.setUserProperty(name: 'email', value: email);
      }
      if (name != null) {
        await _analytics!.setUserProperty(name: 'name', value: name);
      }
    } catch (e) {
      if (kDebugMode) {
        print('Set user properties failed: $e');
      }
    }
  }

  /// Log screen view
  Future<void> logScreenView(String screenName, {String? screenClass}) async {
    if (_analytics == null) return;

    try {
      await _analytics!.logScreenView(
        screenName: screenName,
        screenClass: screenClass ?? screenName,
      );
    } catch (e) {
      if (kDebugMode) {
        print('Screen view log failed: $e');
      }
    }
  }

  /// Record custom error
  Future<void> recordError(dynamic error, StackTrace stackTrace, {bool fatal = false}) async {
    if (_crashlytics == null) return;

    try {
      await _crashlytics!.recordError(
        error,
        stackTrace,
        fatal: fatal,
      );
    } catch (e) {
      if (kDebugMode) {
        print('Error recording failed: $e');
      }
    }
  }

  /// Log custom message to Crashlytics
  Future<void> logMessage(String message) async {
    if (_crashlytics == null) return;

    try {
      await _crashlytics!.log(message);
    } catch (e) {
      if (kDebugMode) {
        print('Crashlytics log failed: $e');
      }
    }
  }

  /// Get FCM token
  Future<String?> getFCMToken() async {
    if (_messaging == null) return null;

    try {
      final token = await _messaging!.getToken();
      return token;
    } catch (e) {
      if (kDebugMode) {
        print('FCM token retrieval failed: $e');
      }
      return null;
    }
  }

  /// Handle foreground messages
  void setupForegroundMessageHandler(
    void Function(RemoteMessage) handler,
  ) {
    if (_messaging == null) return;

    FirebaseMessaging.onMessage.listen(handler);
  }

  /// Handle background messages
  Future<void> setupBackgroundMessageHandler(
    Future<void> Function(RemoteMessage) handler,
  ) async {
    if (_messaging == null) return;

    FirebaseMessaging.onBackgroundMessage(handler);
  }

  /// Handle message opened from terminated state
  Future<RemoteMessage?> getInitialMessage() async {
    if (_messaging == null) return null;

    try {
      return await _messaging!.getInitialMessage();
    } catch (e) {
      if (kDebugMode) {
        print('Get initial message failed: $e');
      }
      return null;
    }
  }
}

/// Provider for FirebaseService
final firebaseServiceProvider = Provider<FirebaseService>((ref) {
  return FirebaseService();
});

/// Provider for Firebase Analytics
final firebaseAnalyticsProvider = Provider<FirebaseAnalytics?>((ref) {
  return ref.watch(firebaseServiceProvider).analytics;
});

/// Provider for Firebase Crashlytics
final firebaseCrashlyticsProvider = Provider<FirebaseCrashlytics?>((ref) {
  return ref.watch(firebaseServiceProvider).crashlytics;
});

/// Provider for Firebase Messaging
final firebaseMessagingProvider = Provider<FirebaseMessaging?>((ref) {
  return ref.watch(firebaseServiceProvider).messaging;
});
