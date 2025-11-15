#!/bin/bash
# ANDE Chain - Activación Final del Cloudflare Tunnel

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Activación del Tunnel${NC}"
echo -e "${GREEN}========================================${NC}"

# Token del tunnel obtenido
TUNNEL_TOKEN="eyJhIjoiNThmOTBhZGM1NzFkMzFjNGI3YTg2MGI2ZWRlZjM0MDYiLCJ0IjoiNWZjZWQ2Y2YtOTJlYi00MTY3LWFiZDMtZDBiOTM5NzYxM2NjIiwicyI6ImVpV1hyeHhOYVBvdVdNWUQrUW5vekNVUmxDRWZUYTNPWTN5Vk5PclNBRGc9In0="

echo -e "${YELLOW}Instalando Cloudflare Tunnel con token...${NC}"

# Instalar el tunnel con el token
sudo cloudflared service install $TUNNEL_TOKEN

# Iniciar y habilitar el servicio
echo -e "${YELLOW}Iniciando servicio...${NC}"
sudo systemctl start cloudflared
sudo systemctl enable cloudflared

# Verificar estado
echo -e "${YELLOW}Verificando estado del tunnel...${NC}"
sleep 3
sudo systemctl status cloudflared --no-pager

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Tunnel Activado!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "${YELLOW}Los endpoints están disponibles en:${NC}"
echo "  • https://rpc.ande.network"
echo "  • wss://ws.ande.network"
echo "  • https://api.ande.network"
echo "  • https://explorer.ande.network"
echo "  • https://grafana.ande.network"
echo "  • https://metrics.ande.network"
echo
echo -e "${YELLOW}Comandos útiles:${NC}"
echo "  Ver logs: sudo journalctl -u cloudflared -f"
echo "  Estado: sudo systemctl status cloudflared"
echo "  Reiniciar: sudo systemctl restart cloudflared"
echo
echo -e "${GREEN}¡Tu ANDE Chain ahora es accesible globalmente!${NC}"