# Crear Token API de Cloudflare para ANDE Chain Tunnel

## Pasos para Crear el Token Correcto

### 1. Selecciona "Custom token"
En la página que estás viendo, click en **"Custom token"**

### 2. Configuración del Token

**Token name:**
```
ANDE Chain Tunnel Manager
```

### 3. Permissions (Permisos)

Añade estos permisos específicos:

#### Account Permissions:
- **Cloudflare Tunnel** → Edit
- **Access: Apps and Policies** → Edit
- **Access: Service Tokens** → Edit

#### Zone Permissions:
- **Zone** → ande.network → DNS → Edit
- **Zone** → ande.network → Zone Settings → Read

### 4. Account Resources
- **Include** → Your Account (selecciona tu cuenta)

### 5. Zone Resources
- **Include** → Specific zone → ande.network

### 6. Client IP Address Filtering (Opcional)
- Puedes dejarlo en blanco o añadir la IP de tu servidor: 192.168.0.8

### 7. TTL (Tiempo de vida)
- Start Date: Today
- End Date: No expiry (o 1 año si prefieres renovar)

### 8. Continue to Summary
Click en "Continue to summary"

### 9. Create Token
Revisa que tengas estos permisos:
- ✅ Account.Cloudflare Tunnel:Edit
- ✅ Zone.DNS:Edit para ande.network
- ✅ Zone.Zone Settings:Read para ande.network

Click en **"Create Token"**

### 10. ⚠️ IMPORTANTE: Copia el Token
**El token se mostrará UNA SOLA VEZ**
```
Ejemplo: V1.a7b3c9d2e5f8g1h4i6j8k0l2m4n6o8p0q2r4s6t8u0v2w4x6y8z0
```

## Usar el Token

Una vez que tengas el token, ejecuta este comando en tu Mac:

```bash
# Guarda el token en una variable
export CLOUDFLARE_API_TOKEN="tu-token-aqui"

# Verifica que funciona
curl -X GET "https://api.cloudflare.com/client/v4/user/tokens/verify" \
     -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
     -H "Content-Type: application/json"
```

Deberías ver:
```json
{
  "result": {"id":"...","status":"active"},
  "success": true
}
```

## Configurar el Tunnel con el Token

Ahora ejecuta el script que creé:

```bash
cd /Users/munay/dev/ande-labs/ande-chain
./cloudflare-tunnel-setup-api.sh
```

Te pedirá:
1. **Account ID**: Lo encuentras en el dashboard de Cloudflare (esquina inferior derecha)
2. **API Token**: Pega el token que acabas de crear

## Alternativa: Usar Global API Key (NO recomendado)

Si prefieres usar la Global API Key (menos seguro):
1. Ve a My Profile → API Tokens
2. View Global API Key
3. Usa tu email + Global API Key

Pero es MEJOR usar un token específico con permisos limitados.

## Permisos Mínimos Necesarios

Para que el tunnel funcione necesitas poder:
- ✅ Crear/editar tunnels en tu cuenta
- ✅ Modificar DNS de ande.network
- ✅ Leer configuración de la zona

NO necesitas:
- ❌ Acceso a billing
- ❌ Acceso a analytics
- ❌ Acceso a Workers
- ❌ Acceso a otras zonas

## Troubleshooting

Si el token no funciona, verifica que tenga:
1. Permisos de Cloudflare Tunnel a nivel de cuenta
2. Permisos de DNS Edit para la zona ande.network
3. Que no haya expirado
4. Que no tenga restricciones de IP si estás trabajando desde diferentes lugares

---
*Recuerda: El token es como una contraseña. Mantenlo seguro y no lo compartas públicamente.*