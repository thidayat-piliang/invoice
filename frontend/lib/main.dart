import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:firebase_core/firebase_core.dart';
import 'app/app.dart';
import 'shared/services/local_storage.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize local storage
  await LocalStorage().init();

  // Initialize Firebase (if configured)
  // await Firebase.initializeApp();

  runApp(
    const ProviderScope(
      child: FlashBillApp(),
    ),
  );
}
