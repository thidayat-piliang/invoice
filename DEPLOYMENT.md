# FlashBill Deployment Guide 🚀

Lengkapkan FlashBill dari nol sampai production. **Ikuti langkah demi langkah!**

---

## 📋 **Daftar Isi**

1. [Prasyarat](#prasyarat)
2. [Setup Server](#setup-server)
3. [Install Dependencies](#install-dependencies)
4. [Deploy Backend](#deploy-backend)
5. [Deploy Frontend](#deploy-frontend)
6. [Setup HTTPS](#setup-https)
7. [Start Services](#start-services)
8. [Test Everything](#test-everything)
9. [Troubleshooting](#troubleshooting)

---

## 🔑 **Prasyarat**

### **Server Requirements:**
- **OS**: Ubuntu 20.04/22.04/24.04 (recommended)
- **RAM**: Minimum 2GB (4GB recommended)
- **CPU**: 2 cores minimum
- **Disk**: 20GB minimum

### **Domain & DNS:**
- Domain yang valid (contoh: `api.flashbill.id`)
- DNS A Record: `@` → `YOUR_SERVER_IP`
- DNS A Record: `www` → `YOUR_SERVER_IP`

### **Port yang Dibutuhkan:**
```bash
# Port 80 (HTTP) - Let's Encrypt verification
# Port 443 (HTTPS) - Production traffic
# Port 3000 (Backend API) - Internal only
```

---

## 🖥️ **Step 1: Setup Server**

### **1.1 Update System**
```bash
sudo apt-get update
sudo apt-get upgrade -y
sudo apt-get install -y curl wget git htop
```

### **1.2 Install Rust (Backend)**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Verifikasi
rustc --version
cargo --version
```

### **1.3 Install Flutter (Frontend - Optional)**
```bash
# Download Flutter
wget https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.16.0-stable.tar.xz
tar -xf flutter_linux_3.16.0-stable.tar.xz
sudo mv flutter /opt/

# Add to PATH
echo 'export PATH="$PATH:/opt/flutter/bin"' >> ~/.bashrc
source ~/.bashrc

# Verifikasi
flutter --version
```

### **1.4 Install PostgreSQL**
```bash
sudo apt-get install -y postgresql postgresql-contrib

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database & user
sudo -u postgres psql -c "CREATE DATABASE flashbill;"
sudo -u postgres psql -c "CREATE USER flashbill WITH PASSWORD 'your_secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE flashbill TO flashbill;"
```

### **1.5 Install Redis**
```bash
sudo apt-get install -y redis-server

# Start service
sudo systemctl start redis-server
sudo systemctl enable redis-server

# Verifikasi
redis-cli ping
# Should return: PONG
```

### **1.6 Install Nginx & Certbot**
```bash
sudo apt-get install -y nginx certbot python3-certbot-nginx

# Start Nginx
sudo systemctl start nginx
sudo systemctl enable nginx
```

---

## 📦 **Step 2: Clone & Setup Project**

### **2.1 Clone Repository**
```bash
cd /home
git clone https://github.com/yourusername/flashbill.git
cd flashbill
```

### **2.2 Setup Environment Variables**
```bash
# Backend
cd backend
cp .env.example .env

# Edit .env dengan editor
nano .env
```

**Isi `.env` dengan:**
```env
# Database
DATABASE_URL=postgres://flashbill:your_secure_password@localhost:5432/flashbill

# JWT
JWT_SECRET=your_super_secret_jwt_key_change_this_in_production

# Redis
REDIS_URL=redis://127.0.0.1:6379

# Server
PORT=3000
HOST=127.0.0.1

# Email (Opsional, untuk production)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
FROM_EMAIL=noreply@flashbill.id
FROM_NAME=FlashBill

# Payment Gateways
STRIPE_SECRET_KEY=sk_test_...
PAYPAL_CLIENT_ID=your_paypal_client_id
PAYPAL_CLIENT_SECRET=your_paypal_client_secret

# Firebase (Opsional)
FIREBASE_API_KEY=your_firebase_key
FCM_SERVER_KEY=your_fcm_key
```

---

## 🖥️ **Step 3: Deploy Backend (Rust API)**

### **3.1 Build Backend**
```bash
cd /home/flashbill/backend

# Build untuk production
cargo build --release

# Hasil: /home/flashbill/backend/target/release/flashbill-api
```

### **3.2 Setup Systemd Service**
```bash
sudo nano /etc/systemd/system/flashbill-api.service
```

**Isi dengan:**
```ini
[Unit]
Description=FlashBill API Server
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=trunix
WorkingDirectory=/home/flashbill/backend
EnvironmentFile=/home/flashbill/backend/.env
ExecStart=/home/flashbill/backend/target/release/flashbill-api
Restart=always
RestartSec=5

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/home/flashbill/backend/logs

[Install]
WantedBy=multi-user.target
```

### **3.3 Start Backend**
```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable auto-start
sudo systemctl enable flashbill-api

# Start service
sudo systemctl start flashbill-api

# Check status
sudo systemctl status flashbill-api

# View logs
sudo journalctl -u flashbill-api -f
```

### **3.4 Test Backend**
```bash
# Check if running
curl http://localhost:3000/health

# Should return: {"status":"ok","version":"x.x.x"}
```

---

## 📱 **Step 4: Deploy Frontend (Flutter)**

### **4.1 Build Frontend**
```bash
cd /home/flashbill/frontend

# Get dependencies
flutter pub get

# Build web (atau app)
flutter build web --release
```

### **4.2 Serve Frontend (Opsional)**
```bash
# Option 1: Serve with Nginx (Recommended)
sudo cp -r build/web /var/www/flashbill

# Option 2: Use Flutter server (Development)
flutter run -d web-server --web-port 8080 --web-hostname 0.0.0.0
```

### **4.3 Configure Nginx untuk Frontend**
```bash
sudo nano /etc/nginx/sites-available/flashbill-frontend
```

**Isi dengan:**
```nginx
server {
    listen 80;
    server_name flashbill.id www.flashbill.id;

    root /var/www/flashbill;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy
    location /api/ {
        proxy_pass http://127.0.0.1:3000/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

```bash
sudo ln -sf /etc/nginx/sites-available/flashbill-frontend /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

---

## 🔐 **Step 5: Setup HTTPS (Let's Encrypt)**

### **5.1 Jalankan HTTPS Setup**
```bash
# Ganti dengan domain Anda
sudo /home/flashbill/setup-https.sh api.flashbill.id
```

**Atau manual:**
```bash
# Generate SSL certificate
sudo certbot certonly --standalone \
    --preferred-challenges http \
    --agree-tos \
    --email admin@flashbill.id \
    -d api.flashbill.id \
    -d www.flashbill.id
```

### **5.2 Configure Nginx untuk HTTPS**
```bash
sudo /home/flashbill/configure-ssl-security.sh api.flashbill.id
```

### **5.3 Setup Auto-Renewal**
```bash
sudo /home/flashbill/setup-auto-renewal.sh
```

---

## 🔄 **Step 6: Database Migration**

### **6.1 Run Migrations**
```bash
cd /home/flashbill/backend

# Install sqlx-cli if needed
cargo install sqlx-cli

# Run migrations
sqlx database create
sqlx migrate run
```

### **6.2 Seed Database (Opsional)**
```bash
# Jika ada seed script
cargo run --bin seed
```

---

## 🚀 **Step 7: Start Everything**

### **7.1 Start Services**
```bash
# PostgreSQL (sudah jalan)
sudo systemctl status postgresql

# Redis (sudah jalan)
sudo systemctl status redis-server

# Backend API
sudo systemctl start flashbill-api
sudo systemctl enable flashbill-api

# Nginx
sudo systemctl start nginx
sudo systemctl enable nginx
```

### **7.2 Check Status**
```bash
# Semua service
sudo systemctl list-units --type=service | grep -E 'postgres|redis|flashbill|nginx'

# Logs
sudo journalctl -u flashbill-api -f
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

---

## ✅ **Step 8: Test Everything**

### **8.1 Test Backend API**
```bash
# Health check
curl https://api.flashbill.id/health

# Login test
curl -X POST https://api.flashbill.id/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

### **8.2 Test Frontend**
```bash
# Open in browser
https://flashbill.id

# Or check if served
curl -I https://flashbill.id
```

### **8.3 Test SSL**
```bash
# Check certificate
certbot certificates

# Test renewal
certbot renew --dry-run

# SSL Labs test
# https://www.ssllabs.com/ssltest/analyze.html?d=api.flashbill.id
```

### **8.4 Test All Endpoints**
```bash
# Health
curl https://api.flashbill.id/health

# Metrics (if enabled)
curl https://api.flashbill.id/metrics

# API endpoints
curl https://api.flashbill.id/api/invoices
```

---

## 📊 **Monitoring & Maintenance**

### **Check Logs**
```bash
# Backend
sudo journalctl -u flashbill-api -f

# Nginx
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log

# SSL renewal
sudo tail -f /var/log/flashbill-ssl-renew.log
```

### **Restart Services**
```bash
# Backend only
sudo systemctl restart flashbill-api

# Nginx only
sudo systemctl reload nginx

# All
sudo systemctl restart flashbill-api nginx
```

### **Update Backend**
```bash
cd /home/flashbill/backend
git pull origin main
cargo build --release
sudo systemctl restart flashbill-api
```

---

## 🆘 **Troubleshooting**

### **Problem: Backend tidak bisa connect ke database**
```bash
# Cek PostgreSQL
sudo systemctl status postgresql

# Cek database
sudo -u postgres psql -c "\l"

# Cek user
sudo -u postgres psql -c "\du"
```

### **Problem: Port 3000 tidak bisa diakses**
```bash
# Cek apakah port terbuka
sudo netstat -tlnp | grep 3000

# Cek firewall
sudo ufw status

# Cek service
sudo systemctl status flashbill-api
```

### **Problem: Nginx error 502 Bad Gateway**
```bash
# Backend mati
sudo systemctl status flashbill-api

# Cek port
curl http://localhost:3000/health

# Restart backend
sudo systemctl restart flashbill-api
```

### **Problem: SSL certificate expired**
```bash
# Manual renew
sudo certbot renew

# Cek expiry
openssl x509 -enddate -noout -in /etc/letsencrypt/live/api.flashbill.id/fullchain.pem
```

### **Problem: Auto-renewal tidak jalan**
```bash
# Test manual
sudo certbot renew --dry-run

# Cek cron
crontab -l

# Cek log
sudo grep CRON /var/log/syslog
```

---

## 🎯 **Production Checklist**

### **Before Going Live:**
- [ ] Domain configured and pointing to server
- [ ] PostgreSQL running with correct database
- [ ] Redis running
- [ ] Backend API running on port 3000
- [ ] Nginx configured for HTTPS
- [ ] SSL certificate installed
- [ ] Auto-renewal configured
- [ ] Security headers enabled
- [ ] Rate limiting enabled
- [ ] Firewall configured (ports 80, 443 open)
- [ ] Environment variables set correctly
- [ ] JWT secret changed from default
- [ ] Database password secure
- [ ] Logs configured
- [ ] Backup strategy in place

### **Security:**
- [ ] Change default JWT secret
- [ ] Use strong database password
- [ ] Enable firewall (ufw)
- [ ] Disable root login
- [ ] Use SSH keys
- [ ] SSL Labs A+ rating
- [ ] Security headers test passed

### **Monitoring:**
- [ ] Backend health check working
- [ ] Nginx access logs configured
- [ ] Error logs monitored
- [ ] SSL expiry alerts
- [ ] Disk space monitoring
- [ ] RAM/CPU monitoring

---

## 📝 **Quick Commands Reference**

```bash
# Status semua service
sudo systemctl status postgresql redis-server flashbill-api nginx

# Restart semua
sudo systemctl restart postgresql redis-server flashbill-api nginx

# Logs
sudo journalctl -u flashbill-api -f
sudo tail -f /var/log/nginx/access.log

# SSL
certbot certificates
certbot renew --dry-run

# Test
curl https://api.flashbill.id/health
```

---

## 🎉 **Selesai!**

FlashBill sudah siap production!

**URL:**
- API: `https://api.flashbill.id`
- Frontend: `https://flashbill.id`

**Support:**
- HTTPS Setup: `HTTPS_SETUP.md`
- SSL Issues: `certbot --help`
