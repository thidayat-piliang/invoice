import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:firebase_core/firebase_core.dart';
import 'app/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize Firebase (if configured)
  // await Firebase.initializeApp();

  runApp(
    const ProviderScope(
      child: FlashBillApp(),
    ),
  );
}
