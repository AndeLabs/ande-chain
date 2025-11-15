#!/bin/bash
# Quick setup script with your new Cloudflare token

echo "Configurando Cloudflare Tunnel para ANDE Chain..."
echo

# Solicitar el token
echo "Pega el token de Cloudflare que acabas de crear:"
read -s CLOUDFLARE_TOKEN
echo

# Solicitar Account ID
echo "Tu Account ID de Cloudflare (lo encuentras en el dashboard, esquina inferior derecha):"
read ACCOUNT_ID
echo

# Verificar que el token funciona
echo "Verificando token..."
VERIFY=$(curl -s -X GET "https://api.cloudflare.com/client/v4/user/tokens/verify" \
     -H "Authorization: Bearer $CLOUDFLARE_TOKEN" \
     -H "Content-Type: application/json")

if echo "$VERIFY" | grep -q '"success":true'; then
    echo "✓ Token válido!"
else
    echo "✗ Error: Token inválido"
    echo "$VERIFY"
    exit 1
fi

# Guardar configuración
cat > cloudflare-config.env << EOF
CLOUDFLARE_TOKEN="$CLOUDFLARE_TOKEN"
ACCOUNT_ID="$ACCOUNT_ID"
TUNNEL_ID="5fced6cf-92eb-4167-abd3-d0b9397613cc"
EOF

echo "✓ Configuración guardada en cloudflare-config.env"
echo
echo "Próximo paso: Ejecuta el script completo de configuración:"
echo "./cloudflare-tunnel-setup-api.sh"
echo
echo "El script usará automáticamente tu token para configurar todo."