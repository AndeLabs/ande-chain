# ANDE Chain Monitoring Infrastructure

Professional monitoring setup for ANDE Chain testnet deployment with multi-sequencer consensus.

## 📊 Components

### 1. Prometheus
- **Purpose**: Metrics collection and time-series database
- **Port**: 9093 (external), 9090 (internal)
- **Configuration**: `prometheus.yml`
- **Data Retention**: 30 days
- **Scrape Interval**: 15 seconds

**Monitored Services**:
- Sequencer Node 1 (primary)
- Sequencer Node 2 (backup)
- Sequencer Node 3 (backup)
- Celestia Light Node
- HAProxy Load Balancer
- Prometheus self-monitoring

### 2. Grafana
- **Purpose**: Metrics visualization and dashboards
- **Port**: 3001
- **Default Credentials**: admin / ande2024 (change in production!)
- **Auto-provisioned**:
  - Prometheus datasource
  - ANDE Chain Overview dashboard

**Pre-configured Dashboards**:
1. **ANDE Chain - Testnet Overview**
   - Current block height
   - Average block time
   - Transactions per second (TPS)
   - Sequencer health status
   - Block production timeline
   - Transaction throughput
   - Active validators
   - Transaction pool metrics
   - Gas usage
   - Sequencer rotations
   - Celestia DA submissions

### 3. HAProxy
- **Purpose**: Load balancing and high availability
- **Ports**:
  - 80: HTTP RPC load balancer
  - 443: HTTPS (requires SSL certificates)
  - 8404: Stats dashboard
  - 8546: WebSocket load balancer
  - 9101: Prometheus metrics
- **Configuration**: `haproxy.cfg`

**Features**:
- Round-robin load balancing across 3 sequencers
- Health checks every 10 seconds
- Automatic failover to backup sequencers
- Rate limiting (100 requests/10s per IP)
- CORS support for blockchain RPC
- WebSocket connection support
- Prometheus metrics export

### 4. Celestia Light Node
- **Purpose**: Data availability layer integration
- **Network**: Mocha-4 testnet
- **Ports**:
  - 26658: RPC endpoint
  - 26659: Gateway
  - 26658: Metrics (via /metrics)

---

## 🚀 Quick Start

### Start Monitoring Stack

```bash
cd /path/to/ande-chain
docker compose -f docker-compose-testnet.yml up -d prometheus grafana haproxy
```

### Access Dashboards

- **Prometheus**: http://localhost:9093
- **Grafana**: http://localhost:3001 (admin/ande2024)
- **HAProxy Stats**: http://localhost:8404/stats

### View Metrics

```bash
# Prometheus metrics from sequencer-1
curl http://localhost:9090/metrics | grep ande_

# HAProxy metrics
curl http://localhost:9101/metrics

# Celestia metrics
curl http://localhost:26658/metrics
```

---

## 📈 Key Metrics

### Chain Metrics
```promql
# Current block height
ande_chain_block_height

# Average block time
ande_consensus_block_time_seconds

# Transactions per second
rate(ande_evm_transactions_total[1m])
```

### Consensus Metrics
```promql
# Active validators
ande_consensus_active_validators

# Current proposer
ande_consensus_current_proposer

# Sequencer rotations
ande_consensus_rotation_count

# Timeout events
ande_consensus_timeout_events_total
```

### EVM Metrics
```promql
# Total transactions
ande_evm_transactions_total

# Gas used
rate(ande_evm_gas_used_total[5m])

# Transaction pool
ande_evm_txpool_pending
ande_evm_txpool_queued
```

### Celestia DA Metrics
```promql
# DA submissions
ande_celestia_da_submissions_total

# DA bytes submitted
ande_celestia_da_bytes_total

# DA submission latency
ande_celestia_da_latency_seconds
```

### System Metrics
```promql
# Sequencer health
up{job=~"sequencer.*"}

# HTTP request rate
rate(haproxy_frontend_http_requests_total[5m])

# Backend response time
haproxy_backend_response_time_average_seconds
```

---

## 🔧 Configuration

### Prometheus Configuration

Edit `monitoring/prometheus.yml` to:
- Add/remove scrape targets
- Adjust scrape intervals
- Configure alerting rules
- Add recording rules

Example:
```yaml
scrape_configs:
  - job_name: 'sequencer-4'
    static_configs:
      - targets: ['ande-sequencer-4:9090']
        labels:
          instance: 'sequencer-4'
          role: 'backup'
```

### Grafana Provisioning

**Datasources**: `grafana/provisioning/datasources/prometheus.yml`
**Dashboards**: `grafana/provisioning/dashboards/default.yml`
**Dashboard JSON**: `grafana/provisioning/dashboards/json/`

To add a new dashboard:
1. Create dashboard in Grafana UI
2. Export as JSON
3. Save to `grafana/provisioning/dashboards/json/`
4. Restart Grafana: `docker compose -f docker-compose-testnet.yml restart grafana`

### HAProxy Configuration

Edit `monitoring/haproxy.cfg` to:
- Add/remove backend servers
- Adjust health check intervals
- Configure rate limiting
- Enable HTTPS (requires SSL certificates)

**Enable HTTPS**:
1. Uncomment `frontend https_rpc` section
2. Add SSL certificate to `/etc/ssl/certs/ande-chain.pem`
3. Restart HAProxy

---

## 🔍 Monitoring Best Practices

### 1. Set Up Alerts

Create `prometheus/rules/alerts.yml`:
```yaml
groups:
  - name: ande_chain_alerts
    interval: 30s
    rules:
      - alert: SequencerDown
        expr: up{job=~"sequencer.*"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Sequencer {{ $labels.instance }} is down"

      - alert: HighBlockTime
        expr: ande_consensus_block_time_seconds > 5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Block time is high: {{ $value }}s"

      - alert: ConsensusNotRotating
        expr: changes(ande_consensus_current_proposer[10m]) == 0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Consensus has not rotated in 10 minutes"
```

### 2. Configure Alertmanager

Update `prometheus.yml`:
```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

Add Alertmanager to `docker-compose-testnet.yml`:
```yaml
alertmanager:
  image: prom/alertmanager:latest
  ports:
    - "9094:9093"
  volumes:
    - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml
```

### 3. Regular Monitoring Tasks

**Daily**:
- Check sequencer health: http://localhost:8404/stats
- Verify all sequencers are in sync
- Monitor disk space usage

**Weekly**:
- Review Grafana dashboards for anomalies
- Check Prometheus targets status
- Verify backup completion

**Monthly**:
- Review and archive old metrics
- Update dashboards based on new requirements
- Conduct performance testing

---

## 🛠 Troubleshooting

### Prometheus Not Scraping Targets

```bash
# Check Prometheus targets
curl http://localhost:9093/api/v1/targets | jq

# Check if sequencer metrics endpoint is accessible
curl http://localhost:9090/metrics

# View Prometheus logs
docker compose -f docker-compose-testnet.yml logs prometheus
```

### Grafana Dashboard Not Loading

```bash
# Check Grafana logs
docker compose -f docker-compose-testnet.yml logs grafana

# Verify Prometheus datasource
curl http://localhost:3001/api/datasources

# Reload provisioning
docker compose -f docker-compose-testnet.yml restart grafana
```

### HAProxy Not Load Balancing

```bash
# Check HAProxy stats
curl http://localhost:8404/stats

# View HAProxy logs
docker compose -f docker-compose-testnet.yml logs haproxy

# Test backend health
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

### High Memory Usage

```bash
# Check container memory usage
docker stats

# Reduce Prometheus retention
# Edit prometheus.yml: --storage.tsdb.retention.time=15d

# Reduce scrape frequency
# Edit prometheus.yml: scrape_interval: 30s
```

---

## 📚 Additional Resources

### Prometheus
- Query Language: https://prometheus.io/docs/prometheus/latest/querying/basics/
- Recording Rules: https://prometheus.io/docs/prometheus/latest/configuration/recording_rules/
- Alerting Rules: https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/

### Grafana
- Dashboard JSON Model: https://grafana.com/docs/grafana/latest/dashboards/json-model/
- Provisioning: https://grafana.com/docs/grafana/latest/administration/provisioning/
- Variables: https://grafana.com/docs/grafana/latest/dashboards/variables/

### HAProxy
- Configuration Manual: http://cbonte.github.io/haproxy-dconv/
- Health Checks: https://www.haproxy.com/documentation/hapee/latest/load-balancing/health-checking/active/
- Rate Limiting: https://www.haproxy.com/blog/four-examples-of-haproxy-rate-limiting/

---

## 🔐 Security Notes

1. **Change Default Credentials**: Update Grafana admin password
2. **Enable Authentication**: Configure HAProxy stats authentication
3. **Restrict Access**: Use firewall rules to limit access to monitoring ports
4. **HTTPS**: Enable SSL/TLS for production deployments
5. **API Keys**: Use API keys for programmatic access to Grafana/Prometheus

---

## 📝 Maintenance

### Backup Monitoring Data

```bash
# Backup Prometheus data
docker run --rm \
  -v ande-chain_prometheus-data:/prometheus \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/prometheus-$(date +%Y%m%d).tar.gz /prometheus

# Backup Grafana data
docker run --rm \
  -v ande-chain_grafana-data:/var/lib/grafana \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/grafana-$(date +%Y%m%d).tar.gz /var/lib/grafana
```

### Update Monitoring Stack

```bash
# Pull latest images
docker compose -f docker-compose-testnet.yml pull prometheus grafana haproxy

# Recreate containers
docker compose -f docker-compose-testnet.yml up -d --force-recreate prometheus grafana haproxy
```

---

**Last Updated**: 2024-11-15
**Version**: 1.0.0
**Maintainer**: ANDE Labs
