#!/bin/bash

# Manual SSL Setup - Untuk development/testing
# Menggunakan self-signed certificate atau setup manual

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Manual SSL Setup Guide ===${NC}"
echo

echo "Untuk production, Anda perlu:"
echo "1. Domain yang valid (contoh: api.flashbill.id)"
echo "2. DNS A record mengarah ke IP server"
echo "3. Port 80 dan 443 terbuka di firewall"
echo

echo -e "${YELLOW}Pilihan setup:${NC}"
echo "1. Self-signed certificate (development)"
echo "2. Let's Encrypt (production - butuh domain)"
echo "3. Manual setup dengan Cloudflare/SSL provider lain"
echo

read -p "Pilih [1/2/3]: " choice

case $choice in
    1)
        echo -e "${YELLOW}Membuat self-signed certificate...${NC}"
        sudo mkdir -p /etc/ssl/certs/self-signed

        sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout /etc/ssl/certs/self-signed/flashbill.key \
            -out /etc/ssl/certs/self-signed/flashbill.crt \
            -subj "/C=ID/ST=Jakarta/L=Jakarta/O=FlashBill/CN=localhost"

        echo -e "${GREEN}✓ Self-signed certificate created${NC}"
        echo "  Key: /etc/ssl/certs/self-signed/flashbill.key"
        echo "  Cert: /etc/ssl/certs/self-signed/flashbill.crt"

        # Update Nginx config untuk self-signed
        sudo sed -i 's|/etc/letsencrypt/live/.*/fullchain.pem|/etc/ssl/certs/self-signed/flashbill.crt|g' /etc/nginx/sites-available/flashbill-api
        sudo sed -i 's|/etc/letsencrypt/live/.*/privkey.pem|/etc/ssl/certs/self-signed/flashbill.key|g' /etc/nginx/sites-available/flashbill-api

        sudo nginx -t && sudo systemctl reload nginx 2>/dev/null || sudo nginx -s reload

        echo -e "${GREEN}✓ Nginx updated with self-signed cert${NC}"
        ;;

    2)
        echo -e "${YELLOW}Untuk Let's Encrypt, jalankan:${NC}"
        echo "  sudo /home/trunix/invoice/setup-https.sh yourdomain.com"
        echo
        echo "Contoh:"
        echo "  sudo /home/trunix/invoice/setup-https.sh api.flashbill.id"
        ;;

    3)
        echo -e "${YELLOW}Manual SSL Setup:${NC}"
        echo
        echo "1. Upload certificate Anda ke server:"
        echo "   - /etc/ssl/certs/yourdomain.crt"
        echo "   - /etc/ssl/private/yourdomain.key"
        echo
        echo "2. Update Nginx config:"
        echo "   ssl_certificate /etc/ssl/certs/yourdomain.crt;"
        echo "   ssl_certificate_key /etc/ssl/private/yourdomain.key;"
        echo
        echo "3. Reload Nginx:"
        echo "   sudo nginx -t && sudo systemctl reload nginx"
        ;;

    *)
        echo -e "${RED}Invalid choice${NC}"
        ;;
esac

echo
echo -e "${GREEN}=== Setup Complete ===${NC}"
