#!/bin/bash

# ============================================================================
# DEPLOY DO CONTRATO BRAZA TOKEN
# ============================================================================

set -e
source scripts/load-env.sh

log_info "=========================================="
log_info "  FAZENDO DEPLOY DO CONTRATO"
log_info "=========================================="

# Verificar se WASM existe
if [ ! -f ".soroban/braza_token.wasm" ]; then
    log_error "WASM não encontrado! Execute primeiro: ./scripts/build.sh"
    exit 1
fi

# Deploy do contrato
log_info "Fazendo deploy na rede $NETWORK..."

CONTRACT_ID=$(soroban contract deploy \
    --wasm .soroban/braza_token.wasm \
    --source $ADMIN_SECRET_KEY \
    --network $NETWORK)

if [ -z "$CONTRACT_ID" ]; then
    log_error "Falha no deploy!"
    exit 1
fi

# Salvar Contract ID
echo "$CONTRACT_ID" > .soroban/contract-id-$NETWORK.txt

log_success "Deploy concluído!"
log_info "Contract ID: $CONTRACT_ID"
log_info "Salvo em: .soroban/contract-id-$NETWORK.txt"
log_info "=========================================="
