# FlashBill API Integration Tests

This directory contains comprehensive integration tests for the FlashBill API using REST client testing.

## Overview

The integration tests cover all major features of the FlashBill API:

- **Authentication**: Registration, login, profile management
- **Clients**: CRUD operations, client statistics, client invoices
- **Invoices**: Creation, sending, PDF generation, reminders, payments
- **Payments**: Payment processing, refunds, statistics, payment methods
- **Expenses**: Expense tracking, categorization, statistics
- **Reports**: Overview, income, expenses, tax, aging reports, export
- **Settings**: Business, tax, notification, and invoice settings

## Test Structure

```
tests/
├── integration/
│   ├── mod.rs              # Module declarations
│   ├── test_client.rs      # REST API client wrapper
│   ├── utils.rs            # Test utilities and helpers
│   ├── auth_test.rs        # Authentication tests
│   ├── clients_test.rs     # Client management tests
│   ├── invoices_test.rs    # Invoice management tests
│   ├── payments_test.rs    # Payment processing tests
│   ├── expenses_test.rs    # Expense tracking tests
│   ├── reports_test.rs     # Reporting tests
│   └── settings_test.rs    # Settings management tests
├── integration_test.rs     # Main test entry point
└── README.md               # This file
```

## Prerequisites

1. **PostgreSQL Database**: A running PostgreSQL instance
2. **Test Database**: Create a dedicated test database
3. **API Server**: The FlashBill API must be running

## Setup

### 1. Create Test Database

```bash
createdb flashbill_test
```

### 2. Set Environment Variables

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/flashbill_test"
export API_URL="http://localhost:3000"
```

### 3. Run Migrations

Start the API server to run migrations automatically:

```bash
cargo run --bin flashbill-api
```

Or run migrations manually:

```bash
sqlx migrate run --database-url $DATABASE_URL
```

## Running Tests

### Option 1: Using the Test Script (Recommended)

The `run_integration_tests.sh` script handles everything:

```bash
./run_integration_tests.sh
```

This script will:
- Check if the API is running
- Start the API if needed
- Run all integration tests
- Clean up (stop API if started by script)

### Option 2: Manual Test Execution

1. Start the API server:
```bash
cargo run --bin flashbill-api &
```

2. Run tests:
```bash
cargo test --test integration_test
```

3. Run specific test module:
```bash
cargo test --test integration_test auth_test::
```

4. Run a specific test:
```bash
cargo test --test integration_test test_full_auth_flow
```

## Test Coverage

### Auth Tests (`auth_test.rs`)
- ✅ Full registration and login flow
- ✅ Profile updates
- ✅ Token-based authentication
- ✅ Validation errors
- ✅ Protected endpoint access

### Clients Tests (`clients_test.rs`)
- ✅ Create, read, update, delete clients
- ✅ List with filters
- ✅ Client statistics
- ✅ Client invoice relationships
- ✅ 404 handling

### Invoices Tests (`invoices_test.rs`)
- ✅ Invoice creation with items
- ✅ Multi-item invoices with tax
- ✅ Status transitions (draft → sent → paid)
- ✅ PDF generation
- ✅ Sending and reminders
- ✅ Partial payments

### Payments Tests (`payments_test.rs`)
- ✅ Payment creation
- ✅ Refund processing
- ✅ Partial payments
- ✅ Multiple payment methods
- ✅ Payment statistics

### Expenses Tests (`expenses_test.rs`)
- ✅ Expense CRUD operations
- ✅ Category filtering
- ✅ Tax-deductible tracking
- ✅ Expense statistics
- ✅ Validation

### Reports Tests (`reports_test.rs`)
- ✅ Overview statistics
- ✅ Income reports
- ✅ Expense reports
- ✅ Tax reports
- ✅ Aging reports
- ✅ Export functionality (CSV/PDF)
- ✅ Date range filtering

### Settings Tests (`settings_test.rs`)
- ✅ Business settings (company name, phone, etc.)
- ✅ Tax settings (state, rate, exemption)
- ✅ Notification preferences
- ✅ Invoice settings (template, terms, notes)
- ✅ Settings persistence

## Test Data Management

Each test creates its own unique data:
- Unique email addresses using timestamps
- Automatic cleanup after each test
- No interference between tests

## Error Handling Tests

All tests verify:
- ✅ Success responses (200, 201, 204)
- ✅ Error responses (400, 401, 404, 403)
- ✅ Validation errors
- ✅ Authentication failures
- ✅ Missing resource handling

## Best Practices

1. **Always clean up**: Tests delete their created data
2. **Unique data**: Use timestamps to avoid conflicts
3. **Independent tests**: Each test works standalone
4. **Clear assertions**: Verify both status codes and response data
5. **Comprehensive coverage**: Test happy path and error cases

## Troubleshooting

### API not found
```bash
# Check if API is running
curl http://localhost:3000/health

# Start API if needed
cargo run --bin flashbill-api
```

### Database connection failed
```bash
# Verify DATABASE_URL
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL
```

### Tests timing out
- Increase API timeout in `test_client.rs`
- Check database performance
- Verify API server resources

## CI/CD Integration

For CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Setup PostgreSQL
  uses: postgresql-action@v1
  with:
    postgresql version: 15
    database: flashbill_test

- name: Run migrations
  run: sqlx migrate run

- name: Start API
  run: cargo run --bin flashbill-api &

- name: Run integration tests
  run: cargo test --test integration_test
```

## Performance Notes

- Tests run sequentially (`--test-threads=1`) to avoid database conflicts
- Each test typically takes 100-500ms
- Full suite: ~10-20 seconds
- API startup: ~2-5 seconds

## Future Enhancements

- [ ] Add performance benchmarks
- [ ] Add concurrent user simulation
- [ ] Add file upload/download tests
- [ ] Add WebSocket tests for real-time features
- [ ] Add stress tests
- [ ] Add security penetration tests
