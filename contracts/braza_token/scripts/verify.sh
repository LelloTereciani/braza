#!/bin/bash

# ============================================================================
# VERIFICAR DEPLOY DO CONTRATO
# ============================================================================

set -e
source scripts/load-env.sh

log_info "=========================================="
log_info "  VERIFICANDO DEPLOY"
log_info "=========================================="

# Verificar se Contract ID existe
if [ ! -f ".soroban/contract-id-$NETWORK.txt" ]; then
    log_error "Contract ID não encontrado!"
    log_info "Execute primeiro: ./scripts/deploy.sh"
    exit 1
fi

CONTRACT_ID=$(cat .soroban/contract-id-$NETWORK.txt)

log_info "Contract ID: $CONTRACT_ID"
log_info "Network: $NETWORK"
log_info "Admin: $ADMIN_ADDRESS"
log_info "=========================================="

# Verificar se contrato está inicializado
log_info "Verificando inicialização..."

# Obter nome do token
TOKEN_NAME_RESULT=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    name 2>/dev/null || echo "")

if [ -z "$TOKEN_NAME_RESULT" ]; then
    log_warning "Contrato ainda não inicializado"
    log_info "Execute: ./scripts/initialize.sh"
    exit 0
fi

log_success "Contrato inicializado!"

# Obter informações do token
log_info "=========================================="
log_info "  INFORMAÇÕES DO TOKEN"
log_info "=========================================="

NAME=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    name)

SYMBOL=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    symbol)

DECIMALS=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    decimals)

TOTAL_SUPPLY=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    total_supply)

ADMIN_BALANCE=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    balance \
    --id $ADMIN_ADDRESS)

ADMIN_CONTRACT=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    get_admin)

# ============================================================================
# CORREÇÃO: Remover aspas antes de calcular
# ============================================================================

TOTAL_SUPPLY_CLEAN=$(echo $TOTAL_SUPPLY | tr -d '"')
ADMIN_BALANCE_CLEAN=$(echo $ADMIN_BALANCE | tr -d '"')
ADMIN_CONTRACT_CLEAN=$(echo $ADMIN_CONTRACT | tr -d '"')

# Calcular valores em BRZ (com 7 decimais) usando awk
TOTAL_SUPPLY_BRZ=$(awk "BEGIN {printf \"%.2f\", $TOTAL_SUPPLY_CLEAN / 10000000}")
ADMIN_BALANCE_BRZ=$(awk "BEGIN {printf \"%.2f\", $ADMIN_BALANCE_CLEAN / 10000000}")

# ============================================================================

log_info "Nome:          $NAME"
log_info "Símbolo:       $SYMBOL"
log_info "Decimais:      $DECIMALS"
log_info "Total Supply:  $TOTAL_SUPPLY_CLEAN ($TOTAL_SUPPLY_BRZ BRZ)"
log_info "Admin:         $ADMIN_CONTRACT_CLEAN"
log_info "Balance Admin: $ADMIN_BALANCE_CLEAN ($ADMIN_BALANCE_BRZ BRZ)"

# Verificar se admin está correto
if [ "$ADMIN_CONTRACT_CLEAN" != "$ADMIN_ADDRESS" ]; then
    log_warning "Admin do contrato diferente do .env!"
    log_warning "Contrato: $ADMIN_CONTRACT_CLEAN"
    log_warning ".env:     $ADMIN_ADDRESS"
else
    log_success "Admin configurado corretamente!"
fi

# Verificar supply inicial
EXPECTED_INITIAL_SUPPLY=100000000000000
if [ "$TOTAL_SUPPLY_CLEAN" = "$EXPECTED_INITIAL_SUPPLY" ]; then
    log_success "Supply inicial correto: 10,000,000 BRZ"
else
    log_warning "Supply inicial: $TOTAL_SUPPLY_CLEAN (esperado: $EXPECTED_INITIAL_SUPPLY)"
fi

# Verificar se admin recebeu o mint inicial
if [ "$ADMIN_BALANCE_CLEAN" = "$TOTAL_SUPPLY_CLEAN" ]; then
    log_success "Admin recebeu todo o supply inicial!"
else
    log_warning "Balance do admin ($ADMIN_BALANCE_CLEAN) diferente do total supply ($TOTAL_SUPPLY_CLEAN)"
fi

log_info "=========================================="
log_success "Verificação concluída!"
log_info "=========================================="

# Mostrar links úteis
if [ "$NETWORK" = "testnet" ]; then
    log_info "🔗 Stellar Expert: https://stellar.expert/explorer/testnet/contract/$CONTRACT_ID"
    log_info "🔗 Stellar Laboratory: https://laboratory.stellar.org/#explorer?resource=contracts&endpoint=single&network=test"
elif [ "$NETWORK" = "mainnet" ]; then
    log_info "🔗 Stellar Expert: https://stellar.expert/explorer/public/contract/$CONTRACT_ID"
    log_info "🔗 Stellar Laboratory: https://laboratory.stellar.org/#explorer?resource=contracts&endpoint=single&network=public"
fi

log_info "=========================================="
