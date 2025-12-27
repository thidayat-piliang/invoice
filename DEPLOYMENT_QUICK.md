# FlashBill Deployment - Quick Start ⚡

**Ikuti langkah ini urut, dari 1 sampai 10!**

---

## 🎯 **CEPAT - Copy-Paste Commands**

```bash
# ============================================
# STEP 1: Install Dependencies
# ============================================
sudo apt-get update && sudo apt-get upgrade -y
sudo apt-get install -y curl wget git htop postgresql redis-server nginx certbot python3-certbot-nginx

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# ============================================
# STEP 2: Setup Database
# ============================================
sudo systemctl start postgresql
sudo systemctl enable postgresql
sudo -u postgres psql -c "CREATE DATABASE flashbill;"
sudo -u postgres psql -c "CREATE USER flashbill WITH PASSWORD 'yourpassword';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE flashbill TO flashbill;"

# Redis
sudo systemctl start redis-server
sudo systemctl enable redis-server

# ============================================
# STEP 3: Clone Project
# ============================================
cd /home
git clone https://github.com/yourusername/flashbill.git
cd flashbill

# ============================================
# STEP 4: Setup Backend
# ============================================
cd backend
cp .env.example .env
# EDIT .env FILE HERE!

# Build
cargo build --release

# Create systemd service
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

# Start backend
sudo systemctl daemon-reload
sudo systemctl enable flashbill-api
sudo systemctl start flashbill-api

# ============================================
# STEP 5: Setup Nginx (HTTP first)
# ============================================
sudo mkdir -p /var/www/letsencrypt

# Create Nginx config
sudo tee /etc/nginx/sites-available/flashbill-api > /dev/null <<EOF
server {
    listen 80;
    server_name api.flashbill.example.com;

    location /.well-known/acme-challenge/ {
        root /var/www/letsencrypt;
    }

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable site
sudo ln -sf /etc/nginx/sites-available/flashbill-api /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl reload nginx

# ============================================
# STEP 6: Generate SSL Certificate
# ============================================
# PASTIKAN domain sudah diarahkan ke server IP!

sudo certbot certonly --standalone \
    --preferred-challenges http \
    --agree-tos \
    --email admin@yourdomain.com \
    -d api.yourdomain.com

# ============================================
# STEP 7: Update Nginx for HTTPS
# ============================================
sudo tee /etc/nginx/sites-available/flashbill-api > /dev/null <<EOF
server {
    listen 80;
    server_name api.yourdomain.com;
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/api.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.yourdomain.com/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

sudo nginx -t && sudo systemctl reload nginx

# ============================================
# STEP 8: Setup Auto-Renewal
# ============================================
sudo tee /usr/local/bin/flashbill-renew.sh > /dev/null <<'EOF'
#!/bin/bash
certbot renew --quiet --deploy-hook "systemctl reload nginx"
EOF

chmod +x /usr/local/bin/flashbill-renew.sh

# Add to cron (runs daily at 3 AM)
(crontab -l 2>/dev/null | grep -v "flashbill-renew.sh"; echo "0 3 * * * /usr/local/bin/flashbill-renew.sh") | crontab -

# ============================================
# STEP 9: Test Everything
# ============================================
# Test backend
curl http://localhost:3000/health

# Test HTTPS
curl https://api.yourdomain.com/health

# Test SSL renewal
sudo certbot renew --dry-run

# ============================================
# STEP 10: Done!
# ============================================
echo "✅ Deployment Complete!"
echo "API: https://api.yourdomain.com"
echo "Health: https://api.yourdomain.com/health"
```

---

## 📋 **Checklist Verifikasi**

Setelah selesai, cek semua ini:

```bash
# 1. Backend running?
sudo systemctl status flashbill-api

# 2. Nginx running?
sudo systemctl status nginx

# 3. PostgreSQL running?
sudo systemctl status postgresql

# 4. Redis running?
sudo systemctl status redis-server

# 5. Port 3000 open?
curl http://localhost:3000/health

# 6. HTTPS working?
curl -I https://api.yourdomain.com/health

# 7. SSL valid?
certbot certificates

# 8. Auto-renewal configured?
crontab -l | grep certbot

# 9. Security headers?
curl -I https://api.yourdomain.com | grep -i strict

# 10. Logs working?
sudo journalctl -u flashbill-api -f
```

---

## 🆘 **Jika Error**

### **Backend tidak jalan:**
```bash
sudo systemctl restart flashbill-api
sudo journalctl -u flashbill-api -n 50
```

### **Nginx error:**
```bash
sudo nginx -t
sudo systemctl reload nginx
sudo tail -f /var/log/nginx/error.log
```

### **SSL error:**
```bash
sudo certbot renew --dry-run
sudo certbot renew
sudo systemctl reload nginx
```

### **Database error:**
```bash
sudo systemctl restart postgresql
sudo -u postgres psql -c "\l"
```

---

## 🎯 **Done!**

**URL:**
- API: `https://api.yourdomain.com`
- Health: `https://api.yourdomain.com/health`

**Next:**
- Setup frontend
- Add payment gateway
- Monitor logs
