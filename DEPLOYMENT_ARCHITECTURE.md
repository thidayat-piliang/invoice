# FlashBill Deployment Architecture 🏗️

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PRODUCTION SERVER                            │
│                      (Ubuntu 24.04 LTS)                              │
└─────────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Nginx      │    │  PostgreSQL  │    │    Redis     │
│  (Reverse    │    │   Database   │    │    Cache     │
│   Proxy)     │    │              │    │              │
│              │    │              │    │              │
│ Port: 80/443 │    │ Port: 5432   │    │ Port: 6379   │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │
       │                   │                   │
       │                   │                   │
       ▼                   ▼                   ▼
┌──────────────────────────────────────────────────────────┐
│                                                          │
│              FlashBill API (Rust)                       │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │  • Axum Web Framework                         │    │
│  │  • JWT Authentication                         │    │
│  │  • PostgreSQL (SQLx)                          │    │
│  │  • Redis Cache                                │    │
│  │  • Email Queue                                │    │
│  │  • Payment Gateway (PayPal)                   │    │
│  │  • PDF Generation                             │    │
│  │  • Monitoring & Metrics                       │    │
│  └────────────────────────────────────────────────┘    │
│                                                          │
│  Port: 3000 (Internal)                                  │
│  Host: 127.0.0.1                                         │
└──────────────────────────────────────────────────────────┘
                              │
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Clients    │    │   Let's      │    │   External   │
│              │    │   Encrypt    │    │   Services   │
│              │    │              │    │              │
│ • Mobile App │    │ SSL Cert     │    │ • PayPal     │
│ • Web App    │    │ Auto Renew   │    │ • SMTP       │
│ • API Users  │    │              │    │ • Firebase   │
└──────────────┘    └──────────────┘    └──────────────┘
```

---

## 📊 **Traffic Flow**

### **Request Flow:**
```
User Request
    ↓
HTTPS (Port 443)
    ↓
Nginx (Reverse Proxy)
    ↓
HTTP (Port 3000)
    ↓
FlashBill API (Rust)
    ↓
PostgreSQL (Database)
    ↓
Redis (Cache)
    ↓
Response → User
```

### **Auto-Renewal Flow:**
```
Daily 3 AM
    ↓
Cron Job
    ↓
certbot renew
    ↓
Check Certificate
    ↓
If Renewed → Reload Nginx
    ↓
No Downtime!
```

---

## 🗂️ **File Structure**

```
/home/trunix/invoice/
├── backend/
│   ├── src/
│   ├── target/release/flashbill-api
│   ├── .env
│   └── Cargo.toml
├── frontend/
│   ├── lib/
│   ├── build/web/ (production)
│   └── pubspec.yaml
├── setup-https.sh
├── setup-auto-renewal.sh
├── configure-ssl-security.sh
├── manual-ssl-setup.sh
├── test-https-setup.sh
├── DEPLOYMENT.md
├── DEPLOYMENT_QUICK.md
└── HTTPS_SETUP.md

/etc/
├── nginx/
│   ├── nginx.conf
│   └── sites-available/
│       ├── flashbill-api (HTTPS)
│       └── flashbill-frontend (Frontend)
├── systemd/system/
│   └── flashbill-api.service
├── letsencrypt/
│   └── live/yourdomain.com/
│       ├── fullchain.pem
│       └── privkey.pem
└── ssl/
    └── certs/ (self-signed)

/var/
├── log/
│   ├── nginx/
│   └── flashbill-ssl-renew.log
└── www/
    ├── letsencrypt/ (ACME challenge)
    └── flashbill/ (frontend files)
```

---

## 🔐 **Security Layers**

```
┌─────────────────────────────────────┐
│  Layer 1: Network Firewall          │
│  • Port 80, 443 open               │
│  • Port 3000 internal only         │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Layer 2: Nginx                    │
│  • Rate limiting                   │
│  • SSL/TLS                         │
│  • Security headers                │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Layer 3: Application              │
│  • JWT Authentication              │
│  • Input validation                │
│  • SQL injection protection        │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Layer 4: Database                 │
│  • Password authentication         │
│  • Connection limits               │
└─────────────────────────────────────┘
```

---

## 📈 **Monitoring Stack**

```
┌─────────────────────────────────────┐
│  System Metrics                     │
│  • CPU, RAM, Disk                   │
│  • Network traffic                  │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Application Metrics                │
│  • API requests (Prometheus)       │
│  • Response times                   │
│  • Error rates                      │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Logs                               │
│  • Nginx access/error               │
│  • Backend logs (journalctl)        │
│  • SSL renewal logs                 │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│  Alerts                             │
│  • SSL expiry (< 30 days)          │
│  • Service down                     │
│  • High error rate                  │
└─────────────────────────────────────┘
```

---

## 🔄 **Update Flow**

### **Backend Update:**
```bash
git pull origin main
cargo build --release
sudo systemctl restart flashbill-api
```

### **Frontend Update:**
```bash
cd frontend
flutter pub get
flutter build web --release
sudo cp -r build/web /var/www/flashbill
sudo systemctl reload nginx
```

### **SSL Renewal (Automatic):**
```bash
# Daily at 3 AM
certbot renew --deploy-hook "systemctl reload nginx"
```

---

## 🎯 **Production Checklist**

### **Before Going Live:**
- [ ] Domain configured
- [ ] DNS A record set
- [ ] PostgreSQL running
- [ ] Redis running
- [ ] Backend built with `--release`
- [ ] Systemd service enabled
- [ ] Nginx configured
- [ ] SSL certificate installed
- [ ] Auto-renewal working
- [ ] Security headers enabled
- [ ] Rate limiting enabled
- [ ] Firewall configured
- [ ] Logs monitored
- [ ] Backup configured

### **After Deployment:**
- [ ] Test health endpoint
- [ ] Test authentication
- [ ] Test payment flow
- [ ] Check SSL Labs score
- [ ] Verify auto-renewal
- [ ] Monitor logs for 24h

---

## 📞 **Quick Reference**

| Service | Command |
|---------|---------|
| **Backend Status** | `sudo systemctl status flashbill-api` |
| **Backend Logs** | `sudo journalctl -u flashbill-api -f` |
| **Nginx Status** | `sudo systemctl status nginx` |
| **Nginx Reload** | `sudo systemctl reload nginx` |
| **SSL Renew** | `sudo certbot renew` |
| **SSL Check** | `certbot certificates` |
| **Test API** | `curl https://api.yourdomain.com/health` |

---

## 🚀 **Deployment Complete!**

**Your FlashBill is now production-ready with:**
- ✅ HTTPS with Let's Encrypt
- ✅ Auto-renewal (no manual work)
- ✅ Security headers (HSTS, CSP)
- ✅ Rate limiting
- ✅ Monitoring
- ✅ Zero-downtime updates

**URL:** `https://api.yourdomain.com`
