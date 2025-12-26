# Contributing to FlashBill

Thank you for your interest in contributing to FlashBill! We welcome contributions from the community.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Bug Reports](#bug-reports)
- [Feature Requests](#feature-requests)
- [Community](#community)

## Getting Started

### Prerequisites

- Rust 1.75+
- Flutter 3.16+
- PostgreSQL 15+
- Redis 7+
- Git

### Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/flashbill.git
   cd flashbill
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/flashbill/flashbill.git
   ```

4. Setup environment:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

5. Install dependencies:
   ```bash
   # Backend
   cd backend && cargo build

   # Frontend
   cd ../frontend && flutter pub get
   ```

## Development Workflow

### Creating a Feature Branch

```bash
git checkout -b feature/amazing-feature
```

### Making Changes

1. **Backend (Rust):**
   - Follow Rust naming conventions
   - Add tests for new functionality
   - Update documentation
   - Run `cargo fmt` and `cargo clippy`

2. **Frontend (Flutter):**
   - Follow Flutter style guide
   - Use Riverpod for state management
   - Add tests for new features
   - Run `flutter format` and `flutter analyze`

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add invoice creation flow

- Implement invoice form with validation
- Add tax calculation logic
- Integrate PDF generation

Fixes #123
```

### Running Tests

```bash
# Backend
cd backend && cargo test

# Frontend
cd frontend && flutter test
```

## Code Style

### Rust

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust API guidelines
- Document public functions

### Flutter

- Use `flutter format` for formatting
- Use `flutter analyze` for linting
- Follow Flutter style guide
- Use const constructors where possible

### General

- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions small and focused
- Use meaningful variable names

## Testing

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invoice_creation() {
        // Test implementation
    }
}
```

### Frontend Tests

```dart
void main() {
  test('Invoice calculation', () {
    // Test implementation
  });
}
```

## Pull Request Process

1. **Create PR:**
   - Push your branch to your fork
   - Create PR against `develop` branch
   - Use clear PR title and description

2. **PR Checklist:**
   - [ ] Tests pass
   - [ ] Code formatted
   - [ ] Documentation updated
   - [ ] PR description includes issue reference
   - [ ] Screenshots/GIFs for UI changes

3. **Review Process:**
   - Maintainers will review your PR
   - Address feedback promptly
   - Keep PR focused and small

4. **Merge:**
   - Squash and merge for clean history
   - Delete branch after merge

## Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
A clear description of what you expected to happen.

**Environment:**
- OS: [e.g. iOS]
- Version: [e.g. 2.0.0]

**Additional context**
Add any other context about the problem.
```

## Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear description of the problem.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
A clear description of any alternative solutions.

**Additional context**
Add any other context or screenshots.
```

## Community

- **Discord:** https://discord.gg/flashbill
- **Email:** contributors@flashbill.com
- **Twitter:** @FlashBillApp

## Recognition

Contributors will be recognized in:
- Contributors file
- Release notes
- Project documentation

## Questions?

Feel free to open an issue or reach out to the maintainers!

---

**Thank you for contributing to FlashBill!** 🚀
