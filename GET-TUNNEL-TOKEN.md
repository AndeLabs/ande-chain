# Cómo Obtener el Token del Tunnel en Cloudflare

## Opción 1: Desde el Dashboard (Recomendado)

1. **Ve a Cloudflare Dashboard**
   - https://one.dash.cloudflare.com

2. **Navega a Zero Trust**
   - En el menú lateral, click en "Zero Trust"
   - O directo: https://one.dash.cloudflare.com/[tu-account-id]/networks/tunnels

3. **Encuentra tu Tunnel**
   - Busca el tunnel con ID: `5fced6cf-92eb-4167-abd3-d0b9397613cc`
   - O si no existe, crea uno nuevo llamado "ande-chain"

4. **Obtén el Token**

   **Si el tunnel YA existe:**
   - Click en el nombre del tunnel
   - Ve a la pestaña "Configure"
   - Busca el botón "Show Token" o "Rotate Token"
   - Copia el token (empieza con `eyJ...`)

   **Si NO existe el tunnel:**
   - Click en "Create a tunnel"
   - Nombre: `ande-chain`
   - Click "Save tunnel"
   - **IMPORTANTE**: En la siguiente pantalla te mostrará el token
   - Copia todo el token que empieza con `eyJ...`

## Opción 2: Usar tu API Token Existente

Ya tienes un token llamado "Cloudflare Tunnel API Token for ande.network" con los permisos correctos. Puedes usarlo para crear el tunnel:

```bash
# En tu Mac, exporta el token
export CLOUDFLARE_API_TOKEN="tu-api-token-aqui"

# Crea el tunnel
curl -X POST "https://api.cloudflare.com/client/v4/accounts/[ACCOUNT_ID]/tunnels" \
     -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
     -H "Content-Type: application/json" \
     --data '{
       "name": "ande-chain",
       "config_src": "local"
     }'
```

## Opción 3: Crear Nuevo Token de Tunnel

Si prefieres crear un token nuevo específicamente para el tunnel:

1. Ve a https://dash.cloudflare.com/profile/api-tokens
2. Click "Create Token"
3. Usa la plantilla "Cloudflare Tunnel"
4. Configura:
   - Account: Tu cuenta
   - Tunnel: All tunnels o específico
5. Click "Continue to summary" → "Create Token"
6. Copia el token

## Token vs Credenciales

### Token de Tunnel (lo que necesitas):
```
eyJhIjoiYTZjNz...muy-largo...ZiJ9
```

### Credenciales JSON (formato alternativo):
```json
{
  "AccountTag": "a6c7...",
  "TunnelSecret": "base64-encoded-secret",
  "TunnelID": "5fced6cf-92eb-4167-abd3-d0b9397613cc"
}
```

## Activar el Tunnel con el Token

Una vez que tengas el token, en el servidor:

```bash
ssh sator@192.168.0.8
cd ande-chain

# Opción A: Con el script interactivo
./activate-tunnel.sh
# Selecciona opción 1
# Pega el token cuando te lo pida

# Opción B: Directamente
sudo tee /etc/cloudflared/credentials.json > /dev/null <<EOF
{"TunnelToken": "eyJ...tu-token-aqui..."}
EOF

sudo cloudflared service install
sudo systemctl start cloudflared
sudo systemctl enable cloudflared
```

## Verificar que Funciona

```bash
# En el servidor
sudo systemctl status cloudflared

# Desde cualquier lugar (después de unos minutos)
curl https://rpc.ande.network
```

## Notas Importantes

- El token del tunnel es DIFERENTE al API token
- El tunnel token es específico para UN tunnel
- Si el tunnel con ID `5fced6cf-92eb-4167-abd3-d0b9397613cc` no existe, créalo primero
- El token es sensible, no lo compartas públicamente
- Una vez configurado, el tunnel se reconectará automáticamente

## Si Tienes Problemas

1. Verifica que el tunnel existe en el dashboard
2. Revisa los logs: `sudo journalctl -u cloudflared -f`
3. Asegúrate que el DNS apunta al tunnel en Cloudflare
4. Espera 5 minutos para propagación DNS