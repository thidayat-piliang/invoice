# FlashBill Flutter Frontend

A modern, production-ready mobile invoice management application built with Flutter and Riverpod.

## Features

### Core Features
- ✅ **Authentication**: Login, Register, Forgot Password with JWT tokens
- ✅ **Dashboard**: Overview with revenue, expenses, and outstanding invoices
- ✅ **Invoice Management**: Create, view, send, and track invoices
- ✅ **Client Management**: Manage client information and history
- ✅ **Payment Processing**: Record payments with PayPal integration
- ✅ **Expense Tracking**: Track expenses with receipt upload capability
- ✅ **Reports**: Income, expense, tax, and aging reports
- ✅ **Settings**: Business, tax, notification, and invoice settings

### Advanced Features
- ✅ **PDF Generation**: Generate and share professional invoices as PDF
- ✅ **Receipt Scanning**: Image picker with cropping for expense receipts
- ✅ **PayPal Integration**: Complete payment gateway with checkout and refund
- ✅ **Offline Support**: Local storage with Hive for offline functionality
- ✅ **Error Handling**: Comprehensive retry logic and user-friendly error messages
- ✅ **Localization**: Support for English and Indonesian
- ✅ **Firebase Integration**: Analytics, Crashlytics, and Cloud Messaging
- ✅ **Network Monitoring**: Connectivity checks with automatic retry

## Tech Stack

### Frontend
- **Flutter 3.16+**: Modern Flutter framework
- **Riverpod 2.0**: State management
- **Go Router 13.0**: Navigation with deep linking
- **Dio 5.4**: HTTP client with interceptors
- **Hive 2.2**: Local storage
- **Image Picker/Cropper**: Receipt capture and editing
- **PDF/Printing**: PDF generation and sharing
- **Firebase**: Analytics, Crashlytics, Messaging

### Backend Integration
- **Rust API**: RESTful API with Axum framework
- **PostgreSQL**: Database
- **Redis**: Caching and queue management
- **JWT**: Authentication

## Architecture

### Clean Architecture Pattern
```
lib/
├── app/
│   ├── app.dart              # Main application
│   ├── router/               # Navigation routes
│   └── theme/                # App theming
├── features/
│   ├── auth/                 # Authentication feature
│   ├── dashboard/            # Dashboard feature
│   ├── invoices/             # Invoice management
│   ├── clients/              # Client management
│   ├── payments/             # Payment processing
│   ├── expenses/             # Expense tracking
│   ├── reports/              # Reports generation
│   └── settings/             # Settings management
├── shared/
│   ├── services/             # Shared services (API, Firebase, etc.)
│   ├── widgets/              # Reusable UI components
│   └── utils/                # Utility functions
└── l10n/                     # Localization files
```

### State Management
- **Provider-based**: Riverpod for dependency injection
- **State Notifiers**: For complex state management
- **Async State**: Handling loading, error, and data states

## Getting Started

### Prerequisites
- Flutter 3.16 or higher
- Dart 3.2 or higher
- Android Studio / VS Code
- Xcode (for iOS development)
- Flutter SDK installed

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/your-org/flashbill.git
cd flashbill/frontend
```

2. **Install dependencies**
```bash
flutter pub get
```

3. **Configure Firebase (Optional)**
```bash
flutterfire configure --project=your-project-id
```

4. **Run the app**
```bash
# Development
flutter run

# With specific device
flutter run -d chrome  # Web
flutter run -d macos   # macOS
flutter run -d iphone   # iOS simulator
```

### Configuration

#### Environment Variables
Create a `.env` file in the project root:
```env
API_BASE_URL=http://localhost:3000/api/v1
```

Or use build arguments:
```bash
flutter run --dart-define=API_BASE_URL=http://localhost:3000/api/v1
```

#### Firebase Setup (Optional)
1. Go to Firebase Console
2. Create a new project
3. Add Android/iOS apps
4. Download configuration files:
   - `google-services.json` (Android) → `android/app/`
   - `GoogleService-Info.plist` (iOS) → `ios/Runner/`
5. Run `flutterfire configure` to generate `firebase_options.dart`

## Project Structure

### Features
Each feature follows this structure:
```
features/
└── feature_name/
    ├── presentation/
    │   ├── screens/          # Full screens
    │   ├── widgets/          # Feature-specific widgets
    │   └── providers/        # State management
    ├── domain/
    │   ├── entities/         # Data models
    │   └── repositories/     # Repository interfaces
    └── data/
        ├── repositories/     # Repository implementations
        └── datasources/      # Data sources (API, local)
```

### Shared Components
```
shared/
├── services/
│   ├── api_client.dart       # HTTP client with retry
│   ├── firebase_service.dart # Firebase wrapper
│   ├── image_picker_service.dart # Image handling
│   ├── local_storage.dart    # Hive wrapper
│   └── pdf_service.dart      # PDF generation
├── widgets/
│   ├── empty_state.dart      # Empty state widget
│   ├── search_bar.dart       # Search component
│   └── filter_chip.dart      # Filter chip
└── utils/
    ├── constants.dart        # App constants
    ├── validators.dart       # Form validators
    └── helpers.dart          # Utility functions
```

## Key Features in Detail

### 1. Receipt Upload
- **Image Picker**: Gallery or camera selection
- **Image Cropper**: Crop receipts before upload
- **Validation**: File size (max 10MB), format (JPG, PNG, HEIC)
- **Preview**: Real-time image preview
- **Upload**: Multipart form data with progress

### 2. PDF Generation
- **Invoice PDF**: Professional invoice templates
- **Sharing**: Share via device sharing dialog
- **Printing**: Direct print support
- **Downloading**: Save to device storage

### 3. PayPal Integration
- **Checkout**: Create PayPal orders
- **Refund**: Process refunds
- **Status**: Check payment status
- **Error Handling**: Graceful failure handling

### 4. Error Handling & Retry
- **Network Errors**: Automatic retry with exponential backoff
- **User-Friendly Messages**: Clear error messages
- **Connectivity Checks**: Network status monitoring
- **Timeout Handling**: Configurable timeouts

### 5. Localization
- **English & Indonesian**: Full translation support
- **Dynamic Loading**: Locale-based string resolution
- **Fallback**: Default to English

### 6. Firebase Integration
- **Analytics**: Screen views, custom events
- **Crashlytics**: Error reporting, crash logs
- **Messaging**: Push notifications (optional)

## Testing

### Run Tests
```bash
# All tests
flutter test

# Specific file
flutter test test/expense_model_test.dart

# With coverage
flutter test --coverage
```

### Generate Coverage Report
```bash
# Generate HTML report
genhtml coverage/lcov.info -o coverage/html

# Open report
open coverage/html/index.html
```

### Test Structure
- **Unit Tests**: Model and service logic
- **Widget Tests**: UI components
- **Integration Tests**: End-to-end flows

## Development

### Code Generation
```bash
# Generate Riverpod providers
flutter pub run build_runner build

# Watch for changes
flutter pub run build_runner watch
```

### Linting & Formatting
```bash
# Format code
flutter format .

# Analyze code
flutter analyze

# Lint check
flutter pub run flutter_lints:check
```

### Build Commands
```bash
# Debug build
flutter build apk --debug

# Release build
flutter build apk --release

# iOS build
flutter build ipa --release

# Web build
flutter build web --release
```

## API Integration

### Base URL Configuration
```dart
// Development
const API_BASE_URL = 'http://localhost:3000/api/v1';

// Production
const API_BASE_URL = 'https://api.flashbill.app/api/v1';
```

### Authentication Flow
1. User logs in → JWT token received
2. Token stored in Hive
3. Token added to all requests via interceptor
4. 401 response → Clear auth, redirect to login

### Available Endpoints
- `POST /auth/login` - User login
- `POST /auth/register` - User registration
- `GET /invoices` - List invoices
- `POST /invoices` - Create invoice
- `POST /expenses/:id/receipt` - Upload receipt
- `POST /paypal/create-order` - Create PayPal order
- `GET /metrics/health` - Health check

## Performance Optimization

### Image Handling
- **Compression**: Images compressed before upload
- **Caching**: Network images cached
- **Lazy Loading**: List pagination

### State Management
- **Const Constructors**: Where possible
- **Memoization**: Expensive calculations
- **Build Methods**: Optimized rebuilds

### Network
- **Request Deduplication**: Prevent duplicate requests
- **Response Caching**: Cache frequent requests
- **Background Sync**: Offline-first approach

## Security

### Data Protection
- **Token Storage**: Secure Hive storage
- **HTTPS Only**: All API calls over HTTPS
- **Input Validation**: Client and server validation
- **Error Sanitization**: No sensitive data in logs

### Best Practices
- No hardcoded secrets
- Secure token refresh
- Rate limiting awareness
- Input sanitization

## Deployment

### Pre-deployment Checklist
- [ ] Update version in `pubspec.yaml`
- [ ] Update app icons
- [ ] Configure production API URL
- [ ] Set up Firebase for production
- [ ] Test on physical devices
- [ ] Run full test suite
- [ ] Update documentation

### App Store Submission
1. **iOS**: Configure App Store Connect, privacy policy
2. **Android**: Configure Google Play Console, content rating

## Troubleshooting

### Common Issues

**Build Fails**
```bash
flutter clean
flutter pub get
flutter pub run build_runner build
```

**Firebase Errors**
- Ensure configuration files are in correct locations
- Run `flutterfire configure`
- Check Firebase project settings

**API Connection Issues**
- Verify API is running
- Check CORS settings
- Validate API_BASE_URL

**Image Picker Not Working**
- Add permissions to AndroidManifest.xml
- Add permissions to Info.plist (iOS)
- Check device storage

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit changes
4. Push to branch
5. Create Pull Request

### Code Style
- Follow Flutter style guide
- Use meaningful commit messages
- Add tests for new features
- Update documentation

## License

This project is proprietary software. All rights reserved.

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/flashbill/issues
- Documentation: https://docs.flashbill.app

---

**Built with ❤️ using Flutter & Riverpod**
