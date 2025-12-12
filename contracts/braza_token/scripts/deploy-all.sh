#!/bin/bash

# ============================================================================
# BRAZA TOKEN - DEPLOY COMPLETO (ALL-IN-ONE)
# Mint automático de 10M BRZ no initialize
# ============================================================================

set -e

# Carregar variáveis de ambiente
source scripts/load-env.sh

log_info "=========================================="
log_info "  BRAZA TOKEN - DEPLOY COMPLETO"
log_info "=========================================="
log_info "Network: $NETWORK"
log_info "Admin: $ADMIN_ADDRESS"
log_info "Mint Automático: 10,000,000 BRZ"
log_info "=========================================="

# Etapa 1: Build
log_info "Etapa 1/4: Compilando contrato..."
bash scripts/build.sh

# Etapa 2: Deploy
log_info "Etapa 2/4: Fazendo deploy..."
bash scripts/deploy.sh

# Etapa 3: Initialize (com mint automático de 10M BRZ)
log_info "Etapa 3/4: Inicializando contrato (mint automático)..."
bash scripts/initialize.sh

# Etapa 4: Verificação
log_info "Etapa 4/4: Verificando deploy..."
bash scripts/verify.sh

log_success "=========================================="
log_success "  DEPLOY COMPLETO COM SUCESSO!"
log_success "=========================================="
log_info "Contract ID: $(cat .soroban/contract-id-$NETWORK.txt)"
log_info "Admin: $ADMIN_ADDRESS"
log_info "Initial Supply: 10,000,000 BRZ (mintado automaticamente)"
log_info "Max Supply: 21,000,000 BRZ"
log_info "Remaining Supply: 11,000,000 BRZ"
log_success "=========================================="
log_info ""
log_info "Próximos passos:"
log_info "1. Verificar o contrato no Stellar Explorer"
log_info "2. Testar transferências"
log_info "3. Configurar compliance (KYC, blacklist)"
log_info "4. Preparar para auditoria externa"
log_success "=========================================="
