# ANDE Chain - Cloudflare Tunnel Setup Guide

## Current Status
✅ ANDE Chain is running on server: 192.168.0.8
✅ Cloudflare Tunnel software installed
✅ Configuration file created at `/etc/cloudflared/config.yml`
✅ Website configured to use `rpc.ande.network`

## Next Steps: Activate the Tunnel

### Option 1: Using Cloudflare Dashboard (Recommended)

1. **Login to Cloudflare Dashboard**
   - Go to https://dash.cloudflare.com
   - Navigate to Zero Trust > Access > Tunnels

2. **Find Your Tunnel**
   - Look for tunnel ID: `5fced6cf-92eb-4167-abd3-d0b9397613cc`
   - Or create a new tunnel if needed

3. **Get the Token**
   - Click on your tunnel
   - Go to "Configure" tab
   - Copy the token (it starts with `eyJ...`)

4. **Add Token to Server**
   ```bash
   # SSH into server
   ssh sator@192.168.0.8

   # Create credentials file
   sudo nano /etc/cloudflared/credentials.json

   # Paste this content (replace YOUR_TOKEN with actual token):
   {
     "AccountTag": "YOUR_ACCOUNT_ID",
     "TunnelSecret": "YOUR_TOKEN",
     "TunnelID": "5fced6cf-92eb-4167-abd3-d0b9397613cc"
   }

   # Save and exit (Ctrl+X, Y, Enter)
   ```

5. **Install and Start Service**
   ```bash
   # Install as system service
   sudo cloudflared service install

   # Start the tunnel
   sudo systemctl start cloudflared

   # Enable auto-start on boot
   sudo systemctl enable cloudflared

   # Check status
   sudo systemctl status cloudflared
   ```

### Option 2: Using Cloudflare CLI

1. **Login to Cloudflare**
   ```bash
   # SSH into server
   ssh sator@192.168.0.8

   # Login to Cloudflare
   cloudflared tunnel login
   ```

2. **List Tunnels**
   ```bash
   cloudflared tunnel list
   ```

3. **Route DNS**
   ```bash
   # Route all subdomains
   cloudflared tunnel route dns 5fced6cf-92eb-4167-abd3-d0b9397613cc rpc.ande.network
   cloudflared tunnel route dns 5fced6cf-92eb-4167-abd3-d0b9397613cc ws.ande.network
   cloudflared tunnel route dns 5fced6cf-92eb-4167-abd3-d0b9397613cc api.ande.network
   cloudflared tunnel route dns 5fced6cf-92eb-4167-abd3-d0b9397613cc explorer.ande.network
   cloudflared tunnel route dns 5fced6cf-92eb-4167-abd3-d0b9397613cc grafana.ande.network
   ```

4. **Run Tunnel**
   ```bash
   # Run with config
   cloudflared tunnel run --config /etc/cloudflared/config.yml 5fced6cf-92eb-4167-abd3-d0b9397613cc
   ```

## Testing the Setup

### 1. Test RPC Endpoint
```bash
# Test from anywhere
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  https://rpc.ande.network

# Expected result: {"jsonrpc":"2.0","id":1,"result":"0x181e"}
```

### 2. Test WebSocket
```javascript
// Test WebSocket connection
const ws = new WebSocket('wss://ws.ande.network');
ws.on('open', () => {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'eth_subscribe',
    params: ['newHeads'],
    id: 1
  }));
});
```

### 3. Test Block Explorer API
```bash
curl https://api.ande.network
```

## Monitoring

### Check Tunnel Status
```bash
# On server
sudo systemctl status cloudflared
sudo journalctl -u cloudflared -f
```

### Check Chain Status
```bash
# On server
docker compose ps
docker compose logs -f ande-node
```

## Troubleshooting

### Tunnel not connecting
1. Check credentials: `sudo cat /etc/cloudflared/credentials.json`
2. Check logs: `sudo journalctl -u cloudflared -n 50`
3. Restart service: `sudo systemctl restart cloudflared`

### RPC not responding
1. Check if chain is running: `docker compose ps`
2. Test locally: `curl http://localhost:8545`
3. Check firewall: `sudo ufw status`

### WebSocket issues
1. Ensure WebSocket support in Cloudflare dashboard
2. Check nginx/proxy settings if using reverse proxy
3. Test with wscat: `wscat -c wss://ws.ande.network`

## Current Endpoints Configuration

| Subdomain | Service | Port | Status |
|-----------|---------|------|--------|
| rpc.ande.network | JSON-RPC HTTP | 8545 | Configured |
| ws.ande.network | WebSocket RPC | 8546 | Configured |
| api.ande.network | Chain API | 8545 | Configured |
| explorer.ande.network | Block Explorer | 4000 | Ready (needs explorer) |
| grafana.ande.network | Monitoring | 3000 | Ready (needs Grafana) |
| metrics.ande.network | Prometheus | 9001 | Configured |

## Security Notes

- The tunnel provides automatic SSL/TLS encryption
- No need to expose ports directly to internet
- All traffic goes through Cloudflare's network
- DDoS protection included
- Rate limiting can be configured in Cloudflare dashboard

## Next Development Steps

1. ✅ Chain running on server
2. ✅ Cloudflare Tunnel installed
3. ⏳ Add tunnel credentials
4. ⏳ Start tunnel service
5. ⏳ Test public endpoints
6. ⏳ Deploy block explorer
7. ⏳ Set up monitoring dashboards
8. ⏳ Configure rate limiting
9. ⏳ Add backup nodes

## Support

- ANDE Chain GitHub: https://github.com/AndeLabs/ande-chain
- Website: https://www.ande.network
- Documentation: Coming soon at docs.ande.network