import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'app/app.dart';
import 'shared/services/local_storage.dart';
import 'shared/services/firebase_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize local storage
  await LocalStorage().init();

  // Initialize Firebase services (if configured)
  await FirebaseService().initialize();

  runApp(
    const ProviderScope(
      child: FlashBillApp(),
    ),
  );
}
