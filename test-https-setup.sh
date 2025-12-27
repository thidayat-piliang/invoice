#!/bin/bash

# Test HTTPS Setup
# Verifikasi semua konfigurasi SSL

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== HTTPS Setup Verification ===${NC}"
echo

# 1. Check Nginx
echo -e "${YELLOW}[1/6] Checking Nginx installation...${NC}"
if command -v nginx &> /dev/null; then
    echo -e "${GREEN}✓ Nginx installed: $(nginx -v 2>&1)${NC}"
else
    echo -e "${RED}✗ Nginx not found${NC}"
fi

# 2. Check Certbot
echo
echo -e "${YELLOW}[2/6] Checking Certbot installation...${NC}"
if command -v certbot &> /dev/null; then
    echo -e "${GREEN}✓ Certbot installed: $(certbot --version 2>&1)${NC}"
else
    echo -e "${RED}✗ Certbot not found${NC}"
fi

# 3. Check Nginx config
echo
echo -e "${YELLOW}[3/6] Checking Nginx configuration...${NC}"
if sudo nginx -t 2>&1 | grep -q "successful"; then
    echo -e "${GREEN}✓ Nginx config is valid${NC}"
else
    echo -e "${RED}✗ Nginx config has errors${NC}"
    sudo nginx -t 2>&1
fi

# 4. Check SSL certificates
echo
echo -e "${YELLOW}[4/6] Checking SSL certificates...${NC}"
if [ -d "/etc/letsencrypt/live" ] && [ "$(ls -A /etc/letsencrypt/live 2>/dev/null)" ]; then
    echo -e "${GREEN}✓ SSL certificates found${NC}"
    certbot certificates 2>/dev/null || ls -la /etc/letsencrypt/live/
else
    echo -e "${YELLOW}⚠ No SSL certificates yet (run setup-https.sh)${NC}"
fi

# 5. Check cron job
echo
echo -e "${YELLOW}[5/6] Checking auto-renewal setup...${NC}"
if crontab -l 2>/dev/null | grep -q "certbot renew"; then
    echo -e "${GREEN}✓ Cron job configured${NC}"
    crontab -l | grep "certbot renew"
else
    echo -e "${YELLOW}⚠ No cron job found${NC}"
fi

# 6. Check security headers
echo
echo -e "${YELLOW}[6/6] Checking security configuration...${NC}"
if sudo grep -q "Strict-Transport-Security" /etc/nginx/sites-available/flashbill-api 2>/dev/null; then
    echo -e "${GREEN}✓ HSTS configured${NC}"
else
    echo -e "${YELLOW}⚠ HSTS not configured${NC}"
fi

if sudo grep -q "ssl_protocols TLSv1.2 TLSv1.3" /etc/nginx/sites-available/flashbill-api 2>/dev/null; then
    echo -e "${GREEN}✓ Modern TLS protocols configured${NC}"
else
    echo -e "${YELLOW}⚠ Modern TLS not configured${NC}"
fi

echo
echo -e "${GREEN}=== Summary ===${NC}"
echo

# Check if Rust API is running
echo -e "${YELLOW}Checking FlashBill API status...${NC}"
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${GREEN}✓ FlashBill API is running on port 3000${NC}"
else
    echo -e "${YELLOW}⚠ FlashBill API not responding on port 3000${NC}"
fi

# Check Nginx
echo
echo -e "${YELLOW}Checking Nginx status...${NC}"
if sudo nginx -t 2>&1 | grep -q "successful"; then
    echo -e "${GREEN}✓ Nginx is configured correctly${NC}"

    # Check if listening
    if sudo netstat -tlnp 2>/dev/null | grep -q ":80 " || sudo ss -tlnp 2>/dev/null | grep -q ":80 "; then
        echo -e "${GREEN}✓ Nginx listening on port 80${NC}"
    fi

    if sudo netstat -tlnp 2>/dev/null | grep -q ":443 " || sudo ss -tlnp 2>/dev/null | grep -q ":443 "; then
        echo -e "${GREEN}✓ Nginx listening on port 443${NC}"
    else
        echo -e "${YELLOW}⚠ Nginx not listening on port 443 (need SSL cert)${NC}"
    fi
fi

echo
echo -e "${GREEN}=== Verification Complete ===${NC}"
echo
echo -e "${YELLOW}Next Steps:${NC}"
echo
echo "1. ${GREEN}Production Setup:${NC}"
echo "   sudo /home/trunix/invoice/setup-https.sh yourdomain.com"
echo
echo "2. ${GREEN}Manual SSL (Self-signed for testing):${NC}"
echo "   sudo /home/trunix/invoice/manual-ssl-setup.sh"
echo
echo "3. ${GREEN}Setup Auto-Renewal:${NC}"
echo "   sudo /home/trunix/invoice/setup-auto-renewal.sh"
echo
echo "4. ${GREEN}Configure SSL Security:${NC}"
echo "   sudo /home/trunix/invoice/configure-ssl-security.sh yourdomain.com"
echo
echo -e "${YELLOW}Quick Commands:${NC}"
echo "  # Test SSL"
echo "  curl -vk https://localhost 2>&1 | grep -i 'ssl'"
echo
echo "  # Check certificates"
echo "  certbot certificates"
echo
echo "  # Test renewal"
echo "  certbot renew --dry-run"
echo
echo "  # View Nginx config"
echo "  nginx -T | grep -A 5 'server_name'"
echo
