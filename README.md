# FlashBill - Mobile-First Invoice App

**Version:** 2.0
**Status:** Production Ready
**Last Updated:** 2025-12-25

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Architecture](#architecture)
- [Installation](#installation)
- [Development](#development)
- [Deployment](#deployment)
- [API Documentation](#api-documentation)
- [Contributing](#contributing)

## 🚀 Overview

FlashBill is a mobile-first invoice application designed for small business owners in the US. Built with **Rust (backend)** and **Flutter (frontend)**, it provides a fast, secure, and user-friendly experience for managing invoices, clients, and payments.

### Key Metrics
- ⚡ Time-to-first-invoice: < 3 minutes
- 📱 App load time: < 2 seconds
- 🔒 99.9% uptime
- 💰 < 3% monthly churn rate

## ✨ Features

### Core Features
- ✅ Smart invoice creation with auto-calculation
- ✅ Client management with autocomplete
- ✅ Payment tracking and reminders
- ✅ US tax compliance (state-specific)
- ✅ PDF generation and sharing
- ✅ Receipt scanning (AI-powered)
- ✅ Expense tracking
- ✅ Real-time dashboard
- ✅ Automated payment reminders
- ✅ Multi-platform support (iOS, Android, Web)

### Advanced Features
- 🔄 Recurring invoices
- 📊 Advanced reporting & analytics
- 💳 Stripe/PayPal integration
- 📧 Email notifications
- 📱 Push notifications
- 🔐 Biometric authentication
- 🌙 Offline support
- 🎨 Custom branding

## 🛠 Tech Stack

### Backend (Rust)
- **Framework:** Axum + Tokio
- **Database:** PostgreSQL + Redis
- **Auth:** JWT + Argon2
- **PDF:** printpdf
- **OCR:** Tesseract
- **Email:** Lettre/Resend
- **Monitoring:** Tracing + Prometheus

### Frontend (Flutter)
- **State Management:** Riverpod 2.0
- **Navigation:** Go Router
- **Storage:** Hive/Isar
- **HTTP:** Dio
- **PDF:** printing
- **Analytics:** Firebase
- **Push:** Firebase Cloud Messaging

### Infrastructure
- **Containerization:** Docker
- **Orchestration:** Docker Compose
- **CI/CD:** GitHub Actions
- **CDN:** Cloudflare
- **Monitoring:** Grafana + Prometheus

## 🏗 Architecture

```
┌─────────────────────────────────────────────────┐
│                   Clients                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │  iOS App │  │Android App│  │ Web Admin│     │
│  │ (Flutter)│  │ (Flutter)│  │ (Flutter)│     │
│  └──────────┘  └──────────┘  └──────────┘     │
└──────────────┬─────────────────┬───────────────┘
               │                 │
               │ HTTPS (REST/WS) │
               │                 │
    ┌──────────▼─────────────────▼──────────┐
    │         API Gateway                    │
    │  ┌─────────────────────────────┐      │
    │  │   Authentication Service    │      │
    │  │   Rate Limiting             │      │
    │  │   Request Logging           │      │
    │  └─────────────────────────────┘      │
    └──────────┬─────────────────┬──────────┘
               │                 │
    ┌──────────▼─────┐ ┌─────────▼──────────┐
    │  Core Services │ │   Async Services   │
    │  ┌──────────┐  │ │  ┌──────────────┐ │
    │  │ Invoice  │  │ │  │ Notifications│ │
    │  │ Service  │  │ │  │ Service      │ │
    │  └──────────┘  │ │  └──────────────┘ │
    │  ┌──────────┐  │ │  ┌──────────────┐ │
    │  │ Payment  │  │ │  │ AI Processing│ │
    │  │ Service  │  │ │  │ Service      │ │
    │  └──────────┘  │ │  └──────────────┘ │
    │  ┌──────────┐  │ │  ┌──────────────┐ │
    │  │ Tax Calc │  │ │  │ Report Gen   │ │
    │  │ Service  │  │ │  │ Service      │ │
    │  └──────────┘  │ │  └──────────────┘ │
    └──────────┬─────┘ └─────────┬──────────┘
               │                 │
    ┌──────────▼─────────────────▼──────────┐
    │         Data Layer                     │
    │  ┌─────────────────────────────┐      │
    │  │   PostgreSQL (Primary)      │      │
    │  │   - User Data               │      │
    │  │   - Invoices                │      │
    │  │   - Clients                 │      │
    │  └─────────────────────────────┘      │
    │  ┌─────────────────────────────┐      │
    │  │   Redis (Cache/Session)     │      │
    │  │   - Session Storage         │      │
    │  │   - Rate Limit Data         │      │
    │  │   - Real-time Updates       │      │
    │  └─────────────────────────────┘      │
    │  ┌─────────────────────────────┐      │
    │  │   S3/MinIO (File Storage)   │      │
    │  │   - Receipt Images          │      │
    │  │   - Invoice PDFs            │      │
    │  │   - Logo Uploads            │      │
    │  └─────────────────────────────┘      │
    └───────────────────────────────────────┘
```

## 📦 Installation

### Prerequisites

- Rust 1.75+
- Flutter 3.16+
- PostgreSQL 15+
- Redis 7+
- Docker (optional)

### Quick Start

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/flashbill.git
   cd flashbill
   ```

2. **Setup environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Using Docker (Recommended):**
   ```bash
   make docker-up
   ```

4. **Manual Setup:**
   ```bash
   # Backend
   cd backend
   cargo build
   sqlx migrate run
   cargo run

   # Frontend
   cd ../frontend
   flutter pub get
   flutter run
   ```

## 🧪 Development

### Run Development Servers

```bash
# Backend only
make dev-backend

# Frontend only
make dev-frontend

# Both
make dev
```

### Run Tests

```bash
# All tests
make test

# Backend only
make test-backend

# Frontend only
make test-frontend
```

### Database Management

```bash
# Run migrations
make db-migrate

# Reset database
make db-reset
```

## 🚀 Deployment

### Docker Deployment

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### CI/CD Pipeline

The project uses GitHub Actions for automated testing and deployment:

- **Backend CI:** `.github/workflows/backend-ci.yml`
- **Frontend CI:** `.github/workflows/frontend-ci.yml`
- **Full Deploy:** `.github/workflows/full-deploy.yml`

### Production Checklist

- [ ] Update JWT secret
- [ ] Configure SMTP credentials
- [ ] Set up Firebase for push notifications
- [ ] Configure Stripe/PayPal
- [ ] Set up SSL certificates
- [ ] Configure domain DNS
- [ ] Enable monitoring
- [ ] Setup backups

## 📚 API Documentation

### Base URL
```
https://api.flashbill.com/api/v1
```

### Authentication
All endpoints (except auth) require Bearer token:
```
Authorization: Bearer <access_token>
```

### Endpoints

#### Auth
- `POST /auth/register` - Register new user
- `POST /auth/login` - Login
- `POST /auth/refresh` - Refresh token
- `POST /auth/forgot-password` - Request password reset
- `POST /auth/reset-password` - Reset password

#### Invoices
- `GET /invoices` - List invoices
- `POST /invoices` - Create invoice
- `GET /invoices/{id}` - Get invoice
- `PUT /invoices/{id}` - Update invoice
- `DELETE /invoices/{id}` - Delete invoice
- `POST /invoices/{id}/send` - Send invoice
- `GET /invoices/{id}/pdf` - Generate PDF

#### Clients
- `GET /clients` - List clients
- `POST /clients` - Create client
- `GET /clients/{id}` - Get client
- `PUT /clients/{id}` - Update client
- `DELETE /clients/{id}` - Delete client

#### Reports
- `GET /reports/overview` - Dashboard overview
- `GET /reports/income` - Income report
- `GET /reports/expenses` - Expense report
- `GET /reports/tax` - Tax report
- `GET /reports/aging` - Aging report

### Example Request

```bash
curl -X POST https://api.flashbill.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password123"}'
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔒 Security

For security issues, please email security@flashbill.com instead of opening an issue.

## 📞 Support

- **Documentation:** https://docs.flashbill.com
- **API Reference:** https://api.flashbill.com/docs
- **Support:** support@flashbill.com
- **Discord:** https://discord.gg/flashbill

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Mobile UI with [Flutter](https://flutter.dev/)
- Icons by [Material Design](https://material.io/)
- Database by [PostgreSQL](https://www.postgresql.org/)

---

**FlashBill © 2025. All rights reserved.** 🚀
