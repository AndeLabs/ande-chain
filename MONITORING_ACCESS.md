# ANDE Chain Monitoring Access Guide

**Last Updated**: November 15, 2024
**Server**: 192.168.0.8
**Status**: ✅ All Monitoring Services Operational

---

## 🌐 Access URLs

### Grafana Dashboard
**URL**: http://192.168.0.8:3001

**Credentials**:
- Username: `admin`
- Password: `ande2024`

**Features**:
- Real-time blockchain metrics
- Custom dashboards for ANDE Chain
- Alert configuration
- Data exploration

### Prometheus Metrics
**URL**: http://192.168.0.8:9093

**Features**:
- Time-series metrics database
- PromQL query interface
- Target health monitoring
- Service discovery

---

## 📊 Available Dashboards

### 1. ANDE Chain Overview
Auto-provisioned dashboard with:
- Current block height
- Block production rate
- Transaction throughput
- Node health status
- Gas usage metrics

**Access**: Grafana → Dashboards → ANDE Chain Overview

### 2. Prometheus Targets
Monitor scraping targets health:
- ANDE Node metrics
- Celestia light node
- Prometheus self-monitoring

**Access**: Prometheus → Status → Targets

---

## 🔍 Key Metrics to Monitor

### Blockchain Health
```promql
# Current block height
ande_chain_block_height

# Block production rate
rate(ande_chain_block_height[5m])

# Transactions per second
rate(ande_evm_transactions_total[1m])
```

### Node Performance
```promql
# RPC request rate
rate(rpc_requests_total[5m])

# RPC latency
rpc_request_duration_seconds

# Memory usage
process_resident_memory_bytes
```

### System Resources
```promql
# CPU usage
process_cpu_seconds_total

# Open file descriptors
process_open_fds

# Network I/O
rate(node_network_receive_bytes_total[5m])
```

---

## 🎯 Quick Start Guide

### First Time Access

1. **Open Grafana**
   ```
   http://192.168.0.8:3001
   ```

2. **Login**
   - Username: `admin`
   - Password: `ande2024`
   - ⚠️ **Change password** on first login!

3. **Verify Datasource**
   - Go to: Configuration → Data Sources
   - Check: Prometheus datasource is connected
   - URL should be: `http://prometheus:9090`

4. **Access Dashboards**
   - Go to: Dashboards → Browse
   - Open: "ANDE Chain Overview"
   - Pin to favorites for quick access

### Prometheus Exploration

1. **Open Prometheus**
   ```
   http://192.168.0.8:9093
   ```

2. **Check Targets**
   - Go to: Status → Targets
   - Verify: `ande-node`, `prometheus`, `celestia` targets

3. **Run Queries**
   - Go to: Graph tab
   - Try query: `up`
   - Explore available metrics

---

## 📈 Creating Custom Dashboards

### Method 1: Grafana UI

1. Login to Grafana
2. Click "+" → Dashboard
3. Add Panel
4. Select metric from Prometheus
5. Configure visualization
6. Save dashboard

### Method 2: Import Dashboard

1. Go to: Dashboards → Import
2. Upload JSON file or paste dashboard ID
3. Select Prometheus datasource
4. Import

### Method 3: Provision via Code

1. Create dashboard JSON in:
   ```
   monitoring/grafana/provisioning/dashboards/json/
   ```

2. Restart Grafana:
   ```bash
   docker restart ande-grafana
   ```

---

## 🔔 Setting Up Alerts

### In Grafana

1. Open dashboard panel
2. Click Edit
3. Go to Alert tab
4. Create alert rule:
   ```
   Condition: WHEN last() OF query(A) IS ABOVE 100
   Evaluate every: 1m
   For: 5m
   ```

5. Configure notifications:
   - Email
   - Slack
   - PagerDuty
   - Webhook

### In Prometheus

Create alert rules in:
```yaml
# monitoring/prometheus-alerts.yml
groups:
  - name: ande_chain
    interval: 30s
    rules:
      - alert: NodeDown
        expr: up{job="ande-node"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "ANDE Node is down"

      - alert: HighBlockTime
        expr: ande_block_time_seconds > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Block time too high: {{ $value }}s"
```

---

## 🛠 Troubleshooting

### Grafana Not Accessible

```bash
# Check container status
docker ps | grep grafana

# View logs
docker logs ande-grafana

# Restart service
docker restart ande-grafana

# Verify port
curl http://192.168.0.8:3001/api/health
```

### Prometheus Not Scraping

```bash
# Check targets in Prometheus UI
# http://192.168.0.8:9093/targets

# Verify configuration
docker exec ande-prometheus cat /etc/prometheus/prometheus.yml

# Check logs
docker logs ande-prometheus

# Reload configuration
curl -X POST http://192.168.0.8:9093/-/reload
```

### No Data in Dashboards

1. **Check datasource connection**
   - Grafana → Configuration → Data Sources
   - Test connection to Prometheus

2. **Verify metrics exist**
   - Go to Prometheus: http://192.168.0.8:9093
   - Try query: `up`
   - Check if targets are UP

3. **Check time range**
   - Top-right corner in Grafana
   - Select "Last 5 minutes"

4. **Verify panel queries**
   - Edit panel
   - Check PromQL query syntax
   - Test in Prometheus first

---

## 📊 Recommended Dashboards

### Essential Monitoring

1. **Blockchain Overview**
   - Block height timeline
   - TPS graph
   - Gas usage
   - Node uptime

2. **Node Performance**
   - CPU usage
   - Memory consumption
   - Disk I/O
   - Network traffic

3. **Consensus Metrics** (when multi-sequencer deployed)
   - Active validators
   - Proposer rotations
   - Consensus rounds
   - Vote statistics

### Community Dashboards

Import from Grafana.com:
- Node Exporter Full: `1860`
- Prometheus 2.0 Overview: `3662`
- Docker Container Metrics: `193`

---

## 🔐 Security Best Practices

### Change Default Credentials

```bash
# First login, go to:
# Profile → Change Password

# Or via API:
curl -X PUT http://admin:ande2024@192.168.0.8:3001/api/user/password \
  -H "Content-Type: application/json" \
  -d '{"oldPassword":"ande2024","newPassword":"NEW_SECURE_PASSWORD"}'
```

### Enable HTTPS

1. Generate SSL certificate
2. Configure Grafana:
   ```ini
   [server]
   protocol = https
   cert_file = /path/to/cert.pem
   cert_key = /path/to/key.pem
   ```

3. Restart Grafana

### Restrict Access

1. **Firewall rules**:
   ```bash
   # Allow only from specific IPs
   sudo ufw allow from YOUR_IP to any port 3001
   sudo ufw allow from YOUR_IP to any port 9093
   ```

2. **Authentication**:
   - Enable GitHub OAuth
   - Use LDAP/AD
   - Configure SSO

---

## 📱 Mobile Access

### Grafana Mobile App

1. Download from App Store / Play Store
2. Add server: `http://192.168.0.8:3001`
3. Login with credentials
4. Access dashboards on mobile

### Responsive Web

- Grafana dashboards are mobile-responsive
- Access via mobile browser
- Same URL: http://192.168.0.8:3001

---

## 🔄 Backup & Recovery

### Backup Grafana Data

```bash
# Backup Grafana volume
docker run --rm \
  -v ande-chain_grafana-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/grafana-$(date +%Y%m%d).tar.gz /data
```

### Backup Prometheus Data

```bash
# Backup Prometheus volume
docker run --rm \
  -v ande-chain_prometheus-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/prometheus-$(date +%Y%m%d).tar.gz /data
```

### Restore from Backup

```bash
# Stop services
docker stop ande-grafana ande-prometheus

# Restore volume
docker run --rm \
  -v ande-chain_grafana-data:/data \
  -v $(pwd)/backups:/backup \
  alpine sh -c "cd /data && tar xzf /backup/grafana-YYYYMMDD.tar.gz --strip 1"

# Start services
docker start ande-prometheus ande-grafana
```

---

## 📚 Additional Resources

### Documentation
- Grafana Docs: https://grafana.com/docs/
- Prometheus Docs: https://prometheus.io/docs/
- PromQL Guide: https://prometheus.io/docs/prometheus/latest/querying/basics/

### Community
- Grafana Community: https://community.grafana.com/
- Prometheus Mailing List: https://groups.google.com/g/prometheus-users

### Tutorials
- Dashboard creation: https://grafana.com/tutorials/
- Alert configuration: https://grafana.com/docs/grafana/latest/alerting/
- PromQL examples: https://prometheus.io/docs/prometheus/latest/querying/examples/

---

## ✅ Quick Health Check

Run these commands to verify monitoring stack:

```bash
# Check Grafana health
curl http://192.168.0.8:3001/api/health

# Check Prometheus health
curl http://192.168.0.8:9093/-/healthy

# Check Prometheus targets
curl http://192.168.0.8:9093/api/v1/targets | jq

# Check Grafana datasources
curl http://admin:ande2024@192.168.0.8:3001/api/datasources | jq
```

Expected output: All services should return `ok` or `healthy`.

---

**Need Help?**
1. Check service logs: `docker logs <container-name>`
2. Review documentation in `/monitoring/README.md`
3. Visit Prometheus: http://192.168.0.8:9093
4. Visit Grafana: http://192.168.0.8:3001

---

**Last Updated**: November 15, 2024
**Version**: v1.0.0-testnet
**Status**: ✅ Operational
