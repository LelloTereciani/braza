#!/bin/bash

# ============================================================================
# INICIALIZAR CONTRATO BRAZA TOKEN
# ============================================================================

set -e
source scripts/load-env.sh

log_info "=========================================="
log_info "  INICIALIZANDO CONTRATO"
log_info "=========================================="

# Verificar se Contract ID existe
if [ ! -f ".soroban/contract-id-$NETWORK.txt" ]; then
    log_error "Contract ID não encontrado!"
    log_info "Execute primeiro: ./scripts/deploy.sh"
    exit 1
fi

CONTRACT_ID=$(cat .soroban/contract-id-$NETWORK.txt)

log_info "Contract ID: $CONTRACT_ID"
log_info "Admin: $ADMIN_ADDRESS"
log_info "Token: $TOKEN_NAME ($TOKEN_SYMBOL)"
log_info "Mint Inicial: 10,000,000 BRZ"
log_info "=========================================="

# Inicializar contrato
log_info "Inicializando contrato..."

soroban contract invoke \
    --id $CONTRACT_ID \
    --source $ADMIN_SECRET_KEY \
    --network $NETWORK \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --name "$TOKEN_NAME" \
    --symbol "$TOKEN_SYMBOL"

log_success "Contrato inicializado com sucesso!"
log_info "=========================================="

# Verificar balance do admin
log_info "Verificando balance do admin..."

ADMIN_BALANCE=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    balance \
    --id $ADMIN_ADDRESS)

TOTAL_SUPPLY=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $ADMIN_ADDRESS \
    --network $NETWORK \
    -- \
    total_supply)

log_success "Balance Admin: $ADMIN_BALANCE ($(echo "scale=2; $ADMIN_BALANCE / 10000000" | bc) BRZ)"
log_success "Total Supply: $TOTAL_SUPPLY ($(echo "scale=2; $TOTAL_SUPPLY / 10000000" | bc) BRZ)"
log_info "=========================================="
