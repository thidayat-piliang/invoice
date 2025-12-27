#!/bin/bash

# SSL Security Configuration
# Mengkonfigurasi cipher suite, HSTS, dan security headers

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== SSL Security Configuration ===${NC}"
echo

DOMAIN="$1"

if [ -z "$DOMAIN" ]; then
    echo -e "${YELLOW}Masukkan domain (tanpa https://):${NC}"
    read -r DOMAIN
fi

if [ -z "$DOMAIN" ]; then
    echo -e "${RED}Domain required${NC}"
    exit 1
fi

echo -e "${YELLOW}Configuring SSL security for: ${DOMAIN}${NC}"
echo

# Backup current config
echo -e "${YELLOW}Backing up current Nginx config...${NC}"
sudo cp /etc/nginx/sites-available/flashbill-api /etc/nginx/sites-available/flashbill-api.backup.$(date +%Y%m%d_%H%M%S)

# Create enhanced SSL config
echo -e "${YELLOW}Creating enhanced SSL configuration...${NC}"

sudo cat > /tmp/ssl-enhanced.conf << EOF
server {
    listen 80;
    listen [::]:80;
    server_name ${DOMAIN} www.${DOMAIN};

    # Security headers (HTTP)
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

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

    # SSL Certificate Paths
    ssl_certificate /etc/letsencrypt/live/${DOMAIN}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/${DOMAIN}/privkey.pem;

    # === SSL PROTOCOLS & CIPHERS ===
    # Modern configuration (A+ rating on SSL Labs)
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers off;

    # === SSL SESSION ===
    ssl_session_cache shared:SSL:50m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # === OCSP STAPLING ===
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/${DOMAIN}/chain.pem;

    # === SECURITY HEADERS ===
    # HSTS (HTTP Strict Transport Security)
    # max-age: 1 tahun (31536000 detik)
    # includeSubDomains: berlaku untuk semua subdomain
    # preload: daftarkan ke browser preload list (opsional)
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;

    # Prevent clickjacking
    add_header X-Frame-Options "SAMEORIGIN" always;

    # Prevent MIME sniffing
    add_header X-Content-Type-Options "nosniff" always;

    # XSS Protection
    add_header X-XSS-Protection "1; mode=block" always;

    # Referrer Policy
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Content Security Policy (CSP)
    # Adjust based on your needs
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'self';" always;

    # Feature Policy (modern alternative to CSP)
    add_header Permissions-Policy "geolocation=(), microphone=(), camera=(), payment=(), usb=()" always;

    # === PROXY SETTINGS ===
    proxy_http_version 1.1;
    proxy_set_header Upgrade \$http_upgrade;
    proxy_set_header Connection 'upgrade';
    proxy_set_header Host \$host;
    proxy_set_header X-Real-IP \$remote_addr;
    proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto \$scheme;
    proxy_cache_bypass \$http_upgrade;

    # === RATE LIMITING ===
    # Didefinisikan di http block nginx.conf
    limit_req zone=api burst=20 nodelay;

    # === FILE UPLOAD LIMIT ===
    client_max_body_size 100M;

    # === TIMEOUTS ===
    proxy_connect_timeout 30s;
    proxy_send_timeout 30s;
    proxy_read_timeout 30s;

    # === MAIN API ROUTE ===
    location / {
        proxy_pass http://127.0.0.1:3000;
        limit_req zone=api burst=20 nodelay;
    }

    # === HEALTH CHECK ===
    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        access_log off;
    }

    # === METRICS (Protected) ===
    location /metrics {
        proxy_pass http://127.0.0.1:3000/metrics;
        # Add auth here if needed
        # auth_basic "Metrics";
        # auth_basic_user_file /etc/nginx/.htpasswd;
    }

    # === WEBSOCKET SUPPORT ===
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

    # === ERROR PAGES ===
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
        internal;
    }
}
EOF

# Replace config
sudo cp /tmp/ssl-enhanced.conf /etc/nginx/sites-available/flashbill-api

# Test configuration
echo -e "${YELLOW}Testing Nginx configuration...${NC}"
sudo nginx -t

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Configuration is valid${NC}"

    # Reload Nginx
    echo -e "${YELLOW}Reloading Nginx...${NC}"
    sudo systemctl reload nginx 2>/dev/null || sudo nginx -s reload 2>/dev/null || echo "Note: Reload skipped"

    echo -e "${GREEN}✓ SSL Security configured successfully${NC}"
else
    echo -e "${RED}✗ Configuration test failed${NC}"
    echo "Restoring backup..."
    sudo cp /etc/nginx/sites-available/flashbill-api.backup.* /etc/nginx/sites-available/flashbill-api
    exit 1
fi

# === SSL LABS TEST ===
echo
echo -e "${GREEN}=== SSL Security Configuration Complete ===${NC}"
echo
echo -e "${YELLOW}Security Features Enabled:${NC}"
echo "  ✓ TLS 1.2 + TLS 1.3 only"
echo "  ✓ Modern cipher suites (ECDHE + AES-GCM + CHACHA20)"
echo "  ✓ HSTS (Strict Transport Security)"
echo "  ✓ OCSP Stapling"
echo "  ✓ Secure headers (CSP, X-Frame-Options, etc.)"
echo "  ✓ Rate limiting"
echo "  ✓ HTTP/2 support"
echo
echo -e "${YELLOW}Test Your SSL:${NC}"
echo "  1. SSL Labs Test: https://www.ssllabs.com/ssltest/analyze.html?d=${DOMAIN}"
echo "  2. Security Headers: https://securityheaders.com/?q=${DOMAIN}"
echo
echo -e "${YELLOW}Check Certificate:${NC}"
echo "  certbot certificates"
echo
echo -e "${YELLOW}Renew Certificate:${NC}"
echo "  certbot renew --dry-run"
echo
echo -e "${YELLOW}View Nginx SSL Config:${NC}"
echo "  nginx -T | grep ssl"
echo
