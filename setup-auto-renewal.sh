#!/bin/bash

# Auto-Renewal Setup for Let's Encrypt
# Menjalankan renew setiap hari dan reload Nginx jika sertifikat diperbarui

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Auto-Renewal Setup ===${NC}"
echo

# Cek apakah certbot tersedia
if ! command -v certbot &> /dev/null; then
    echo -e "${RED}Certbot not found. Install dulu.${NC}"
    exit 1
fi

# Cek apakah cron tersedia
if ! command -v cron &> /dev/null; then
    echo -e "${YELLOW}Menginstal cron...${NC}"
    apt-get install -y cron
fi

# 1. Setup cron job untuk renew
echo -e "${YELLOW}Membuat cron job untuk auto-renewal...${NC}"

# Buat script renew khusus
cat > /usr/local/bin/flashbill-renew.sh << 'EOF'
#!/bin/bash

LOG_FILE="/var/log/flashbill-ssl-renew.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

echo "[$DATE] Starting SSL renewal check..." >> $LOG_FILE

# Renew certificate
certbot renew --quiet --deploy-hook "systemctl reload nginx" >> $LOG_FILE 2>&1

# Check if renewal was successful
if [ $? -eq 0 ]; then
    echo "[$DATE] Renewal check completed successfully" >> $LOG_FILE
else
    echo "[$DATE] Renewal check failed" >> $LOG_FILE
fi

echo "----------------------------------------" >> $LOG_FILE
EOF

chmod +x /usr/local/bin/flashbill-renew.sh

# Tambahkan ke cron (setiap hari jam 3 pagi)
(crontab -l 2>/dev/null | grep -v "flashbill-renew.sh"; echo "0 3 * * * /usr/local/bin/flashbill-renew.sh") | crontab -

echo -e "${GREEN}✓ Cron job added: Daily at 3 AM${NC}"

# 2. Setup systemd timer (alternatif untuk cron)
echo
echo -e "${YELLOW}Membuat systemd timer untuk auto-renewal...${NC}"

# Buat service file
sudo bash -c 'cat > /etc/systemd/system/flashbill-ssl-renew.service << "SERVICEEOF"
[Unit]
Description=FlashBill SSL Certificate Renewal
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/flashbill-renew.sh
SERVICEEOF'

# Buat timer file
sudo bash -c 'cat > /etc/systemd/system/flashbill-ssl-renew.timer << "TIMEREOF"
[Unit]
Description=FlashBill SSL Certificate Renewal Timer

[Timer]
OnCalendar=daily
Persistent=true
RandomizedDelaySec=3600

[Install]
WantedBy=timers.target
TIMEREOF'

# Enable timer jika systemd tersedia
if systemctl --version &>/dev/null; then
    systemctl daemon-reload
    systemctl enable flashbill-ssl-renew.timer
    systemctl start flashbill-ssl-renew.timer
    echo -e "${GREEN}✓ Systemd timer enabled${NC}"
else
    echo -e "${YELLOW}Note: Systemd not available, using cron only${NC}"
fi

# 3. Setup log rotation
echo
echo -e "${YELLOW}Membuat log rotation...${NC}"

sudo bash -c 'cat > /etc/logrotate.d/flashbill-ssl << "LOGROTATEEOF"
/var/log/flashbill-ssl-renew.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 root root
}
LOGROTATEEOF'

echo -e "${GREEN}✓ Log rotation configured${NC}"

# 4. Test renew
echo
echo -e "${YELLOW}Testing renewal (dry-run)...${NC}"
certbot renew --dry-run

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Renewal test successful${NC}"
else
    echo -e "${RED}✗ Renewal test failed${NC}"
    echo "Pastikan sertifikat sudah diinstal dengan benar"
fi

# 5. Status
echo
echo -e "${GREEN}=== Auto-Renewal Setup Complete ===${NC}"
echo
echo -e "${YELLOW}Cron Job:${NC}"
crontab -l | grep flashbill
echo
echo -e "${YELLOW}Log File:${NC} /var/log/flashbill-ssl-renew.log"
echo
echo -e "${YELLOW}Manual Test:${NC}"
echo "  certbot renew --dry-run"
echo
echo -e "${YELLOW}Check Status:${NC}"
echo "  certbot certificates"
echo
echo -e "${YELLOW}View Log:${NC}"
echo "  tail -f /var/log/flashbill-ssl-renew.log"
