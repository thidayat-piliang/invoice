# FlashBill API - Professional Invoice Management Backend

A production-ready Rust backend API for FlashBill invoice management system, built with modern async architecture and enterprise-grade features.

## 🚀 Features

### Core Functionality
- **Invoice Management**: Create, update, delete, send invoices with auto-generated numbers
- **Client Management**: Full CRUD for customer database
- **Payment Processing**: Track payments, refunds, and payment history
- **Expense Tracking**: Categorized expense management with reporting
- **Tax Management**: Informational tax settings and calculation (MVP - see Legal Disclaimer)
- **Multi-user Support**: Role-based access control with JWT authentication

### Advanced Features
- **PDF Generation**: Professional invoice PDFs with company branding
- **CSV Export**: Financial reports exportable to CSV format
- **Email Integration**: Automated email sending (invoices, reminders, notifications)
- **Push Notifications**: FCM integration for mobile app notifications
- **File Upload**: Secure file handling for receipts and attachments
- **Redis Caching**: High-performance caching for reports and frequent queries
- **Rate Limiting**: Distributed rate limiting with Redis backend
- **Prometheus Metrics**: Comprehensive application metrics and monitoring
- **Distributed Tracing**: OpenTelemetry integration for request tracing

### Security & Production
- **CORS Configuration**: Configurable cross-origin resource sharing
- **Request Limits**: 10MB request body limit
- **Request Timeouts**: 30-second timeout for all requests
- **Security Headers**: HSTS, X-Content-Type-Options, X-Frame-Options, X-XSS-Protection
- **Password Security**: Argon2 hashing with salt
- **JWT Authentication**: Secure token-based authentication
- **SQL Injection Protection**: Type-safe SQLx queries

## 🏗️ Architecture

```
src/
├── api/                    # API Layer (Axum handlers, routes, middleware)
│   ├── routes/            # Route definitions
│   ├── middleware/        # Auth, rate limiting, metrics, logging
│   └── error.rs           # Unified error handling
├── application/           # Application Layer (Use Cases)
│   ├── use_cases/         # Business logic orchestration
│   └── dto/               # Data Transfer Objects
├── domain/                # Domain Layer (Business Logic)
│   ├── models/            # Domain models
│   ├── repositories/      # Repository traits
│   └── services/          # Domain services
└── infrastructure/        # Infrastructure Layer
    ├── repositories/      # Database implementations
    └── database/          # Database migrations
```

## 🛠️ Tech Stack

| Category | Technology | Version |
|----------|------------|---------|
| **Web Framework** | Axum | 0.8.8 |
| **Async Runtime** | Tokio | 1.x |
| **Database** | PostgreSQL (SQLx) | 0.8.6 |
| **Cache** | Redis | 1.0.2 |
| **Auth** | JWT + Argon2 | - |
| **PDF** | printpdf | 0.8.2 |
| **Email** | lettre | 0.11 |
| **CSV** | csv | 1.3 |
| **Metrics** | Prometheus + OpenTelemetry | 0.14/0.31 |
| **Rate Limiting** | governor | 0.10.4 |
| **HTTP Client** | reqwest | 0.12.28 |

## 📦 Installation & Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLx CLI
cargo install sqlx-cli

# Install PostgreSQL & Redis
# Ubuntu/Debian:
sudo apt-get install postgresql redis-server

# Docker alternative:
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres
docker run -d -p 6379:6379 redis
```

### Configuration

1. **Create `.env` file:**
```env
# Server
PORT=3000
CORS_ORIGIN=http://localhost:3000

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/flashbill

# Redis
REDIS_URL=redis://127.0.0.1:6379

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# Email (SMTP)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
FROM_EMAIL=noreply@flashbill.com
FROM_NAME=FlashBill

# File Upload
FILE_UPLOAD_DIR=./uploads
MAX_FILE_SIZE=10485760

# FCM (Firebase Cloud Messaging) - Optional
FCM_SERVER_KEY=your-fcm-server-key

# Payment Gateways - Optional
STRIPE_SECRET_KEY=sk_test_...
PAYPAL_CLIENT_ID=your-paypal-client-id
PAYPAL_CLIENT_SECRET=your-paypal-secret
```

2. **Initialize Database:**
```bash
# Create database
createdb flashbill

# Run migrations
sqlx migrate run
```

3. **Build & Run:**
```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/flashbill-api

# With specific port
PORT=8080 cargo run
```

## 📡 API Endpoints

### Authentication
```
POST   /api/v1/auth/register              # Register new user
POST   /api/v1/auth/login                 # Login
POST   /api/v1/auth/refresh               # Refresh token
POST   /api/v1/auth/forgot-password       # Request password reset
POST   /api/v1/auth/reset-password        # Reset password
POST   /api/v1/auth/verify-email          # Verify email
GET    /api/v1/auth/me                    # Get current user
PUT    /api/v1/auth/me                    # Update profile
```

### Invoices
```
GET    /api/v1/invoices                   # List invoices (with filters)
POST   /api/v1/invoices                   # Create invoice
GET    /api/v1/invoices/{id}              # Get invoice by ID
PUT    /api/v1/invoices/{id}              # Update invoice
DELETE /api/v1/invoices/{id}              # Delete invoice
POST   /api/v1/invoices/{id}/send         # Send invoice via email
GET    /api/v1/invoices/{id}/pdf          # Download PDF
POST   /api/v1/invoices/{id}/remind       # Send payment reminder
POST   /api/v1/invoices/{id}/payments     # Record payment
```

### Clients
```
GET    /api/v1/clients                    # List clients
POST   /api/v1/clients                    # Create client
GET    /api/v1/clients/{id}               # Get client
PUT    /api/v1/clients/{id}               # Update client
DELETE /api/v1/clients/{id}               # Delete client
GET    /api/v1/clients/{id}/invoices      # Get client's invoices
GET    /api/v1/clients/{id}/stats         # Get client statistics
```

### Payments
```
GET    /api/v1/payments                   # List payments
POST   /api/v1/payments                   # Create payment
GET    /api/v1/payments/{id}              # Get payment
POST   /api/v1/payments/{id}/refund       # Refund payment
GET    /api/v1/payments/stats             # Payment statistics
GET    /api/v1/payments/methods           # Available payment methods
```

### Expenses
```
GET    /api/v1/expenses                   # List expenses
POST   /api/v1/expenses                   # Create expense
GET    /api/v1/expenses/{id}              # Get expense
PUT    /api/v1/expenses/{id}              # Update expense
DELETE /api/v1/expenses/{id}              # Delete expense
GET    /api/v1/expenses/stats             # Expense statistics
```

### Reports
```
GET    /api/v1/reports/overview           # Dashboard overview
GET    /api/v1/reports/income             # Income report
GET    /api/v1/reports/expenses           # Expenses report
GET    /api/v1/reports/tax                # Tax report
GET    /api/v1/reports/aging              # Aging report
POST   /api/v1/reports/export             # Export report (CSV/PDF)
```

### Files
```
POST   /api/v1/files/upload               # Upload file
GET    /api/v1/files/{id}                 # Download file
GET    /api/v1/files                      # List files
DELETE /api/v1/files/{id}                 # Delete file
```

### Tax Management
```
POST   /api/v1/settings/tax               # Create tax setting
GET    /api/v1/settings/tax               # Get all tax settings
GET    /api/v1/settings/tax/default       # Get default tax setting
PUT    /api/v1/settings/tax/{id}          # Update tax setting
DELETE /api/v1/settings/tax/{id}          # Delete tax setting
POST   /api/v1/tax/calculate              # Calculate tax for amount
POST   /api/v1/tax/summary                # Get tax summary for period
POST   /api/v1/tax/validate               # Validate tax ID
```

### System
```
GET    /health                            # Health check
GET    /ready                             # Readiness check
GET    /metrics                           # Prometheus metrics
```

## 🗄️ Database Schema

### Tables
- `users` - User accounts and profiles
- `clients` - Customer information
- `invoices` - Invoice records with line items (includes tax_label, tax_id)
- `payments` - Payment transactions
- `expenses` - Business expenses
- `tax_settings` - Tax configuration (label, rate, is_default, is_active)
- `refresh_tokens` - JWT token management
- `files` - File metadata

### Migrations
```bash
# Create new migration
sqlx migrate add add_new_feature

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Generate SQL schema
sqlx migrate script
```

## 🧪 Testing

### Unit & Integration Tests
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_create_invoice

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Database Setup
```bash
# Create test database
createdb flashbill_test

# Run migrations on test DB
DATABASE_URL=postgres://postgres:postgres@localhost:5432/flashbill_test sqlx migrate run

# Run tests
DATABASE_URL=postgres://postgres:postgres@localhost:5432/flashbill_test cargo test
```

## 🐳 Docker Deployment

### Dockerfile
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/flashbill-api /usr/local/bin
EXPOSE 3000
CMD ["flashbill-api"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/flashbill
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=production-secret-key
    depends_on:
      - db
      - redis

  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: flashbill
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

volumes:
  pgdata:
```

## 📊 Monitoring & Observability

### Prometheus Metrics
Available at `GET /metrics`:
- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency
- `business_invoices_total` - Total invoices created
- `business_revenue_total` - Total revenue
- `cache_hits_total` - Cache hit count
- `cache_misses_total` - Cache miss count
- `error_total` - Error count by type

### Health Checks
```bash
# Health check (always returns 200)
curl http://localhost:3000/health

# Readiness check (verifies DB, Redis connectivity)
curl http://localhost:3000/ready
```

### Distributed Tracing
OpenTelemetry traces are sent to configured OTLP endpoint:
```env
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=flashbill-api
```

## 🔒 Security Best Practices

### Production Checklist
- [ ] Use strong, unique JWT secret (min 32 chars)
- [ ] Enable HTTPS/TLS (use reverse proxy like Nginx)
- [ ] Configure CORS to specific origins only
- [ ] Use environment variables for all secrets
- [ ] Enable Redis authentication
- [ ] Set up PostgreSQL connection limits
- [ ] Configure firewall rules
- [ ] Enable request logging with sensitive data filtering
- [ ] Set up automated backups
- [ ] Monitor logs for suspicious activity
- [ ] Keep dependencies updated
- [ ] Use prepared statements (SQLx does this automatically)

### Rate Limiting
Default configuration:
- 100 requests per minute per IP
- 1000 requests per hour per user
- Distributed across Redis instances

### Data Validation
- Input validation using `validator` crate
- SQL injection protection via SQLx type-safe queries
- XSS protection via security headers
- File upload validation (size, type, extension)

## ⚠️ Legal Disclaimer - Tax Module

### Important Notice
The tax module is an **MVP (Minimum Viable Product)** for informational purposes only:

**What This Module Does:**
- ✅ Stores tax settings (label, rate, is_default)
- ✅ Calculates tax on invoices using stored rates
- ✅ Displays tax information on PDF invoices
- ✅ Generates tax summaries for reporting periods
- ✅ Validates tax ID format

**What This Module Does NOT Do:**
- ❌ **Does NOT calculate, verify, or file taxes on your behalf**
- ❌ **Does NOT provide tax compliance or legal advice**
- ❌ **Does NOT handle tax jurisdiction rules**
- ❌ **Does NOT integrate with government tax systems**
- ❌ **Does NOT guarantee tax calculation accuracy**

### Required Legal Disclaimers
All PDF invoices generated by this system include the following disclaimer:
> "Tax information is for informational purposes only. FlashBill does not calculate, verify, or file taxes on your behalf."

### User Responsibility
Users of this software are **solely responsible for**:
1. Verifying all tax calculations
2. Ensuring compliance with local, state, and federal tax laws
3. Filing appropriate tax returns
4. Consulting with tax professionals for accuracy

### Tax ID Validation
The tax ID validation feature only checks format validity (e.g., EIN format). It does NOT verify:
- If the tax ID actually exists
- If it belongs to the claimed business
- Current tax registration status

---

## 🚀 Performance Optimization

### Caching Strategy
- **Report Data**: Cached in Redis for 5 minutes
- **User Sessions**: Cached for 1 hour
- **Client Lists**: Cached for 10 minutes
- **Metrics**: Real-time, no caching

### Database Optimization
- Connection pooling (max 10 connections)
- Indexes on frequently queried columns
- Query optimization with SQLx prepared statements
- Connection retry logic

### Async Architecture
- Non-blocking I/O for all operations
- Concurrent request handling
- Background task processing for emails/notifications

## 🐛 Troubleshooting

### Common Issues

**Port already in use:**
```bash
lsof -i :3000
kill -9 <PID>
```

**Database connection failed:**
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Verify connection string
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL
```

**Redis connection failed:**
```bash
# Check Redis status
sudo systemctl status redis

# Test connection
redis-cli ping
```

**Migration errors:**
```bash
# Reset database
sqlx migrate revert
sqlx migrate run

# Check migration status
sqlx migrate info
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=flashbill_api=debug,tower_http=debug

# Run with verbose output
cargo run -- --verbose
```

## 📚 API Examples

### Create Invoice
```bash
curl -X POST http://localhost:3000/api/v1/invoices \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "550e8400-e29b-41d4-a716-446655440000",
    "items": [
      {
        "description": "Web Development",
        "quantity": 10,
        "unit_price": 100.0
      }
    ],
    "due_date": "2024-12-31"
  }'
```

### Export Report
```bash
curl -X POST http://localhost:3000/api/v1/reports/export \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "report_type": "income",
    "format": "csv",
    "start_date": "2024-01-01",
    "end_date": "2024-12-31"
  }' \
  --output report.csv
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style
- Use `cargo fmt` before committing
- Run `cargo clippy` for linting
- Follow Rust naming conventions
- Add tests for new features

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum)
- PDF generation with [printpdf](https://github.com/fschutt/printpdf)
- Metrics with [Prometheus](https://prometheus.io/)
- Distributed tracing with [OpenTelemetry](https://opentelemetry.io/)

---

**FlashBill API** - Professional invoice management for modern businesses 🚀
