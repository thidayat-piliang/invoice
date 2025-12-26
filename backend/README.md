# FlashBill Backend API

Rust-based backend API for FlashBill invoice application.

## Tech Stack

- **Framework:** Axum + Tokio
- **Database:** PostgreSQL + Redis
- **Auth:** JWT + Argon2
- **PDF:** printpdf
- **Email:** Lettre
- **Monitoring:** Tracing + Prometheus

## Setup

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install SQLx CLI: `cargo install sqlx-cli`
3. Copy `.env.example` to `.env` and configure
4. Run migrations: `sqlx migrate run`
5. Run: `cargo run`

## API Endpoints

### Auth
- `POST /api/v1/auth/register` - Register
- `POST /api/v1/auth/login` - Login
- `POST /api/v1/auth/refresh` - Refresh token
- `POST /api/v1/auth/forgot-password` - Forgot password
- `POST /api/v1/auth/reset-password` - Reset password
- `GET /api/v1/auth/me` - Get current user
- `PUT /api/v1/auth/me` - Update profile

### Invoices
- `GET /api/v1/invoices` - List invoices
- `POST /api/v1/invoices` - Create invoice
- `GET /api/v1/invoices/{id}` - Get invoice
- `PUT /api/v1/invoices/{id}` - Update invoice
- `DELETE /api/v1/invoices/{id}` - Delete invoice
- `POST /api/v1/invoices/{id}/send` - Send invoice
- `GET /api/v1/invoices/{id}/pdf` - Get PDF

### Clients
- `GET /api/v1/clients` - List clients
- `POST /api/v1/clients` - Create client
- `GET /api/v1/clients/{id}` - Get client
- `PUT /api/v1/clients/{id}` - Update client
- `DELETE /api/v1/clients/{id}` - Delete client

### Payments
- `GET /api/v1/payments` - List payments
- `POST /api/v1/payments` - Create payment
- `POST /api/v1/payments/{id}/refund` - Refund payment

### Reports
- `GET /api/v1/reports/overview` - Dashboard
- `GET /api/v1/reports/income` - Income report
- `GET /api/v1/reports/tax` - Tax report

## Database

### Migrations
```bash
# Create migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert migration
sqlx migrate revert
```

### Schema
See `infrastructure/database/migrations/001_initial_schema.sql`

## Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Benchmarks
cargo bench
```

## Docker

```bash
# Build
docker build -t flashbill-api .

# Run
docker run -p 3000:3000 --env-file .env flashbill-api
```

## Production

### Environment Variables
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret key for JWT
- `SMTP_*` - Email configuration
- `PORT` - Server port (default: 3000)

### Security Checklist
- [ ] Use strong JWT secret
- [ ] Enable HTTPS/TLS
- [ ] Configure rate limiting
- [ ] Set up firewall rules
- [ ] Enable SQL injection protection
- [ ] Configure CORS properly
- [ ] Enable request logging
- [ ] Set up monitoring

### Monitoring
- Health check: `GET /health`
- Ready check: `GET /ready`
- Metrics: `GET /metrics` (Prometheus format)

## License

MIT
