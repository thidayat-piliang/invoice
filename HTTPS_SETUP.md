# FlashBill HTTPS Setup Guide

Lengkapkan FlashBill API dengan HTTPS menggunakan Let's Encrypt SSL dengan auto-renewal.

## Prasyarat

1. **Domain yang valid** (contoh: `api.flashbill.id`)
2. **DNS A Record** mengarah ke IP server
3. **Port 80 dan 443** terbuka di firewall
4. **Server dengan root access**

## Instalasi Cepat

### 1. Install Nginx & Certbot

```bash
sudo apt-get update
sudo apt-get install -y nginx certbot python3-certbot-nginx
```

### 2. Setup HTTPS dengan Auto-Renewal

```bash
# Ganti 'yourdomain.com' dengan domain Anda
sudo /home/trunix/invoice/setup-https.sh yourdomain.com
```

**Contoh:**
```bash
sudo /home/trunix/invoice/setup-https.sh api.flashbill.id
```

### 3. Setup Auto-Renewal Manual (Opsional)

```bash
sudo /home/trunix/invoice/setup-auto-renewal.sh
```

### 4. Konfigurasi SSL Security

```bash
sudo /home/trunix/invoice/configure-ssl-security.sh yourdomain.com
```

### 5. Verifikasi Setup

```bash
/home/trunix/invoice/test-https-setup.sh
```

## Script yang Tersedia

| Script | Fungsi |
|--------|--------|
| `setup-https.sh` | Setup lengkap HTTPS + SSL certificate |
| `setup-auto-renewal.sh` | Setup auto-renewal cron job |
| `configure-ssl-security.sh` | Konfigurasi security headers & cipher |
| `manual-ssl-setup.sh` | Self-signed cert untuk development |
| `test-https-setup.sh` | Verifikasi semua konfigurasi |

## Manual Setup

### Opsi 1: Self-Signed Certificate (Development)

```bash
sudo /home/trunix/invoice/manual-ssl-setup.sh
# Pilih opsi 1
```

### Opsi 2: Let's Encrypt (Production)

```bash
# Pastikan domain sudah diarahkan ke server
sudo certbot certonly --standalone \
    --preferred-challenges http \
    --agree-tos \
    --email admin@yourdomain.com \
    -d yourdomain.com \
    -d www.yourdomain.com
```

### Opsi 3: Cloudflare SSL

Jika menggunakan Cloudflare:
1. Upload certificate ke `/etc/ssl/certs/` dan `/etc/ssl/private/`
2. Update Nginx config:
```nginx
ssl_certificate /etc/ssl/certs/yourdomain.crt;
ssl_certificate_key /etc/ssl/private/yourdomain.key;
```

## Auto-Renewal

### Cron Job (Rekomendasi)

Cron job sudah diatur untuk berjalan setiap hari jam 3 pagi:

```bash
# Cek cron job
crontab -l

# Manual renew test
certbot renew --dry-run
```

### Systemd Timer (Alternatif)

```bash
# Cek status
systemctl status flashbill-ssl-renew.timer

# Manual run
systemctl start flashbill-ssl-renew
```

## SSL Security Features

### Protokol & Cipher
- ✅ TLS 1.2 + TLS 1.3 only
- ✅ ECDHE + AES-GCM + CHACHA20
- ✅ OCSP Stapling
- ✅ SSL Session Cache

### Security Headers
- ✅ HSTS (Strict Transport Security)
- ✅ X-Frame-Options
- ✅ X-Content-Type-Options
- ✅ X-XSS-Protection
- ✅ Content-Security-Policy
- ✅ Referrer-Policy
- ✅ Permissions-Policy

### Rate Limiting
- ✅ 10 requests/second per IP
- ✅ Burst 20 requests

### File Upload
- ✅ Max 100MB

## Testing

### Test SSL Certificate
```bash
# Check certificate info
certbot certificates

# Test renewal
certbot renew --dry-run

# View certificate details
openssl x509 -in /etc/letsencrypt/live/yourdomain.com/fullchain.pem -text -noout
```

### Test HTTPS Connection
```bash
# Test with curl
curl -I https://yourdomain.com/health

# Full SSL test
curl -vk https://yourdomain.com 2>&1 | grep -i 'ssl'

# Test security headers
curl -I https://yourdomain.com | grep -i 'strict-transport-security'
```

### Online Testing Tools
- **SSL Labs**: https://www.ssllabs.com/ssltest/analyze.html?d=yourdomain.com
- **Security Headers**: https://securityheaders.com/?q=yourdomain.com
- **SSL Checker**: https://www.sslshopper.com/ssl-checker.html

## Troubleshooting

### Port 80/443 Not Open
```bash
# Check open ports
sudo netstat -tlnp | grep -E ':(80|443)'

# Open ports (UFW)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

### Certificate Not Found
```bash
# Regenerate certificate
sudo certbot certonly --standalone -d yourdomain.com

# Check live directory
ls -la /etc/letsencrypt/live/
```

### Nginx Not Reloading
```bash
# Test config
sudo nginx -t

# View errors
sudo nginx -T | grep error
sudo tail -f /var/log/nginx/error.log
```

### Auto-Renewal Not Working
```bash
# Test renewal manually
sudo certbot renew --dry-run

# Check cron logs
sudo grep CRON /var/log/syslog

# View renewal log
tail -f /var/log/flashbill-ssl-renew.log
```

### HSTS Not Working
```bash
# Check Nginx config
sudo grep "Strict-Transport-Security" /etc/nginx/sites-available/flashbill-api

# Clear HSTS cache (browser)
# Chrome: chrome://net-internals/#hsts
```

## Maintenance

### Renew Certificate Manually
```bash
sudo certbot renew
sudo systemctl reload nginx
```

### View Certificate Expiry
```bash
sudo openssl x509 -enddate -noout -in /etc/letsencrypt/live/yourdomain.com/fullchain.pem
```

### Check Renewal Logs
```bash
tail -f /var/log/flashbill-ssl-renew.log
```

### Update Certificate Paths
Jika domain berubah:
```bash
sudo /home/trunix/invoice/configure-ssl-security.sh newdomain.com
```

## Security Best Practices

### 1. Always Use HTTPS
- Redirect all HTTP to HTTPS
- Use HSTS with preload

### 2. Keep Certificates Updated
- Auto-renewal enabled
- Check expiry monthly

### 3. Monitor Logs
```bash
# Nginx access log
tail -f /var/log/nginx/access.log

# Nginx error log
tail -f /var/log/nginx/error.log

# SSL renewal log
tail -f /var/log/flashbill-ssl-renew.log
```

### 4. Backup Certificates
```bash
# Backup certificates
sudo tar -czf /backup/ssl-certs-$(date +%Y%m%d).tar.gz /etc/letsencrypt/

# Restore
sudo tar -xzf /backup/ssl-certs-YYYYMMDD.tar.gz -C /
```

### 5. Monitor SSL Labs Score
Aim for **A+ rating**:
- Disable TLS 1.0, 1.1
- Use strong cipher suites
- Enable HSTS
- Enable OCSP Stapling

## Production Checklist

- [ ] Domain configured and pointing to server
- [ ] Port 80 and 443 open
- [ ] SSL certificate generated
- [ ] Nginx configured with HTTPS
- [ ] Auto-renewal tested
- [ ] Security headers configured
- [ ] HSTS enabled
- [ ] Rate limiting enabled
- [ ] SSL Labs test passed (A+)
- [ ] Security headers test passed
- [ ] Monitoring configured
- [ ] Backup strategy in place

## Support

For issues:
1. Run test script: `/home/trunix/invoice/test-https-setup.sh`
2. Check logs: `tail -f /var/log/nginx/error.log`
3. Test renewal: `certbot renew --dry-run`
4. Verify config: `nginx -t`

---

**Note**: This setup is production-ready and follows security best practices for SSL/TLS configuration.
