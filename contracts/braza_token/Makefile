# =====================================================================
# 🚀 BRAZA TOKEN - MAKEFILE OFICIAL (AUDIT EDITION 🛡️)
# =====================================================================

# Cores ANSI
GREEN  := \033[32m
YELLOW := \033[33m
BLUE   := \033[34m
CYAN   := \033[36m
MAG    := \033[35m
RED    := \033[31m
RESET  := \033[0m

CONTRACT_NAME := braza_token
TARGET := wasm32-unknown-unknown
WASM_DIR := target/$(TARGET)/release
WASM_FILE := $(WASM_DIR)/$(CONTRACT_NAME).wasm
OPTIMIZED_WASM := $(WASM_DIR)/$(CONTRACT_NAME).optimized.wasm

# 🌐 Rede
NETWORK := testnet

# 📂 Scripts
SCRIPTS := ./scripts

# =====================================================================
# 🔨 BUILD & OPTIMIZE
# =====================================================================

.PHONY: build
build:
	@echo "$(BLUE)🔨 Compilando contrato (Release)...$(RESET)"
	cargo build --release --target $(TARGET)
	@echo "$(GREEN)✔ Build concluído: $(WASM_FILE)$(RESET)"

.PHONY: optimize
optimize:
	@echo "$(MAG)✨ Otimizando WASM (Soroban Native)...$(RESET)"
	soroban contract optimize --wasm $(WASM_FILE)
	@echo "$(GREEN)✔ Otimização concluída: $(OPTIMIZED_WASM)$(RESET)"
	@ls -lh $(OPTIMIZED_WASM)

.PHONY: all
all: build optimize
	@echo "$(CYAN)🏁 Build + Otimização completos! Pronto para Deploy.$(RESET)"

# =====================================================================
# 🧪 TESTES (AUDITORIA)
# =====================================================================

.PHONY: test
test:
	@echo "$(MAG)🧪 Rodando Testes Unitários e de Integração...$(RESET)"
	cargo test
	@echo "$(GREEN)✔ Todos os testes passaram!$(RESET)"

.PHONY: test-fuzz
test-fuzz:
	@echo "$(RED)🔥 Rodando Testes de Estresse (Fuzzing)...$(RESET)"
	cargo test --test fuzz_proptest
	@echo "$(GREEN)✔ O contrato sobreviveu ao caos!$(RESET)"

.PHONY: audit
audit: test test-fuzz
	@echo "$(CYAN)🛡️  AUDITORIA COMPLETA: APROVADA$(RESET)"

# =====================================================================
# 🚀 DEPLOY
# =====================================================================

.PHONY: deploy
deploy: all
	@echo "$(CYAN)📦 Iniciando deploy na $(NETWORK)...$(RESET)"
	$(SCRIPTS)/deploy.sh

.PHONY: initialize
initialize:
	@echo "$(CYAN)🧭 Inicializando contrato...$(RESET)"
	$(SCRIPTS)/initialize.sh

.PHONY: verify
verify:
	@echo "$(CYAN)🔍 Verificando contrato...$(RESET)"
	$(SCRIPTS)/verify.sh

# =====================================================================
# 🎨 FORMAT / LINT
# =====================================================================

.PHONY: fmt
fmt:
	@echo "$(YELLOW)🎨 Formatando código...$(RESET)"
	cargo fmt
	@echo "$(GREEN)✔ Código formatado$(RESET)"

.PHONY: lint
lint:
	@echo "$(YELLOW)🧹 Rodando Clippy...$(RESET)"
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✔ Lint concluído sem warnings$(RESET)"

# =====================================================================
# 🛠️ UTILIDADES
# =====================================================================

.PHONY: clean
clean:
	@echo "$(YELLOW)🗑 Limpando diretório target...$(RESET)"
	cargo clean
	@echo "$(GREEN)✔ Limpeza concluída$(RESET)"

.PHONY: info
info:
	@echo "$(CYAN)📌 Informações do Projeto:$(RESET)"
	@echo "   • Contrato: $(CONTRACT_NAME)"
	@echo "   • Rede: $(NETWORK)"
	@echo "   • WASM Original: $(WASM_FILE)"
	@echo "   • WASM Otimizado: $(OPTIMIZED_WASM)"

# =====================================================================
# ❓ AJUDA
# =====================================================================

.PHONY: help
help:
	@echo ""
	@echo "$(GREEN)📘 BRAZA TOKEN - Comandos Disponíveis$(RESET)"
	@echo ""
	@echo "$(BLUE)🔨 make build$(RESET)      → Compila o contrato"
	@echo "$(BLUE)⚙ make optimize$(RESET)   → Otimiza o WASM (Gera .optimized.wasm)"
	@echo "$(BLUE)🚀 make all$(RESET)        → Build + Otimização (Recomendado)"
	@echo ""
	@echo "$(MAG)🧪 make test$(RESET)       → Roda testes padrão (Unitários/Integração)"
	@echo "$(RED)🔥 make test-fuzz$(RESET)  → Roda testes de estresse (Proptest)"
	@echo "$(CYAN)🛡️  make audit$(RESET)      → Roda TUDO (Testes + Fuzzing)"
	@echo ""
	@echo "$(CYAN)📦 make deploy$(RESET)     → Deploy do contrato otimizado"
	@echo ""
	@echo "$(YELLOW)🎨 make fmt$(RESET)       → Formatador Rust"
	@echo "$(YELLOW)🧹 make lint$(RESET)      → Clippy (Check de qualidade)"
	@echo "$(YELLOW)🗑 make clean$(RESET)     → Limpar target/"
	@echo ""
