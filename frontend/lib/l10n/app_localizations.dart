import 'dart:async';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_id.dart';

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of your dependencies
/// ```
abstract class AppLocalizations {
  AppLocalizations(String locale) : _localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String _localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate = _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates = <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('id')
  ];

  /// The title of the app
  ///
  /// In en, this message translates to:
  /// **'FlashBill'**
  String get appTitle;

  /// The subtitle of the app
  ///
  /// In en, this message translates to:
  /// **'Invoice Management System'**
  String get appSubtitle;

  /// Login button text
  ///
  /// In en, this message translates to:
  /// **'Login'**
  String get login;

  /// Register button text
  ///
  /// In en, this message translates to:
  /// **'Register'**
  String get register;

  /// Logout button text
  ///
  /// In en, this message translates to:
  /// **'Logout'**
  String get logout;

  /// Email label
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get email;

  /// Password label
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get password;

  /// Confirm password label
  ///
  /// In en, this message translates to:
  /// **'Confirm Password'**
  String get confirmPassword;

  /// Forgot password text
  ///
  /// In en, this message translates to:
  /// **'Forgot Password?'**
  String get forgotPassword;

  /// No account text
  ///
  /// In en, this message translates to:
  /// **'Don't have an account?'**
  String get noAccount;

  /// Have account text
  ///
  /// In en, this message translates to:
  /// **'Already have an account?'**
  String get haveAccount;

  /// Sign in button
  ///
  /// In en, this message translates to:
  /// **'Sign In'**
  String get signIn;

  /// Sign up button
  ///
  /// In en, this message translates to:
  /// **'Sign Up'**
  String get signUp;

  /// Company name label
  ///
  /// In en, this message translates to:
  /// **'Company Name'**
  String get companyName;

  /// Phone label
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get phone;

  /// Business type label
  ///
  /// In en, this message translates to:
  /// **'Business Type'**
  String get businessType;

  /// Dashboard title
  ///
  /// In en, this message translates to:
  /// **'Dashboard'**
  String get dashboard;

  /// Overview label
  ///
  /// In en, this message translates to:
  /// **'Overview'**
  String get overview;

  /// Total revenue label
  ///
  /// In en, this message translates to:
  /// **'Total Revenue'**
  String get totalRevenue;

  /// Total expenses label
  ///
  /// In en, this message translates to:
  /// **'Total Expenses'**
  String get totalExpenses;

  /// Outstanding invoices label
  ///
  /// In en, this message translates to:
  /// **'Outstanding Invoices'**
  String get outstandingInvoices;

  /// Recent activity label
  ///
  /// In en, this message translates to:
  /// **'Recent Activity'**
  String get recentActivity;

  /// Invoices title
  ///
  /// In en, this message translates to:
  /// **'Invoices'**
  String get invoices;

  /// Invoice label
  ///
  /// In en, this message translates to:
  /// **'Invoice'**
  String get invoice;

  /// Create invoice button
  ///
  /// In en, this message translates to:
  /// **'Create Invoice'**
  String get createInvoice;

  /// Edit invoice button
  ///
  /// In en, this message translates to:
  /// **'Edit Invoice'**
  String get editInvoice;

  /// Invoice number label
  ///
  /// In en, this message translates to:
  /// **'Invoice #'**
  String get invoiceNumber;

  /// Client label
  ///
  /// In en, this message translates to:
  /// **'Client'**
  String get client;

  /// Amount label
  ///
  /// In en, this message translates to:
  /// **'Amount'**
  String get amount;

  /// Status label
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get status;

  /// Due date label
  ///
  /// In en, this message translates to:
  /// **'Due Date'**
  String get dueDate;

  /// Issue date label
  ///
  /// In en, this message translates to:
  /// **'Issue Date'**
  String get issueDate;

  /// Items label
  ///
  /// In en, this message translates to:
  /// **'Items'**
  String get items;

  /// Subtotal label
  ///
  /// In en, this message translates to:
  /// **'Subtotal'**
  String get subtotal;

  /// Tax label
  ///
  /// In en, this message translates to:
  /// **'Tax'**
  String get tax;

  /// Discount label
  ///
  /// In en, this message translates to:
  /// **'Discount'**
  String get discount;

  /// Total label
  ///
  /// In en, this message translates to:
  /// **'Total'**
  String get total;

  /// Notes label
  ///
  /// In en, this message translates to:
  /// **'Notes'**
  String get notes;

  /// Terms label
  ///
  /// In en, this message translates to:
  /// **'Terms & Conditions'**
  String get terms;

  /// Send invoice button
  ///
  /// In en, this message translates to:
  /// **'Send Invoice'**
  String get sendInvoice;

  /// Download PDF button
  ///
  /// In en, this message translates to:
  /// **'Download PDF'**
  String get downloadPDF;

  /// Preview PDF button
  ///
  /// In en, this message translates to:
  /// **'Preview PDF'**
  String get previewPDF;

  /// Record payment button
  ///
  /// In en, this message translates to:
  /// **'Record Payment'**
  String get recordPayment;

  /// Send reminder button
  ///
  /// In en, this message translates to:
  /// **'Send Reminder'**
  String get sendReminder;

  /// Clients title
  ///
  /// In en, this message translates to:
  /// **'Clients'**
  String get clients;

  /// Client name label
  ///
  /// In en, this message translates to:
  /// **'Client Name'**
  String get clientName;

  /// Create client button
  ///
  /// In en, this message translates to:
  /// **'Create Client'**
  String get createClient;

  /// Edit client button
  ///
  /// In en, this message translates to:
  /// **'Edit Client'**
  String get editClient;

  /// Email address label
  ///
  /// In en, this message translates to:
  /// **'Email Address'**
  String get emailAddress;

  /// Contact phone label
  ///
  /// In en, this message translates to:
  /// **'Contact Phone'**
  String get contactPhone;

  /// Address label
  ///
  /// In en, this message translates to:
  /// **'Address'**
  String get address;

  /// Payments title
  ///
  /// In en, this message translates to:
  /// **'Payments'**
  String get payments;

  /// Payment label
  ///
  /// In en, this message translates to:
  /// **'Payment'**
  String get payment;

  /// Create payment button
  ///
  /// In en, this message translates to:
  /// **'Create Payment'**
  String get createPayment;

  /// Payment method label
  ///
  /// In en, this message translates to:
  /// **'Payment Method'**
  String get paymentMethod;

  /// Transaction ID label
  ///
  /// In en, this message translates to:
  /// **'Transaction ID'**
  String get transactionId;

  /// Refund button
  ///
  /// In en, this message translates to:
  /// **'Refund'**
  String get refund;

  /// PayPal checkout button
  ///
  /// In en, this message translates to:
  /// **'PayPal Checkout'**
  String get paypalCheckout;

  /// PayPal refund button
  ///
  /// In en, this message translates to:
  /// **'PayPal Refund'**
  String get paypalRefund;

  /// Expenses title
  ///
  /// In en, this message translates to:
  /// **'Expenses'**
  String get expenses;

  /// Expense label
  ///
  /// In en, this message translates to:
  /// **'Expense'**
  String get expense;

  /// Create expense button
  ///
  /// In en, this message translates to:
  /// **'Create Expense'**
  String get createExpense;

  /// Edit expense button
  ///
  /// In en, this message translates to:
  /// **'Edit Expense'**
  String get editExpense;

  /// Description label
  ///
  /// In en, this message translates to:
  /// **'Description'**
  String get description;

  /// Category label
  ///
  /// In en, this message translates to:
  /// **'Category'**
  String get category;

  /// Date label
  ///
  /// In en, this message translates to:
  /// **'Date'**
  String get date;

  /// Notes optional label
  ///
  /// In en, this message translates to:
  /// **'Notes (Optional)'**
  String get notesOptional;

  /// Tax deductible label
  ///
  /// In en, this message translates to:
  /// **'Tax Deductible'**
  String get taxDeductible;

  /// Receipt label
  ///
  /// In en, this message translates to:
  /// **'Receipt'**
  String get receipt;

  /// Upload receipt button
  ///
  /// In en, this message translates to:
  /// **'Upload Receipt'**
  String get uploadReceipt;

  /// View receipt button
  ///
  /// In en, this message translates to:
  /// **'View Receipt'**
  String get viewReceipt;

  /// Replace receipt button
  ///
  /// In en, this message translates to:
  /// **'Replace Receipt'**
  String get replaceReceipt;

  /// No receipt label
  ///
  /// In en, this message translates to:
  /// **'No Receipt'**
  String get noReceipt;

  /// Reports title
  ///
  /// In en, this message translates to:
  /// **'Reports'**
  String get reports;

  /// Income report button
  ///
  /// In en, this message translates to:
  /// **'Income Report'**
  String get incomeReport;

  /// Expense report button
  ///
  /// In en, this message translates to:
  /// **'Expense Report'**
  String get expenseReport;

  /// Tax report button
  ///
  /// In en, this message translates to:
  /// **'Tax Report'**
  String get taxReport;

  /// Aging report button
  ///
  /// In en, this message translates to:
  /// **'Aging Report'**
  String get agingReport;

  /// Export button
  ///
  /// In en, this message translates to:
  /// **'Export'**
  String get export;

  /// Start date label
  ///
  /// In en, this message translates to:
  /// **'Start Date'**
  String get startDate;

  /// End date label
  ///
  /// In en, this message translates to:
  /// **'End Date'**
  String get endDate;

  /// Apply button
  ///
  /// In en, this message translates to:
  /// **'Apply'**
  String get apply;

  /// Settings title
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settings;

  /// Business settings button
  ///
  /// In en, this message translates to:
  /// **'Business Settings'**
  String get businessSettings;

  /// Tax settings button
  ///
  /// In en, this message translates to:
  /// **'Tax Settings'**
  String get taxSettings;

  /// Notification settings button
  ///
  /// In en, this message translates to:
  /// **'Notification Settings'**
  String get notificationSettings;

  /// Invoice settings button
  ///
  /// In en, this message translates to:
  /// **'Invoice Settings'**
  String get invoiceSettings;

  /// Company details label
  ///
  /// In en, this message translates to:
  /// **'Company Details'**
  String get companyDetails;

  /// Tax rates label
  ///
  /// In en, this message translates to:
  /// **'Tax Rates'**
  String get taxRates;

  /// Email notifications label
  ///
  /// In en, this message translates to:
  /// **'Email Notifications'**
  String get emailNotifications;

  /// Invoice template label
  ///
  /// In en, this message translates to:
  /// **'Invoice Template'**
  String get invoiceTemplate;

  /// Save button
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// Update button
  ///
  /// In en, this message translates to:
  /// **'Update'**
  String get update;

  /// Cancel button
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// Delete button
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get delete;

  /// Edit button
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get edit;

  /// Close button
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// Back button
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get back;

  /// Retry button
  ///
  /// In en, this message translates to：
  /// **'Retry'**
  String get retry;

  /// Search label
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get search;

  /// Filter button
  ///
  /// In en, this message translates to:
  /// **'Filter'**
  String get filter;

  /// Clear button
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clear;

  /// Apply filters button
  ///
  /// In en, this message translates to:
  /// **'Apply Filters'**
  String get applyFilters;

  /// Add button
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get add;

  /// Submit button
  ///
  /// In en, this message translates to:
  /// **'Submit'**
  String get submit;

  /// Yes button
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get yes;

  /// No button
  ///
  /// In en, this message translates to:
  /// **'No'**
  String get no;

  /// Confirm button
  ///
  /// In en, this message translates to:
  /// **'Confirm'**
  String get confirm;

  /// Are you sure text
  ///
  /// In en, this message translates to:
  /// **'Are you sure?'**
  String get areYouSure;

  /// Success message
  ///
  /// In en, this message translates to:
  /// **'Success'**
  String get success;

  /// Error message
  ///
  /// In en, this message translates to:
  /// **'Error'**
  String get error;

  /// Loading message
  ///
  /// In en, this message translates to:
  /// **'Loading'**
  String get loading;

  /// No data message
  ///
  /// In en, this message translates to:
  /// **'No Data Found'**
  String get noData;

  /// Try adjusting message
  ///
  /// In en, this message translates to:
  /// **'Try adjusting your filters or add new data'**
  String get tryAdjusting;

  /// Something went wrong message
  ///
  /// In en, this message translates to:
  /// **'Something went wrong'**
  String get somethingWentWrong;

  /// Internet connection message
  ///
  /// In en, this message translates to:
  /// **'Internet connection'**
  String get internetConnection;

  /// Server unavailable message
  ///
  /// In en, this message translates to:
  /// **'Server unavailable'**
  String get serverUnavailable;

  /// Validation error message
  ///
  /// In en, this message translates to:
  /// **'Validation error'**
  String get validationError;

  /// Draft status
  ///
  /// In en, this message translates to:
  /// **'Draft'**
  String get draft;

  /// Sent status
  ///
  /// In en, this message translates to:
  /// **'Sent'**
  String get sent;

  /// Viewed status
  ///
  /// In en, this message translates to:
  /// **'Viewed'**
  String get viewed;

  /// Paid status
  ///
  /// In en, this message translates to:
  /// **'Paid'**
  String get paid;

  /// Overdue status
  ///
  /// In en, this message translates to:
  /// **'Overdue'**
  String get overdue;

  /// Partial status
  ///
  /// In en, this message translates to:
  /// **'Partial'**
  String get partial;

  /// Cancelled status
  ///
  /// In en, this message translates to:
  /// **'Cancelled'**
  String get cancelled;

  /// Office category
  ///
  /// In en, this message translates to:
  /// **'Office'**
  String get office;

  /// Travel category
  ///
  /// In en, this message translates to：
  /// **'Travel'**
  String get travel;

  /// Marketing category
  ///
  /// In en, this message translates to:
  /// **'Marketing'**
  String get marketing;

  /// Utilities category
  ///
  /// In en, this message translates to:
  /// **'Utilities'**
  String get utilities;

  /// Other category
  ///
  /// In en, this message translates to:
  /// **'Other'**
  String get other;

  /// Receipt tips title
  ///
  /// In en, this message translates to:
  /// **'Tips for better receipts'**
  String get receiptTipsTitle;

  /// Receipt tip 1
  ///
  /// In en, this message translates to:
  /// **'Ensure good lighting'**
  String get receiptTip1;

  /// Receipt tip 2
  ///
  /// In en, this message translates to:
  /// **'Capture all four corners of the receipt'**
  String get receiptTip2;

  /// Receipt tip 3
  ///
  /// In en, this message translates to:
  /// **'Make sure text is readable'**
  String get receiptTip3;

  /// Receipt tip 4
  ///
  /// In en, this message translates to:
  /// **'Avoid glare and shadows'**
  String get receiptTip4;

  /// Required field validation
  ///
  /// In en, this message translates to:
  /// **'This field is required'**
  String get requiredField;

  /// Invalid email validation
  ///
  /// In en, this message translates to:
  /// **'Invalid email address'**
  String get invalidEmail;

  /// Password mismatch validation
  ///
  /// In en, this message translates to:
  /// **'Passwords do not match'**
  String get passwordMismatch;

  /// Invalid amount validation
  ///
  /// In en, this message translates to:
  /// **'Invalid amount'**
  String get invalidAmount;

  /// File too large validation
  ///
  /// In en, this message translates to:
  /// **'File is too large (max 10MB)'**
  String get fileTooLarge;

  /// Invalid image validation
  ///
  /// In en, this message translates to:
  /// **'Invalid image file'**
  String get invalidImage;

  /// Select image action
  ///
  /// In en, this message translates to:
  /// **'Select Image'**
  String get selectImage;

  /// Take photo action
  ///
  /// In en, this message translates to:
  /// **'Take Photo'**
  String get takePhoto;

  /// Choose from gallery action
  ///
  /// In en, this message translates to:
  /// **'Choose from Gallery'**
  String get chooseFromGallery;

  /// Share action
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get share;

  /// Print action
  ///
  /// In en, this message translates to:
  /// **'Print'**
  String get print;

  /// Download action
  ///
  /// In en, this message translates to:
  /// **'Download'**
  String get download;

  /// Upload action
  ///
  /// In en, this message translates to:
  /// **'Upload'**
  String get upload;
}

class _AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) {
    return <String>['en', 'id'].contains(locale.languageCode);
  }

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en': return AppLocalizationsEn();
    case 'id': return AppLocalizationsId();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.'
  );
}
