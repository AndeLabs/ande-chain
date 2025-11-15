#!/bin/bash
RPC="https://rpc.ande.network"

echo "==================================================="
echo "  AUDITORIA DE CONTRATOS - ANDECHAIN TESTNET"
echo "  Chain ID: 6174"
echo "==================================================="
echo ""

echo "Checking: ANDE Token Proxy"
echo "  Address: 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"
code=$(cast code 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 --rpc-url "$RPC" 2>/dev/null)
if [ "$code" != "0x" ] && [ -n "$code" ]; then
    echo "  Status: ✅ DEPLOYED"
else
    echo "  Status: ❌ NOT DEPLOYED"
fi
echo ""

echo "Checking: Staking Proxy"
echo "  Address: 0xa513E6E4b8f2a923D98304ec87F64353C4D5C853"
code=$(cast code 0xa513E6E4b8f2a923D98304ec87F64353C4D5C853 --rpc-url "$RPC" 2>/dev/null)
if [ "$code" != "0x" ] && [ -n "$code" ]; then
    echo "  Status: ✅ DEPLOYED"
else
    echo "  Status: ❌ NOT DEPLOYED"
fi
echo ""

echo "Checking: Timelock Proxy"
echo "  Address: 0x8A791620dd6260079BF849Dc5567aDC3F2FdC318"
code=$(cast code 0x8A791620dd6260079BF849Dc5567aDC3F2FdC318 --rpc-url "$RPC" 2>/dev/null)
if [ "$code" != "0x" ] && [ -n "$code" ]; then
    echo "  Status: ✅ DEPLOYED"
else
    echo "  Status: ❌ NOT DEPLOYED"
fi
echo ""

echo "Checking: Governor Proxy"
echo "  Address: 0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
code=$(cast code 0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e --rpc-url "$RPC" 2>/dev/null)
if [ "$code" != "0x" ] && [ -n "$code" ]; then
    echo "  Status: ✅ DEPLOYED"
else
    echo "  Status: ❌ NOT DEPLOYED"
fi
echo ""

echo "==================================================="
