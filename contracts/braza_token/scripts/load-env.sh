#!/bin/bash

# ============================================================================
# CARREGAR VARIÁVEIS DE AMBIENTE
# ============================================================================

# Cores para logs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Funções de log
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Verificar se .env existe
if [ ! -f .env ]; then
    log_error "Arquivo .env não encontrado!"
    log_info "Crie um arquivo .env com as seguintes variáveis:"
    echo ""
    echo "STELLAR_NETWORK=testnet"
    echo "ADMIN_SECRET_KEY_TESTNET=SXXXXX..."
    echo "ADMIN_PUBLIC_KEY_TESTNET=GXXXXX..."
    echo 'TOKEN_NAME="Braza Token"'
    echo 'TOKEN_SYMBOL="BRZ"'
    echo ""
    exit 1
fi

# Carregar variáveis do .env (ignorando comentários e linhas vazias)
set -a  # Exportar automaticamente todas as variáveis
source <(grep -v '^[[:space:]]*#' .env | grep -v '^[[:space:]]*$' | sed 's/#.*$//' | sed 's/[[:space:]]*$//')
set +a

# Mapear variáveis do .env para formato esperado pelos scripts
export NETWORK="${STELLAR_NETWORK:-testnet}"

# Selecionar chaves baseado na rede
if [ "$NETWORK" = "testnet" ]; then
    export ADMIN_SECRET_KEY="${ADMIN_SECRET_KEY_TESTNET}"
    export ADMIN_ADDRESS="${ADMIN_PUBLIC_KEY_TESTNET}"
    export CONTRACT_ID="${CONTRACT_ID_TESTNET}"
    export WASM_HASH="${WASM_HASH_TESTNET}"
elif [ "$NETWORK" = "mainnet" ]; then
    export ADMIN_SECRET_KEY="${ADMIN_SECRET_KEY_MAINNET}"
    export ADMIN_ADDRESS="${ADMIN_PUBLIC_KEY_MAINNET}"
    export CONTRACT_ID="${CONTRACT_ID_MAINNET}"
    export WASM_HASH="${WASM_HASH_MAINNET}"
else
    log_error "STELLAR_NETWORK inválido: $NETWORK (use testnet ou mainnet)"
    exit 1
fi

# Validar variáveis obrigatórias
if [ -z "$NETWORK" ]; then
    log_error "STELLAR_NETWORK não definido no .env"
    exit 1
fi

if [ -z "$ADMIN_SECRET_KEY" ]; then
    log_error "ADMIN_SECRET_KEY_${NETWORK^^} não definido no .env"
    log_info "Valor atual: ADMIN_SECRET_KEY_TESTNET=$ADMIN_SECRET_KEY_TESTNET"
    exit 1
fi

if [ -z "$ADMIN_ADDRESS" ]; then
    log_error "ADMIN_PUBLIC_KEY_${NETWORK^^} não definido no .env"
    log_info "Valor atual: ADMIN_PUBLIC_KEY_TESTNET=$ADMIN_PUBLIC_KEY_TESTNET"
    exit 1
fi

if [ -z "$TOKEN_NAME" ]; then
    log_warning "TOKEN_NAME não definido, usando padrão: Braza Token"
    export TOKEN_NAME="Braza Token"
fi

if [ -z "$TOKEN_SYMBOL" ]; then
    log_warning "TOKEN_SYMBOL não definido, usando padrão: BRZ"
    export TOKEN_SYMBOL="BRZ"
fi

# Criar diretório .soroban se não existir
mkdir -p .soroban

log_success "Variáveis de ambiente carregadas com sucesso!"
log_info "Network: $NETWORK"
log_info "Admin: $ADMIN_ADDRESS"
log_info "Token: $TOKEN_NAME ($TOKEN_SYMBOL)"
