#!/bin/bash

# ============================================================================
# LIMPAR BUILDS E DEPLOYS
# ============================================================================

set -e
source scripts/load-env.sh

log_warning "=========================================="
log_warning "  LIMPEZA DE BUILDS E DEPLOYS"
log_warning "=========================================="
log_warning "Esta ação irá remover:"
log_warning "- Todos os builds (target/)"
log_warning "- Arquivos WASM compilados"
log_warning "- Contract IDs salvos"
log_warning "=========================================="

# Confirmar
read -p "Deseja continuar? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    log_info "Limpeza cancelada pelo usuário"
    exit 0
fi

log_info "Limpando builds do Cargo..."
cargo clean

log_info "Limpando arquivos WASM..."
rm -rf .soroban/*.wasm

log_info "Limpando Contract IDs..."
rm -rf .soroban/contract-id-*.txt

log_success "=========================================="
log_success "  LIMPEZA CONCLUÍDA!"
log_success "=========================================="
log_info "Próximos passos:"
log_info "1. Execute: ./scripts/build.sh"
log_info "2. Execute: ./scripts/deploy-all.sh"
log_success "=========================================="
