# FlashBill Frontend

Flutter-based mobile application for FlashBill invoice management.

## Tech Stack

- **Framework:** Flutter 3.16+
- **State Management:** Riverpod 2.0
- **Navigation:** Go Router
- **HTTP Client:** Dio
- **Storage:** Hive/Isar
- **Analytics:** Firebase
- **Push Notifications:** Firebase Cloud Messaging

## Setup

1. Install Flutter: https://flutter.dev/docs/get-started/install
2. Clone repository
3. Run: `flutter pub get`
4. Run: `flutter run`

## Project Structure

```
lib/
├── main.dart
├── app/
│   ├── app.dart
│   ├── router/
│   │   └── app_router.dart
│   ├── theme/
│   │   ├── app_theme.dart
│   │   ├── colors.dart
│   │   └── typography.dart
│   └── localization/
├── features/
│   ├── auth/
│   │   ├── presentation/
│   │   │   ├── screens/
│   │   │   ├── widgets/
│   │   │   └── providers/
│   │   ├── domain/
│   │   └── data/
│   ├── dashboard/
│   ├── invoices/
│   ├── clients/
│   ├── payments/
│   └── settings/
├── shared/
│   ├── widgets/
│   ├── utils/
│   └── services/
└── core/
    ├── di/
    ├── constants/
    └── exceptions/
```

## Features

### Screens
- **Login:** User authentication
- **Register:** New account creation
- **Dashboard:** Overview metrics
- **Invoice List:** View all invoices
- **Create Invoice:** New invoice form
- **Invoice Detail:** View invoice details
- **Client List:** Manage clients
- **Settings:** App configuration

### State Management
```dart
// Example: Invoice Provider
final invoiceProvider = StateNotifierProvider<InvoiceNotifier, InvoiceState>(
  (ref) => InvoiceNotifier(ApiClient()),
);
```

### API Integration
```dart
// Example: API Client
final response = await apiClient.createInvoice(data);
```

## Building

### Android
```bash
flutter build apk --release
```

### iOS
```bash
flutter build ios --release
```

### Web
```bash
flutter build web --release
```

## Testing

```bash
# Run tests
flutter test

# Run with coverage
flutter test --coverage
```

## Code Generation

```bash
# Generate providers
flutter pub run build_runner build

# Watch for changes
flutter pub run build_runner watch
```

## Firebase Setup

1. Install Firebase CLI
2. Run: `firebase login`
3. Run: `flutterfire configure`
4. Add `google-services.json` (Android) and `GoogleService-Info.plist` (iOS)

## CI/CD

See `.github/workflows/frontend-ci.yml`

## Release Checklist

- [ ] Update version in `pubspec.yaml`
- [ ] Update app icon
- [ ] Generate release build
- [ ] Test on physical devices
- [ ] Update screenshots
- [ ] Submit to App Store/Play Store

## License

MIT
