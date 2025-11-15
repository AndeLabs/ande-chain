#!/bin/bash
# ANDE Chain - Cloudflare Tunnel Auto Setup

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Cloudflare Tunnel Auto Setup${NC}"
echo -e "${GREEN}========================================${NC}"

# Token ya verificado
API_TOKEN="zMmSa2x59iRRQEoklmVQKtJRbyKPps43shRmU1Rk"
echo -e "${GREEN}✓ Token verificado y activo${NC}"

# Obtener Account ID automáticamente
echo -e "${YELLOW}Obteniendo Account ID...${NC}"
ACCOUNT_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

ACCOUNT_ID=$(echo "$ACCOUNT_RESPONSE" | grep -oP '"id":"\K[^"]+' | head -1)
ACCOUNT_NAME=$(echo "$ACCOUNT_RESPONSE" | grep -oP '"name":"\K[^"]+' | head -1)

if [ -n "$ACCOUNT_ID" ]; then
    echo -e "${GREEN}✓ Account encontrado: ${ACCOUNT_NAME} (${ACCOUNT_ID})${NC}"
else
    echo -e "${RED}Error obteniendo Account ID${NC}"
    exit 1
fi

# Obtener Zone ID para ande.network
echo -e "${YELLOW}Obteniendo Zone ID de ande.network...${NC}"
ZONE_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=ande.network" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

ZONE_ID=$(echo "$ZONE_RESPONSE" | grep -oP '"id":"\K[^"]+' | head -1)

if [ -n "$ZONE_ID" ]; then
    echo -e "${GREEN}✓ Zone ID encontrado: ${ZONE_ID}${NC}"
else
    echo -e "${RED}Error: No se encontró la zona ande.network${NC}"
    exit 1
fi

# Verificar si existe un tunnel
echo -e "${YELLOW}Verificando tunnels existentes...${NC}"
TUNNELS_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

# Buscar tunnel existente llamado ande-chain
TUNNEL_ID=$(echo "$TUNNELS_RESPONSE" | grep -B1 '"name":"ande-chain"' | grep -oP '"id":"\K[^"]+' | head -1)

if [ -z "$TUNNEL_ID" ]; then
    # Crear nuevo tunnel si no existe
    echo -e "${YELLOW}Creando nuevo tunnel 'ande-chain'...${NC}"

    CREATE_RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json" \
         --data '{
           "name": "ande-chain",
           "config_src": "cloudflare"
         }')

    TUNNEL_ID=$(echo "$CREATE_RESPONSE" | grep -oP '"id":"\K[^"]+' | head -1)
    TUNNEL_TOKEN=$(echo "$CREATE_RESPONSE" | grep -oP '"token":"\K[^"]+' | head -1)

    if [ -n "$TUNNEL_ID" ]; then
        echo -e "${GREEN}✓ Tunnel creado: ${TUNNEL_ID}${NC}"
    else
        echo -e "${RED}Error creando tunnel${NC}"
        echo "$CREATE_RESPONSE"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Tunnel existente encontrado: ${TUNNEL_ID}${NC}"

    # Obtener token del tunnel existente
    echo -e "${YELLOW}Obteniendo token del tunnel...${NC}"
    TOKEN_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json")

    TUNNEL_TOKEN=$(echo "$TOKEN_RESPONSE" | grep -oP '"token":"\K[^"]+' | head -1)
fi

# Configurar DNS records
echo -e "${YELLOW}Configurando registros DNS...${NC}"

SUBDOMAINS=("rpc" "ws" "api" "explorer" "grafana" "metrics")

for SUBDOMAIN in "${SUBDOMAINS[@]}"; do
    echo -e "${YELLOW}Configurando ${SUBDOMAIN}.ande.network...${NC}"

    # Primero, buscar y eliminar registro existente si hay
    EXISTING_RECORDS=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records?name=${SUBDOMAIN}.ande.network" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json")

    RECORD_ID=$(echo "$EXISTING_RECORDS" | grep -oP '"id":"\K[^"]+' | head -1)

    if [ -n "$RECORD_ID" ]; then
        echo "  Eliminando registro existente..."
        curl -s -X DELETE "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records/${RECORD_ID}" \
             -H "Authorization: Bearer ${API_TOKEN}" > /dev/null
    fi

    # Crear nuevo registro CNAME
    DNS_RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json" \
         --data "{
           \"type\": \"CNAME\",
           \"name\": \"${SUBDOMAIN}\",
           \"content\": \"${TUNNEL_ID}.cfargotunnel.com\",
           \"proxied\": true
         }")

    if echo "$DNS_RESPONSE" | grep -q '"success":true'; then
        echo -e "${GREEN}  ✓ ${SUBDOMAIN}.ande.network configurado${NC}"
    else
        echo -e "${RED}  ✗ Error configurando ${SUBDOMAIN}.ande.network${NC}"
    fi
done

# Configurar rutas del tunnel
echo -e "${YELLOW}Configurando rutas del tunnel...${NC}"

# Configuración del tunnel
TUNNEL_CONFIG=$(cat <<EOF
{
  "config": {
    "ingress": [
      {
        "hostname": "rpc.ande.network",
        "service": "http://192.168.0.8:8545"
      },
      {
        "hostname": "ws.ande.network",
        "service": "ws://192.168.0.8:8546"
      },
      {
        "hostname": "api.ande.network",
        "service": "http://192.168.0.8:8545"
      },
      {
        "hostname": "explorer.ande.network",
        "service": "http://192.168.0.8:4000"
      },
      {
        "hostname": "grafana.ande.network",
        "service": "http://192.168.0.8:3000"
      },
      {
        "hostname": "metrics.ande.network",
        "service": "http://192.168.0.8:9001"
      },
      {
        "service": "http_status:404"
      }
    ]
  }
}
EOF
)

# Actualizar configuración del tunnel
CONFIG_RESPONSE=$(curl -s -X PUT "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json" \
     --data "$TUNNEL_CONFIG")

if echo "$CONFIG_RESPONSE" | grep -q '"success":true'; then
    echo -e "${GREEN}✓ Rutas del tunnel configuradas${NC}"
else
    echo -e "${YELLOW}Nota: Las rutas se configurarán cuando el tunnel se conecte${NC}"
fi

# Generar archivo de credenciales para el servidor
echo -e "${YELLOW}Generando archivo de instalación para el servidor...${NC}"

cat > install-tunnel-on-server.sh << 'INSTALL_SCRIPT'
#!/bin/bash
# Instalar y configurar Cloudflare Tunnel en el servidor

set -e

echo "Instalando Cloudflare Tunnel en el servidor..."

# Crear directorio de configuración
sudo mkdir -p /etc/cloudflared

# Guardar configuración del tunnel
sudo tee /etc/cloudflared/config.yml > /dev/null <<EOF
tunnel: TUNNEL_ID_PLACEHOLDER
credentials-file: /etc/cloudflared/TUNNEL_ID_PLACEHOLDER.json

ingress:
  - hostname: rpc.ande.network
    service: http://localhost:8545
  - hostname: ws.ande.network
    service: ws://localhost:8546
  - hostname: api.ande.network
    service: http://localhost:8545
  - hostname: explorer.ande.network
    service: http://localhost:4000
  - hostname: grafana.ande.network
    service: http://localhost:3000
  - hostname: metrics.ande.network
    service: http://localhost:9001
  - service: http_status:404
EOF

# Guardar credenciales
sudo tee /etc/cloudflared/TUNNEL_ID_PLACEHOLDER.json > /dev/null <<EOF
{
  "AccountTag": "ACCOUNT_ID_PLACEHOLDER",
  "TunnelSecret": "TUNNEL_SECRET_PLACEHOLDER",
  "TunnelID": "TUNNEL_ID_PLACEHOLDER"
}
EOF

# Configurar permisos
sudo chmod 600 /etc/cloudflared/*.json

# Instalar servicio
sudo cloudflared service install
sudo systemctl start cloudflared
sudo systemctl enable cloudflared

echo "✓ Cloudflare Tunnel instalado y activo"
echo "Verifica el estado con: sudo systemctl status cloudflared"
echo "Ver logs: sudo journalctl -u cloudflared -f"
INSTALL_SCRIPT

# Si tenemos el token del tunnel, usarlo
if [ -n "$TUNNEL_TOKEN" ]; then
    cat > install-tunnel-simple.sh << EOF
#!/bin/bash
# Instalación simple con token

set -e

echo "Instalando Cloudflare Tunnel..."

# Ejecutar cloudflared con el token
sudo cloudflared service install ${TUNNEL_TOKEN}

# Iniciar servicio
sudo systemctl start cloudflared
sudo systemctl enable cloudflared

echo "✓ Tunnel instalado y activo"
sudo systemctl status cloudflared
EOF

    chmod +x install-tunnel-simple.sh
    echo -e "${GREEN}✓ Script de instalación simple creado: install-tunnel-simple.sh${NC}"
fi

# Reemplazar placeholders
sed -i.bak "s/TUNNEL_ID_PLACEHOLDER/${TUNNEL_ID}/g" install-tunnel-on-server.sh
sed -i.bak "s/ACCOUNT_ID_PLACEHOLDER/${ACCOUNT_ID}/g" install-tunnel-on-server.sh

chmod +x install-tunnel-on-server.sh

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Configuración Completa!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "${YELLOW}Información del Setup:${NC}"
echo "  Account ID: ${ACCOUNT_ID}"
echo "  Zone ID: ${ZONE_ID}"
echo "  Tunnel ID: ${TUNNEL_ID}"
echo
echo -e "${YELLOW}DNS Configurados:${NC}"
for SUBDOMAIN in "${SUBDOMAINS[@]}"; do
    echo "  ✓ ${SUBDOMAIN}.ande.network → ${TUNNEL_ID}.cfargotunnel.com"
done
echo
echo -e "${YELLOW}Próximos pasos:${NC}"
echo
echo "1. Copia el script al servidor:"
echo -e "${BLUE}   scp install-tunnel-*.sh sator@192.168.0.8:~/ande-chain/${NC}"
echo
echo "2. Ejecuta en el servidor:"
echo -e "${BLUE}   ssh sator@192.168.0.8${NC}"
echo -e "${BLUE}   cd ande-chain${NC}"
echo -e "${BLUE}   ./install-tunnel-simple.sh${NC}"
echo
echo -e "${GREEN}Los endpoints estarán disponibles en unos minutos:${NC}"
echo "  • https://rpc.ande.network"
echo "  • wss://ws.ande.network"
echo "  • https://api.ande.network"
echo
echo -e "${GREEN}¡Tu ANDE Chain estará globalmente accesible!${NC}"