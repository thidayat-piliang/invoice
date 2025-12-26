import 'package:intl/intl.dart';

class Formatters {
  static String formatCurrency(double amount) {
    final formatter = NumberFormat.currency(
      symbol: '\$',
      decimalDigits: 2,
    );
    return formatter.format(amount);
  }

  static String formatDate(DateTime date) {
    final formatter = DateFormat('MM/dd/yyyy');
    return formatter.format(date);
  }

  static String formatDateTime(DateTime dateTime) {
    final formatter = DateFormat('MM/dd/yyyy, h:mm a');
    return formatter.format(dateTime);
  }

  static String formatPhoneNumber(String phone) {
    // Simple US phone formatter
    final cleaned = phone.replaceAll(RegExp(r'[^\d]'), '');
    if (cleaned.length == 10) {
      return '(${cleaned.substring(0, 3)}) ${cleaned.substring(3, 6)}-${cleaned.substring(6)}';
    }
    return phone;
  }

  static String formatInvoiceNumber(int year, int sequence) {
    return 'INV-$year-${sequence.toString().padLeft(4, '0')}';
  }
}
