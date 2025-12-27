#!/bin/bash

# FlashBill HTTPS Setup Script
# Menginstal SSL Let's Encrypt dengan auto-renewal

set -e

# Warna untuk output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== FlashBill HTTPS Setup with Let's Encrypt ===${NC}"
echo

# Cek apakah domain disediakan
if [ -z "$1" ]; then
    echo -e "${RED}Error: Domain name required${NC}"
    echo "Usage: sudo ./setup-https.sh yourdomain.com"
    echo "Example: sudo ./setup-https.sh api.flashbill.id"
    exit 1
fi

DOMAIN="$1"
EMAIL="admin@${DOMAIN}"

echo -e "${YELLOW}Domain: ${DOMAIN}${NC}"
echo -e "${YELLOW}Email: ${EMAIL}${NC}"
echo

# Cek apakah Nginx sudah terinstal
if ! command -v nginx &> /dev/null; then
    echo -e "${RED}Nginx not found. Installing...${NC}"
    apt-get update
    apt-get install -y nginx
fi

# Cek apakah Certbot sudah terinstal
if ! command -v certbot &> /dev/null; then
    echo -e "${RED}Certbot not found. Installing...${NC}"
    apt-get update
    apt-get install -y certbot python3-certbot-nginx
fi

# Buat direktori untuk Let's Encrypt
mkdir -p /var/www/letsencrypt
chown -R www-data:www-data /var/www/letsencrypt

# Buat konfigurasi Nginx untuk HTTP (untuk verifikasi Let's Encrypt)
echo -e "${YELLOW}Membuat konfigurasi Nginx...${NC}"

cat > /etc/nginx/sites-available/flashbill-api << EOF
server {
    listen 80;
    listen [::]:80;
    server_name ${DOMAIN} www.${DOMAIN};

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Let's Encrypt verification
    location /.well-known/acme-challenge/ {
        root /var/www/letsencrypt;
    }

    # Redirect to HTTPS
    location / {
        return 301 https://\$host\$request_uri;
    }
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name ${DOMAIN} www.${DOMAIN};

    # SSL Configuration - akan diisi oleh Certbot
    ssl_certificate /etc/letsencrypt/live/${DOMAIN}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/${DOMAIN}/privkey.pem;

    # SSL Security - Modern Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers off;

    # SSL Session
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/${DOMAIN}/chain.pem;

    # Security headers (HTTPS)
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Proxy settings
    proxy_http_version 1.1;
    proxy_set_header Upgrade \$http_upgrade;
    proxy_set_header Connection 'upgrade';
    proxy_set_header Host \$host;
    proxy_set_header X-Real-IP \$remote_addr;
    proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto \$scheme;
    proxy_cache_bypass \$http_upgrade;

    # Client max body size for file uploads
    client_max_body_size 100M;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        limit_req zone=api burst=20 nodelay;
    }

    # Health check
    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        access_log off;
    }

    # Metrics
    location /metrics {
        proxy_pass http://127.0.0.1:3000/metrics;
    }

    # WebSocket
    location /ws {
        proxy_pass http://127.0.0.1:3000/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    # Error pages
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
        internal;
    }
}
EOF

# Aktifkan site
ln -sf /etc/nginx/sites-available/flashbill-api /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test konfigurasi Nginx
echo -e "${YELLOW}Testing Nginx configuration...${NC}"
nginx -t

# Reload Nginx
echo -e "${YELLOW}Reloading Nginx...${NC}"
systemctl reload nginx 2>/dev/null || service nginx reload 2>/dev/null || echo "Note: Nginx reload skipped (systemd not available)"

echo
echo -e "${GREEN}=== Generate SSL Certificate ===${NC}"
echo

# Generate sertifikat SSL dengan Let's Encrypt
echo -e "${YELLOW}Generating SSL certificate with Let's Encrypt...${NC}"
echo -e "${YELLOW}Pastikan domain ${DOMAIN} sudah diarahkan ke server ini!${NC}"
echo
echo "Jika siap, tekan Enter untuk melanjutkan (atau Ctrl+C untuk batal)"
read -r

# Stop Nginx sementara untuk port 80
echo -e "${YELLOW}Menghentikan Nginx sementara...${NC}"
systemctl stop nginx 2>/dev/null || service nginx stop 2>/dev/null || true

# Generate sertifikat
certbot certonly --standalone \
    --preferred-challenges http \
    --agree-tos \
    --email "${EMAIL}" \
    -d "${DOMAIN}" \
    -d "www.${DOMAIN}" \
    --non-interactive

# Start Nginx kembali
echo -e "${YELLOW}Memulai Nginx kembali...${NC}"
systemctl start nginx 2>/dev/null || service nginx start 2>/dev/null || nginx

# Update konfigurasi Nginx untuk HTTPS
echo -e "${YELLOW}Memperbarui konfigurasi Nginx untuk HTTPS...${NC}"
nginx -t && systemctl reload nginx 2>/dev/null || service nginx reload 2>/dev/null || nginx -s reload

echo
echo -e "${GREEN}=== Setup Auto-Renewal ===${NC}"
echo

# Setup auto-renewal dengan cron
echo -e "${YELLOW}Membuat cron job untuk auto-renewal...${NC}"

# Cek apakah cron sudah berjalan
if ! crontab -l 2>/dev/null | grep -q "certbot renew"; then
    # Tambahkan cron job untuk renew setiap hari di jam 3 pagi
    (crontab -l 2>/dev/null; echo "0 3 * * * /usr/bin/certbot renew --quiet --deploy-hook 'systemctl reload nginx'") | crontab -
    echo -e "${GREEN}✓ Cron job added: Renewal set for 3 AM daily${NC}"
else
    echo -e "${YELLOW}Cron job already exists${NC}"
fi

# Test renew manual
echo
echo -e "${YELLOW}Testing certificate renewal...${NC}"
certbot renew --dry-run

echo
echo -e "${GREEN}=== HTTPS Setup Complete! ===${NC}"
echo
echo -e "Domain: ${GREEN}${DOMAIN}${NC}"
echo -e "SSL Certificate: ${GREEN}/etc/letsencrypt/live/${DOMAIN}/${NC}"
echo -e "Auto-renewal: ${GREEN}Daily at 3 AM${NC}"
echo
echo -e "${YELLOW}Test your HTTPS:${NC}"
echo "  curl -I https://${DOMAIN}/health"
echo
echo -e "${YELLOW}Check certificate info:${NC}"
echo "  certbot certificates"
echo
echo -e "${YELLOW}Manual renew test:${NC}"
echo "  certbot renew --dry-run"
echo
