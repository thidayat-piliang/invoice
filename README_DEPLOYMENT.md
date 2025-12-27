# FlashBill Deployment - Complete Guide 📚

**Bingung deployment? Ikuti panduan ini step-by-step!**

---

## 🎯 **Pilih Cara Deployment Anda:**

### **Opsi 1: Cepat (Copy-Paste)**
→ Buka: `DEPLOYMENT_QUICK.md`
- Semua command siap pakai
- Estimasi: 30 menit

### **Opsi 2: Lengkap (Step-by-Step)**
→ Buka: `DEPLOYMENT.md`
- Penjelasan detail setiap langkah
- Troubleshooting lengkap
- Estimasi: 1-2 jam

### **Opsi 3: HTTPS Only**
→ Buka: `HTTPS_SETUP.md`
- Hanya untuk SSL/HTTPS
- Estimasi: 15 menit

---

## 📋 **Apa yang Dibutuhkan?**

### **Server:**
- Ubuntu 20.04/22.04/24.04
- RAM 2GB minimum (4GB recommended)
- Port 80 & 443 terbuka

### **Domain:**
- Domain yang valid (contoh: `api.flashbill.id`)
- DNS A Record → IP Server

### **Skill:**
- Basic Linux commands
- Tidak perlu Rust/Flutter expert (sudah dibuild)

---

## 🚀 **Mulai Sekarang!**

### **Step 1: Siapkan Server**
```bash
# Login ke server
ssh user@your-server-ip

# Update system
sudo apt-get update && sudo apt-get upgrade -y
```

### **Step 2: Install Semua Dependencies**
```bash
# Copy-paste ini
sudo apt-get install -y \
  curl wget git htop \
  postgresql postgresql-contrib \
  redis-server \
  nginx certbot python3-certbot-nginx

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### **Step 3: Setup Database**
```bash
sudo systemctl start postgresql
sudo systemctl enable postgresql

sudo -u postgres psql -c "CREATE DATABASE flashbill;"
sudo -u postgres psql -c "CREATE USER flashbill WITH PASSWORD 'yourpassword';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE flashbill TO flashbill;"

sudo systemctl start redis-server
sudo systemctl enable redis-server
```

### **Step 4: Clone & Deploy**
```bash
cd /home
git clone https://github.com/yourusername/flashbill.git
cd flashbill

# Setup backend
cd backend
cp .env.example .env
# EDIT .env SEKARANG!

cargo build --release

# Setup systemd
sudo tee /etc/systemd/system/flashbill-api.service > /dev/null <<EOF
[Unit]
Description=FlashBill API
After=network.target

[Service]
Type=simple
User=trunix
WorkingDirectory=/home/flashbill/backend
EnvironmentFile=/home/flashbill/backend/.env
ExecStart=/home/flashbill/backend/target/release/flashbill-api
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable flashbill-api
sudo systemctl start flashbill-api
```

### **Step 5: Setup HTTPS**
```bash
# Install Nginx & Certbot
sudo apt-get install -y nginx certbot python3-certbot-nginx

# Jalankan HTTPS setup
sudo /home/flashbill/setup-https.sh api.yourdomain.com
```

**Selesai!** API Anda sudah jalan di `https://api.yourdomain.com`

---

## 🧪 **Test Setup**

```bash
# Test backend
curl http://localhost:3000/health

# Test HTTPS
curl https://api.yourdomain.com/health

# Test SSL
certbot certificates
```

---

## 📖 **Dokumentasi Lengkap**

| File | Isi |
|------|-----|
| **`DEPLOYMENT_QUICK.md`** | Command cepat, copy-paste |
| **`DEPLOYMENT.md`** | Panduan lengkap dengan penjelasan |
| **`HTTPS_SETUP.md`** | Setup SSL detail |
| **`DEPLOYMENT_ARCHITECTURE.md`** | Diagram & arsitektur |

---

## 🆘 **Butuh Bantuan?**

### **Error Umum:**

**1. Backend tidak jalan**
```bash
sudo systemctl restart flashbill-api
sudo journalctl -u flashbill-api -n 50
```

**2. Nginx error**
```bash
sudo nginx -t
sudo systemctl reload nginx
```

**3. SSL error**
```bash
sudo certbot renew --dry-run
```

---

## 🎯 **Ringkasan**

```
┌─────────────────────────────────────┐
│  Server Ubuntu                      │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  Nginx (HTTPS)                │ │
│  │  Port: 80, 443                │ │
│  └───────────────┬───────────────┘ │
│                  │                 │
│  ┌───────────────▼───────────────┐ │
│  │  FlashBill API (Rust)        │ │
│  │  Port: 3000                  │ │
│  └───────────────┬───────────────┘ │
│                  │                 │
│  ┌───────────────▼───────────────┐ │
│  │  PostgreSQL + Redis          │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

**URL:**
- API: `https://api.yourdomain.com`
- Health: `https://api.yourdomain.com/health`

---

## ✅ **Checklist Deployment**

- [ ] Server siap (Ubuntu, RAM 2GB+)
- [ ] Domain + DNS configured
- [ ] Dependencies installed
- [ ] Database setup
- [ ] Backend built & running
- [ ] HTTPS configured
- [ ] Auto-renewal working
- [ ] Test passed

---

## 🚀 **Ready to Deploy!**

Pilih panduan:
1. **Pemula** → `DEPLOYMENT_QUICK.md`
2. **Lengkap** → `DEPLOYMENT.md`
3. **HTTPS Only** → `HTTPS_SETUP.md`

**Semua script sudah disiapkan, tinggal ikuti langkah!** 🎉
