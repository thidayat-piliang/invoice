import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: ListView(
        children: [
          const SizedBox(height: 16),
          _buildSection(
            'Business Settings',
            [
              _buildTile(
                context,
                Icons.business,
                'Business Information',
                () {},
              ),
              _buildTile(
                context,
                Icons.attach_money,
                'Tax Settings',
                () {},
              ),
              _buildTile(
                context,
                Icons.notifications,
                'Notifications',
                () {},
              ),
            ],
          ),
          const SizedBox(height: 16),
          _buildSection(
            'Invoice Settings',
            [
              _buildTile(
                context,
                Icons.description,
                'Invoice Template',
                () {},
              ),
              _buildTile(
                context,
                Icons.email,
                'Email Settings',
                () {},
              ),
            ],
          ),
          const SizedBox(height: 16),
          _buildSection(
            'Account',
            [
              _buildTile(
                context,
                Icons.lock,
                'Change Password',
                () {},
              ),
              _buildTile(
                context,
                Icons.logout,
                'Logout',
                () {
                  // Show logout confirmation
                  _showLogoutDialog(context, ref);
                },
                color: Colors.red,
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildSection(String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Text(
            title,
            style: const TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w600,
              color: Colors.grey,
            ),
          ),
        ),
        ...children,
      ],
    );
  }

  Widget _buildTile(
    BuildContext context,
    IconData icon,
    String title,
    VoidCallback onTap, {
    Color? color,
  }) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        leading: Icon(icon, color: color ?? const Color(0xFF4361EE)),
        title: Text(
          title,
          style: TextStyle(color: color),
        ),
        trailing: const Icon(Icons.arrow_forward_ios, size: 16),
        onTap: onTap,
      ),
    );
  }

  void _showLogoutDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Logout'),
        content: const Text('Are you sure you want to logout?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(context).pop();
              // ref.read(authProvider.notifier).logout();
              context.go('/auth/login');
            },
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('Logout'),
          ),
        ],
      ),
    );
  }
}
