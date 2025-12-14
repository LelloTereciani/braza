# =====================================================================
# 🚀 BRAZA TOKEN - MAKEFILE OFICIAL (COM CORES 🎨)
# =====================================================================

# Cores ANSI
GREEN  := \033[32m
YELLOW := \033[33m
BLUE   := \033[34m
CYAN   := \033[36m
MAG	:= \033[35m
RED	:= \033[31m
RESET  := \033[0m

CONTRACT_NAME := braza_token
TARGET := wasm32-unknown-unknown
WASM_PATH := target/$(TARGET)/release/$(CONTRACT_NAME).wasm

# 🌐 Rede
NETWORK := testnet

# 📂 Scripts
SCRIPTS := ./scripts

# =====================================================================
# 🔨 BUILD
# =====================================================================

.PHONY: build
build:
	@echo "$(BLUE)🔨 Compilando contrato...$(RESET)"
	cargo build --release --target $(TARGET)
	@echo "$(GREEN)✔ Build concluído: $(WASM_PATH)$(RESET)"

.PHONY: optimize
optimize:
	@echo "$(MAG)✨ Otimizando WASM...$(RESET)"
	wasm-opt -Oz $(WASM_PATH) -o $(WASM_PATH)
	@echo "$(GREEN)✔ Otimização concluída!$(RESET)"

.PHONY: build-opt
build-opt: build optimize
	@echo "$(CYAN)🏁 Build + Otimização completos!$(RESET)"

# =====================================================================
# 🚀 DEPLOY
# =====================================================================

.PHONY: deploy
deploy: build-opt
	@echo "$(CYAN)📦 Iniciando deploy...$(RESET)"
	$(SCRIPTS)/deploy.sh

.PHONY: initialize
initialize:
	@echo "$(CYAN)🧭 Inicializando contrato...$(RESET)"
	$(SCRIPTS)/initialize.sh

.PHONY: verify
verify:
	@echo "$(CYAN)🔍 Verificando contrato...$(RESET)"
	$(SCRIPTS)/verify.sh

.PHONY: deploy-all
deploy-all:
	@echo "$(CYAN)🚀 Executando pipeline completo: build → deploy → init → verify$(RESET)"
	$(SCRIPTS)/deploy-all.sh

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
# 🧪 TESTES
# =====================================================================

.PHONY: test
test:
	@echo "$(RED)⚠ Testes ainda serão implementados.$(RESET)"
	# cargo test

.PHONY: test-int
test-int:
	@echo "$(RED)⚠ Testes de integração ainda serão implementados.$(RESET)"
	# cargo test --test '*'

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
	@echo "   • WASM: $(WASM_PATH)"

# =====================================================================
# ❓ AJUDA
# =====================================================================

.PHONY: help
help:
	@echo ""
	@echo "$(GREEN)📘 BRAZA TOKEN - Comandos Disponíveis$(RESET)"
	@echo ""
	@echo "$(BLUE)🔨 build$(RESET)		   → Compila o contrato"
	@echo "$(BLUE)⚙ optimize$(RESET)		→ Otimiza o WASM"
	@echo "$(BLUE)🚀 build-opt$(RESET)	   → Build + otimização"
	@echo ""
	@echo "$(CYAN)📦 deploy$(RESET)		  → Deploy do contrato"
	@echo "$(CYAN)🧭 initialize$(RESET)	  → Inicializar contrato"
	@echo "$(CYAN)🔍 verify$(RESET)		  → Verificar deploy"
	@echo "$(CYAN)🚀 deploy-all$(RESET)	  → Pipeline completo"
	@echo ""
	@echo "$(YELLOW)🎨 fmt$(RESET)			 → Formatador Rust"
	@echo "$(YELLOW)🧹 lint$(RESET)			→ Clippy (warning = erro)"
	@echo "$(YELLOW)🗑 clean$(RESET)		   → Limpar target/"
	@echo "$(YELLOW)ℹ info$(RESET)			→ Exibir informações"
	@echo ""
	@echo "$(MAG)🧪 test$(RESET)			 → Testes unitários (futuro)"
	@echo "$(MAG)🧪 test-int$(RESET)		 → Testes de integração (futuro)"
	@echo ""
	@echo "$(GREEN)❓ help$(RESET)			→ Exibe esta ajuda"
	@echo ""
