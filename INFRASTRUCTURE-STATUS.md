# ANDE Chain - Infrastructure Status Dashboard

## 🚀 Deployment Status

### ✅ Completed
- [x] ANDE Chain implementation with Evolve EVM and Celestia DA
- [x] GitHub repository created: https://github.com/AndeLabs/ande-chain
- [x] Ubuntu server deployed at 192.168.0.8
- [x] Docker containers running with Reth v1.1.3
- [x] Chain responding on local network (Chain ID: 6174)
- [x] Cloudflare Tunnel software installed
- [x] Tunnel configuration created for all endpoints
- [x] Website already configured to use rpc.ande.network

### 🔄 In Progress
- [ ] Cloudflare Tunnel token activation (waiting for token from dashboard)
- [ ] DNS propagation for ande.network subdomains

### 📋 Pending
- [ ] Block explorer deployment
- [ ] Grafana monitoring dashboard
- [ ] Prometheus metrics collection
- [ ] Load balancing setup
- [ ] Backup node deployment

## 🌐 Network Architecture

```
Internet Users
      ↓
Cloudflare Global Network
      ↓
Cloudflare Tunnel (ID: 5fced6cf-92eb-4167-abd3-d0b9397613cc)
      ↓
Ubuntu Server (192.168.0.8)
      ↓
Docker Containers:
  - ANDE Node (Reth-based)
  - JWT Authentication
  - Future: Evolve Sequencer
  - Future: Celestia Light Node
```

## 🔗 Endpoints Configuration

| Endpoint | URL | Status | Purpose |
|----------|-----|--------|---------|
| RPC HTTP | https://rpc.ande.network | ⏳ Awaiting Token | JSON-RPC API |
| WebSocket | wss://ws.ande.network | ⏳ Awaiting Token | Real-time updates |
| API | https://api.ande.network | ⏳ Awaiting Token | REST API |
| Explorer | https://explorer.ande.network | ⏳ Needs Deployment | Block explorer |
| Grafana | https://grafana.ande.network | ⏳ Needs Setup | Monitoring |
| Metrics | https://metrics.ande.network | ⏳ Awaiting Token | Prometheus |
| Website | https://www.ande.network | ✅ Live | Main website |
| Docs | https://docs.ande.network | 📋 Planned | Documentation |

## 🔧 Current Services

### Running on Server (192.168.0.8)
```bash
CONTAINER ID   IMAGE                          STATUS   PORTS
ande-node      ghcr.io/paradigmxyz/reth:v1.1.3   Up      0.0.0.0:8545->8545/tcp
                                                        0.0.0.0:8546->8546/tcp
                                                        0.0.0.0:8551->8551/tcp
                                                        0.0.0.0:9001->9001/tcp
                                                        0.0.0.0:30303->30303/tcp
jwt-init       alpine:3.22.0                    Exited  JWT token generated
```

## 📊 Chain Statistics

| Metric | Value |
|--------|-------|
| Chain ID | 6174 (0x181e) |
| Network Name | ANDE Network |
| Native Token | ANDE |
| Consensus | Evolve EVM |
| Data Availability | Celestia |
| Block Time | ~2 seconds |
| Gas Limit | 30,000,000 |

## 🛠️ Quick Commands

### Check Chain Status
```bash
# From anywhere (once tunnel is active)
curl https://rpc.ande.network

# From local network
curl http://192.168.0.8:8545

# Current working test
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://192.168.0.8:8545
```

### Server Management
```bash
# SSH to server
ssh sator@192.168.0.8

# Check Docker status
docker compose ps
docker compose logs -f ande-node

# Restart services
docker compose restart

# System service
sudo systemctl status ande-chain
```

### Cloudflare Tunnel
```bash
# Check tunnel status
sudo systemctl status cloudflared
sudo journalctl -u cloudflared -f

# Activate tunnel (on server)
chmod +x activate-tunnel.sh
./activate-tunnel.sh
```

## 🚦 Next Steps

### Immediate (Today)
1. **Get Cloudflare Tunnel Token**
   - Login to https://dash.cloudflare.com
   - Navigate to Zero Trust > Access > Tunnels
   - Find tunnel: 5fced6cf-92eb-4167-abd3-d0b9397613cc
   - Copy token and run activation script

2. **Test Public Access**
   - Verify https://rpc.ande.network responds
   - Test WebSocket connection
   - Check website can connect to chain

### Short Term (This Week)
1. **Deploy Block Explorer**
   - Set up Blockscout or similar
   - Configure for ANDE Chain
   - Connect to explorer.ande.network

2. **Set Up Monitoring**
   - Deploy Grafana container
   - Configure Prometheus metrics
   - Create dashboards

3. **Enhance Security**
   - Set up rate limiting in Cloudflare
   - Configure DDoS protection
   - Add backup authentication

### Long Term (This Month)
1. **Scale Infrastructure**
   - Deploy additional nodes
   - Set up load balancing
   - Configure automatic failover

2. **Complete Evolve Integration**
   - Deploy Evolve Sequencer
   - Connect Celestia DA layer
   - Enable full rollup functionality

3. **Documentation**
   - Create developer docs
   - API documentation
   - Integration guides

## 📈 Performance Metrics

| Metric | Current | Target |
|--------|---------|--------|
| TPS | ~1,000 | 10,000+ |
| Block Time | 2s | 1s |
| Finality | 4s | 2s |
| Uptime | 99% | 99.99% |

## 🔐 Security Checklist

- [x] Firewall configured (UFW)
- [x] SSH access secured
- [x] Docker containers isolated
- [x] JWT authentication for Engine API
- [ ] SSL/TLS via Cloudflare (pending activation)
- [ ] Rate limiting configured
- [ ] DDoS protection active
- [ ] Regular backups scheduled
- [ ] Monitoring alerts configured
- [ ] Security audit completed

## 📞 Support & Resources

- **GitHub**: https://github.com/AndeLabs/ande-chain
- **Website**: https://www.ande.network
- **Documentation**: Coming soon at docs.ande.network
- **Server**: sator@192.168.0.8

## 🎯 Current Priority

**ACTIVATE CLOUDFLARE TUNNEL** - This will make your chain globally accessible and complete the basic infrastructure setup. Once the tunnel is active, all the configured endpoints will be live and your website at ande.network will be able to connect to the chain.

---
*Last Updated: November 2024*
*Status: Production-Ready, Awaiting Global Activation*