#!/bin/bash
# ANDE Chain - Cloudflare Tunnel Setup usando API Token

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Cloudflare Tunnel Setup${NC}"
echo -e "${GREEN}========================================${NC}"

# Solicitar información
echo -e "${YELLOW}Por favor proporciona la siguiente información:${NC}"
echo
echo -e "${BLUE}1. Ve a: https://dash.cloudflare.com/profile${NC}"
echo -e "${BLUE}   En la parte inferior derecha, copia tu Account ID${NC}"
read -p "Account ID: " ACCOUNT_ID
echo

echo -e "${BLUE}2. Copia tu API Token 'Cloudflare Tunnel API Token for ande.network'${NC}"
echo -e "${BLUE}   Dashboard → My Profile → API Tokens → View del token${NC}"
read -s -p "API Token: " API_TOKEN
echo
echo

# Verificar si el tunnel existe
echo -e "${YELLOW}Verificando tunnels existentes...${NC}"
TUNNEL_ID="5fced6cf-92eb-4167-abd3-d0b9397613cc"

# Obtener lista de tunnels
TUNNELS=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

# Verificar si nuestro tunnel existe
if echo "$TUNNELS" | grep -q "$TUNNEL_ID"; then
    echo -e "${GREEN}✓ Tunnel encontrado: ${TUNNEL_ID}${NC}"
    TUNNEL_EXISTS=true
else
    echo -e "${YELLOW}Tunnel no encontrado. Creando nuevo tunnel...${NC}"
    TUNNEL_EXISTS=false
fi

# Si el tunnel no existe, crear uno nuevo
if [ "$TUNNEL_EXISTS" = false ]; then
    echo -e "${YELLOW}Creando tunnel 'ande-chain'...${NC}"

    # Generar secret aleatorio
    TUNNEL_SECRET=$(openssl rand -base64 32)

    # Crear tunnel
    CREATE_RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json" \
         --data "{
           \"name\": \"ande-chain\",
           \"tunnel_secret\": \"${TUNNEL_SECRET}\"
         }")

    # Extraer nuevo tunnel ID
    NEW_TUNNEL_ID=$(echo "$CREATE_RESPONSE" | grep -oP '"id":"\K[^"]+' | head -1)

    if [ -n "$NEW_TUNNEL_ID" ]; then
        echo -e "${GREEN}✓ Tunnel creado: ${NEW_TUNNEL_ID}${NC}"
        TUNNEL_ID="$NEW_TUNNEL_ID"

        # Obtener el token del tunnel
        TUNNEL_TOKEN=$(echo "$CREATE_RESPONSE" | grep -oP '"token":"\K[^"]+' | head -1)
    else
        echo -e "${RED}Error creando tunnel${NC}"
        echo "$CREATE_RESPONSE"
        exit 1
    fi
fi

# Obtener token del tunnel existente (si no lo tenemos)
if [ -z "$TUNNEL_TOKEN" ]; then
    echo -e "${YELLOW}Obteniendo credenciales del tunnel...${NC}"

    # Para un tunnel existente, necesitamos obtener o rotar el token
    CREDS_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json")

    TUNNEL_TOKEN=$(echo "$CREDS_RESPONSE" | grep -oP '"token":"\K[^"]+' | head -1)

    if [ -z "$TUNNEL_TOKEN" ]; then
        echo -e "${YELLOW}No se pudo obtener token existente. Rotando token...${NC}"

        # Rotar token
        ROTATE_RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
             -H "Authorization: Bearer ${API_TOKEN}" \
             -H "Content-Type: application/json")

        TUNNEL_TOKEN=$(echo "$ROTATE_RESPONSE" | grep -oP '"token":"\K[^"]+' | head -1)
    fi
fi

if [ -z "$TUNNEL_TOKEN" ]; then
    echo -e "${RED}No se pudo obtener el token del tunnel${NC}"
    echo -e "${YELLOW}Por favor, obtén el token manualmente desde el dashboard:${NC}"
    echo "1. Ve a https://one.dash.cloudflare.com"
    echo "2. Zero Trust → Access → Tunnels"
    echo "3. Click en el tunnel 'ande-chain'"
    echo "4. Copia el token de instalación"
    exit 1
fi

echo -e "${GREEN}✓ Token obtenido${NC}"

# Crear rutas DNS
echo -e "${YELLOW}Configurando rutas DNS...${NC}"

ZONE_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=ande.network" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json" | grep -oP '"id":"\K[^"]+' | head -1)

if [ -n "$ZONE_ID" ]; then
    echo -e "${GREEN}✓ Zone ID encontrado: ${ZONE_ID}${NC}"

    # Crear registros DNS CNAME para el tunnel
    for SUBDOMAIN in rpc ws api explorer grafana metrics; do
        echo -e "${YELLOW}Creando DNS para ${SUBDOMAIN}.ande.network...${NC}"

        # Primero eliminar registro existente si hay
        curl -s -X DELETE "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records?name=${SUBDOMAIN}.ande.network" \
             -H "Authorization: Bearer ${API_TOKEN}" > /dev/null 2>&1

        # Crear nuevo registro CNAME
        curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records" \
             -H "Authorization: Bearer ${API_TOKEN}" \
             -H "Content-Type: application/json" \
             --data "{
               \"type\": \"CNAME\",
               \"name\": \"${SUBDOMAIN}\",
               \"content\": \"${TUNNEL_ID}.cfargotunnel.com\",
               \"proxied\": true
             }" > /dev/null

        echo -e "${GREEN}✓ ${SUBDOMAIN}.ande.network configurado${NC}"
    done
fi

# Guardar configuración para el servidor
echo -e "${YELLOW}Generando configuración para el servidor...${NC}"

cat > cloudflare-credentials.sh << EOF
#!/bin/bash
# Configuración del Tunnel de Cloudflare para ANDE Chain

# Crear directorio de configuración
sudo mkdir -p /etc/cloudflared

# Guardar token
sudo tee /etc/cloudflared/credentials.json > /dev/null <<'EOT'
{
  "TunnelToken": "${TUNNEL_TOKEN}"
}
EOT

# Configurar servicio
sudo cloudflared service install
sudo systemctl start cloudflared
sudo systemctl enable cloudflared

echo "✓ Cloudflare Tunnel configurado y activo"
echo "Verifica el estado con: sudo systemctl status cloudflared"
EOF

chmod +x cloudflare-credentials.sh

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✓ Configuración Completa!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "${YELLOW}Información del Tunnel:${NC}"
echo "Tunnel ID: ${TUNNEL_ID}"
echo "Account ID: ${ACCOUNT_ID}"
echo
echo -e "${YELLOW}Próximos pasos:${NC}"
echo "1. Copia cloudflare-credentials.sh al servidor:"
echo "   ${BLUE}scp cloudflare-credentials.sh sator@192.168.0.8:~/ande-chain/${NC}"
echo
echo "2. Ejecuta en el servidor:"
echo "   ${BLUE}ssh sator@192.168.0.8${NC}"
echo "   ${BLUE}cd ande-chain${NC}"
echo "   ${BLUE}./cloudflare-credentials.sh${NC}"
echo
echo -e "${YELLOW}Los endpoints estarán disponibles en:${NC}"
echo "  • https://rpc.ande.network"
echo "  • wss://ws.ande.network"
echo "  • https://api.ande.network"
echo "  • https://explorer.ande.network"
echo "  • https://grafana.ande.network"
echo
echo -e "${GREEN}El archivo cloudflare-credentials.sh contiene tu token.${NC}"
echo -e "${RED}¡Manténlo seguro y no lo compartas!${NC}"