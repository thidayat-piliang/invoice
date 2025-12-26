import 'package:hive/hive.dart';
import 'package:hive_flutter/hive_flutter.dart';

class LocalStorage {
  static const String _authBox = 'auth_box';
  static const String _tokenKey = 'auth_token';
  static const String _userIdKey = 'user_id';
  static const String _emailKey = 'user_email';
  static const String _settingsBox = 'settings_box';

  static LocalStorage? _instance;
  factory LocalStorage() => _instance ??= LocalStorage._internal();

  LocalStorage._internal();

  Future<void> init() async {
    await Hive.initFlutter();
    await Hive.openBox(_authBox);
    await Hive.openBox(_settingsBox);
  }

  // Auth methods
  Future<void> saveToken(String token) async {
    final box = Hive.box(_authBox);
    await box.put(_tokenKey, token);
  }

  Future<String?> getToken() async {
    final box = Hive.box(_authBox);
    return box.get(_tokenKey);
  }

  Future<void> saveUserData(String userId, String email) async {
    final box = Hive.box(_authBox);
    await box.put(_userIdKey, userId);
    await box.put(_emailKey, email);
  }

  Future<String?> getUserId() async {
    final box = Hive.box(_authBox);
    return box.get(_userIdKey);
  }

  Future<String?> getUserEmail() async {
    final box = Hive.box(_authBox);
    return box.get(_emailKey);
  }

  Future<void> clearAuth() async {
    final box = Hive.box(_authBox);
    await box.clear();
  }

  Future<bool> hasToken() async {
    final token = await getToken();
    return token != null;
  }

  // Settings methods
  Future<void> saveSetting(String key, dynamic value) async {
    final box = Hive.box(_settingsBox);
    await box.put(key, value);
  }

  Future<dynamic> getSetting(String key, {dynamic defaultValue}) async {
    final box = Hive.box(_settingsBox);
    return box.get(key, defaultValue: defaultValue);
  }

  Future<void> clearAll() async {
    await Hive.box(_authBox).clear();
    await Hive.box(_settingsBox).clear();
  }
}
