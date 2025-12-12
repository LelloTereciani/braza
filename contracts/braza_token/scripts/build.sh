#!/bin/bash

# ============================================================================
# BUILD DO CONTRATO BRAZA TOKEN
# ============================================================================

set -e
source scripts/load-env.sh

log_info "=========================================="
log_info "  COMPILANDO BRAZA TOKEN"
log_info "=========================================="

# Criar diretório .soroban se não existir
mkdir -p .soroban

# Compilar contrato
log_info "Compilando contrato para WASM..."
cargo build --release --target wasm32-unknown-unknown

# Verificar se compilou
if [ ! -f "target/wasm32-unknown-unknown/release/braza_token.wasm" ]; then
    log_error "Falha na compilação!"
    exit 1
fi

log_success "Compilação concluída!"

# Otimizar WASM
log_info "Otimizando WASM..."
soroban contract optimize \
    --wasm target/wasm32-unknown-unknown/release/braza_token.wasm

# Verificar se otimizou
if [ ! -f "target/wasm32-unknown-unknown/release/braza_token.optimized.wasm" ]; then
    log_error "Falha na otimização!"
    exit 1
fi

# Copiar WASM otimizado para .soroban
cp target/wasm32-unknown-unknown/release/braza_token.optimized.wasm .soroban/braza_token.wasm

log_success "WASM otimizado copiado para .soroban/"

# Mostrar tamanhos
log_info "=========================================="
log_info "  TAMANHOS DOS ARQUIVOS"
log_info "=========================================="
ORIGINAL_SIZE=$(stat -f%z target/wasm32-unknown-unknown/release/braza_token.wasm 2>/dev/null || stat -c%s target/wasm32-unknown-unknown/release/braza_token.wasm)
OPTIMIZED_SIZE=$(stat -f%z target/wasm32-unknown-unknown/release/braza_token.optimized.wasm 2>/dev/null || stat -c%s target/wasm32-unknown-unknown/release/braza_token.optimized.wasm)
REDUCTION=$((100 - (OPTIMIZED_SIZE * 100 / ORIGINAL_SIZE)))

log_info "Original:   $(numfmt --to=iec-i --suffix=B $ORIGINAL_SIZE 2>/dev/null || echo "$ORIGINAL_SIZE bytes")"
log_info "Otimizado:  $(numfmt --to=iec-i --suffix=B $OPTIMIZED_SIZE 2>/dev/null || echo "$OPTIMIZED_SIZE bytes")"
log_success "Redução:    ${REDUCTION}%"
log_info "=========================================="

log_success "Build concluído com sucesso!"
