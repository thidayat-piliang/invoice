import 'package:flutter/material.dart';

enum StatusType {
  draft,
  sent,
  paid,
  overdue,
  partial,
  cancelled,
  completed,
  failed,
  refunded,
}

class StatusBadge extends StatelessWidget {
  final StatusType status;
  final String label;
  final bool outlined;

  const StatusBadge({
    super.key,
    required this.status,
    required this.label,
    this.outlined = false,
  });

  @override
  Widget build(BuildContext context) {
    final colors = _getStatusColors();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: outlined ? colors.background : colors.color.withOpacity(0.1),
        border: outlined ? Border.all(color: colors.color) : null,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Text(
        label.toUpperCase(),
        style: TextStyle(
          color: colors.color,
          fontSize: 11,
          fontWeight: FontWeight.w600,
          letterSpacing: 0.5,
        ),
      ),
    );
  }

  _StatusColors _getStatusColors() {
    switch (status) {
      case StatusType.draft:
        return _StatusColors(Colors.grey, Colors.grey[100]!);
      case StatusType.sent:
        return _StatusColors(Colors.blue, Colors.blue[50]!);
      case StatusType.paid:
        return _StatusColors(Colors.green, Colors.green[50]!);
      case StatusType.overdue:
        return _StatusColors(Colors.red, Colors.red[50]!);
      case StatusType.partial:
        return _StatusColors(Colors.orange, Colors.orange[50]!);
      case StatusType.cancelled:
        return _StatusColors(Colors.grey, Colors.grey[200]!);
      case StatusType.completed:
        return _StatusColors(Colors.green, Colors.green[50]!);
      case StatusType.failed:
        return _StatusColors(Colors.red, Colors.red[50]!);
      case StatusType.refunded:
        return _StatusColors(Colors.purple, Colors.purple[50]!);
    }
  }
}

class _StatusColors {
  final Color color;
  final Color background;

  _StatusColors(this.color, this.background);
}
