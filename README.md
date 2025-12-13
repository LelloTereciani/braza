# 🇧🇷 BRAZA TOKEN - A Moeda do Empreendedor Brasileiro

[![Stellar](https://img.shields.io/badge/Stellar-Soroban-blue)](https://stellar.org)
[![SEP-41](https://img.shields.io/badge/SEP--41-Compliant-green)](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)
[![Testnet](https://img.shields.io/badge/Testnet-Deployed-success)](https://stellar.expert/explorer/testnet/contract/CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK)

> **A moeda regional que representa o empreendedorismo brasileiro que produz e carrega o Brasil.**

Token SEP-41 para Stellar/Soroban com funcionalidades avançadas de governança, taxação inteligente e economia descentralizada.

---

## 🇧🇷 Visão Geral

O **BRAZA** é a moeda do empreendedor brasileiro - aquele que acorda cedo, trabalha duro e move a economia real do país. Criada para libertar o produtor das amarras do sistema centralizado, o BRAZA representa a força da iniciativa privada e do livre mercado.

### Por que BRAZA?

- 🇧🇷 **Identidade Nacional:** Representa o Brasil empreendedor
- 💪 **Liberdade Econômica:** Fuga do controle governamental centralizado
- 🏭 **Economia Real:** Feita por quem produz, para quem produz
- 🚀 **Descentralização:** Poder nas mãos da comunidade, não do governo
- 💰 **Menor Unidade:** bra (1 BRAZA = 10.000.000 bra)

---

## 🎯 Características Principais

| Característica | Descrição |
|----------------|-----------|
| 🔒 **Supply Fixo** | 21 milhões BRAZA (inspirado no Bitcoin) |
| 💰 **Taxação Inteligente** | Progressiva e contextual |
| 🏘️ **Economia Local** | Taxa reduzida para comércio local (0.05%) |
| 🐋 **Anti-Concentração** | Taxa progressiva até 0.3% para grandes holders |
| 🔐 **Segurança** | Reentrancy guard, overflow protection, pause mechanism |
| 📊 **Vesting** | Cliff-based para alinhamento de longo prazo |
| 🎁 **Distribuição Sem Taxa** | Fundador pode distribuir tokens sem custo |

---

## 📊 Tokenomics

### Supply Distribution

| Categoria | Quantidade | Percentual | Vesting |
|-----------|------------|------------|---------|
| **Fundadores** | 8.4M BRAZA | 40% | 5-7 anos com cliff |
| **Comunidade** | 10.5M BRAZA | 50% | Distribuição gradual |
| **Reserva** | 2.1M BRAZA | 10% | Governança futura |
| **TOTAL** | 21M BRAZA | 100% | - |

### Estrutura de Taxas

#### Taxa Progressiva (Anti-Concentração)

| Tier | Holding | Taxa |
|------|---------|------|
| **Tier 1** | < 0.1% supply | 0.05% |
| **Tier 2** | 0.1% - 1% supply | 0.15% |
| **Tier 3** | > 1% supply | 0.30% |

#### Taxa Contextual

| Contexto | Taxa |
|----------|------|
| **Exchange → Exchange** | 0.10% |
| **Comércio Local** | 0.05% |
| **Distribuição Admin** | 0% (sem taxa) |

---

## 🏗️ Arquitetura do Contrato

### Módulos

``` braza_token/
├── src/
│   ├── lib.rs              # Entry point
│   ├── token.rs            # Funções principais SEP-41
│   ├── storage.rs          # Estruturas de dados e constantes
│   ├── validation.rs       # Validações e reentrancy guard
│   ├── events.rs           # Emissão de eventos
│   ├── vesting.rs          # Sistema de vesting
│   ├── admin.rs            # Funções administrativas
│   └── compliance.rs       # KYC, blacklist, limites
├── scripts/
│   ├── build.sh            # Compilar e otimizar
│   ├── deploy.sh           # Deploy na blockchain
│   ├── initialize.sh       # Inicializar contrato
│   ├── verify.sh           # Verificar deploy
│   └── deploy-all.sh       # Deploy completo automatizado
└── tests/
    └── (próximos passos)
```

### Funcionalidades Implementadas

#### SEP-41 Standard

- ✅ `initialize()` - Inicialização do contrato
- ✅ `name()` - Nome do token
- ✅ `symbol()` - Símbolo do token
- ✅ `decimals()` - Casas decimais (7)
- ✅ `balance()` - Consultar saldo
- ✅ `total_supply()` - Supply total
- ✅ `transfer()` - Transferir tokens
- ✅ `approve()` - Aprovar allowance
- ✅ `allowance()` - Consultar allowance
- ✅ `transfer_from()` - Transferir via allowance

#### Funcionalidades Avançadas

- ✅ `mint()` - Criar novos tokens (admin)
- ✅ `burn()` - Queimar tokens
- ✅ `blacklist()` - Sistema de blacklist
- ✅ `pause()` / `unpause()` - Pausar operações
- ✅ Vesting com cliff
- ✅ Timelock para ações administrativas
- ✅ Sistema de taxação progressiva
- ✅ Compliance (KYC, limites diários)

---

## 🚀 Deploy

### Informações do Contrato (Testnet)

| Item | Valor |
|------|-------|
| **Contract ID** | `CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK` |
| **Network** | Stellar Testnet |
| **Admin** | `G00000000000000000000000000000000000000000000000` |
| **Initial Supply** | 10,000,000 BRAZA |
| **Max Supply** | 21,000,000 BRAZA |
| **Deployed** | 2025 |

### Links Úteis

- 🔗 [Stellar Expert (Testnet)](https://stellar.expert/explorer/testnet/contract/CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK)
- 🔗 [Stellar Laboratory](https://laboratory.stellar.org/#explorer?resource=contracts&endpoint=single&network=test)

---

## 🛠️ Desenvolvimento

### Pré-requisitos

``` bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Stellar CLI
cargo install --locked stellar-cli --features opt

# Soroban SDK
rustup target add wasm32-unknown-unknown

Instalação
``` bash

# Clonar repositório
git clone https://github.com/seu-usuario/braza-token.git
cd braza-token/contracts/braza_token

# Configurar ambiente
cp .env.example .env
nano .env  # Editar com suas credenciais# Clonar repositório
git clone https://github.com/seu-usuario/braza-token.git
cd braza-token/contracts/braza_token

# Configurar ambiente

cp .env.example .env
nano .env  

# Editar com suas credenciaisBuild

``` bash

# Compilar e otimizar
./scripts/build.sh

# Saída esperada:
# [SUCCESS] Compilação concluída!
# [INFO] Original:   16KiB
# [INFO] Otimizado:  14KiB
# [SUCCESS] Redução:    15%# Compilar e otimizar
./scripts/build.sh

# Saída esperada:
# [SUCCESS] Compilação concluída!
# [INFO] Original:   16KiB
# [INFO] Otimizado:  14KiB
# [SUCCESS] Redução:    15%

Deploy

``` bash

# Deploy completo (build + deploy + initialize + verify)
./scripts/deploy-all.sh

# Ou executar etapas individuais:
./scripts/build.sh       # Compilar
./scripts/deploy.sh      # Deploy
./scripts/initialize.sh  # Inicializar
./scripts/verify.sh      # Verificar# Deploy completo (build + deploy + initialize + verify)
./scripts/deploy-all.sh

# Ou executar etapas individuais:
./scripts/build.sh       # Compilar
./scripts/deploy.sh      # Deploy
./scripts/initialize.sh  # Inicializar
./scripts/verify.sh      # Verificar🧪 Próximos Passos: Testes AbrangentesRoadmap de Testes (Cobertura ~100%)

1. Testes Unitários (tests/unit/).
Objetivo: Testar funções individuais isoladamente.

``` bash 

tests/unit/
├── test_token_basic.rs          # name, symbol, decimals, balance
├── test_transfers.rs            # transfer, transfer_from
├── test_allowances.rs           # approve, allowance, reset_allowance
├── test_mint_burn.rs            # mint, burn
├── test_vesting.rs              # create_vesting, release_vested
├── test_admin.rs                # pause, unpause, blacklist
├── test_fees.rs                 # calculate_progressive_fee
├── test_compliance.rs           # KYC, country restrictions
└── test_validation.rs           # Input validationstests/unit/
├── test_token_basic.rs          # name, symbol, decimals, balance
├── test_transfers.rs            # transfer, transfer_from
├── test_allowances.rs           # approve, allowance, reset_allowance
├── test_mint_burn.rs            # mint, burn
├── test_vesting.rs              # create_vesting, release_vested
├── test_admin.rs                # pause, unpause, blacklist
├── test_fees.rs                 # calculate_progressive_fee
├── test_compliance.rs           # KYC, country restrictions
└── test_validation.rs           # Input validationsCobertura esperada: 80-90%2.

2. Testes de Integração (tests/integration/).
Objetivo: Testar interações entre módulos

``` bash 


tests/integration/
├── test_full_lifecycle.rs       # Initialize → Transfer → Burn
├── test_vesting_flow.rs         # Create → Wait → Release
├── test_admin_flow.rs           # Pause → Unpause → Transfer
├── test_allowance_flow.rs       # Approve → Transfer_from
├── test_fee_collection.rs       # Transfer → Fee → Collector
└── test_timelock_flow.rs        # Propose → Wait → Executetests/integration/
├── test_full_lifecycle.rs       # Initialize → Transfer → Burn
├── test_vesting_flow.rs         # Create → Wait → Release
├── test_admin_flow.rs           # Pause → Unpause → Transfer
├── test_allowance_flow.rs       # Approve → Transfer_from
├── test_fee_collection.rs       # Transfer → Fee → Collector
└── test_timelock_flow.rs        # Propose → Wait → ExecuteCobertura esperada: 70-80%3. 

3. Testes de Performance (tests/performance/).
Objetivo: Validar limites de CPU, memória e storage

``` bash

tests/performance/
├── test_gas_limits.rs           # Medir CPU por função
├── test_storage_limits.rs       # Testar limites de storage
├── test_batch_operations.rs     # Transferências em lote
├── test_vesting_scale.rs        # 100+ vesting schedules
└── test_worst_case.rs           # Cenários de pior casotests/performance/
├── test_gas_limits.rs           # Medir CPU por função
├── test_storage_limits.rs       # Testar limites de storage
├── test_batch_operations.rs     # Transferências em lote
├── test_vesting_scale.rs        # 100+ vesting schedules
└── test_worst_case.rs           # Cenários de pior casoMétricas:

CPU: < 10M instruções por invocação
Storage: < 100KB por entrada
Latência: < 5s por transação

4. Testes de Funcionalidade (tests/functional/).
Objetivo: Validar requisitos de negócio

``` bash

tests/functional/
├── test_tokenomics.rs           # Supply, distribution, vesting
├── test_fee_structure.rs        # Taxas progressivas e contextuais
├── test_anti_concentration.rs   # Taxa para grandes holders
├── test_local_commerce.rs       # Taxa reduzida 0.05%
└── test_admin_distribution.rs   # Distribuição sem taxatests/functional/
├── test_tokenomics.rs           # Supply, distribution, vesting
├── test_fee_structure.rs        # Taxas progressivas e contextuais
├── test_anti_concentration.rs   # Taxa para grandes holders
├── test_local_commerce.rs       # Taxa reduzida 0.05%
└── test_admin_distribution.rs   # Distribuição sem taxaCenários:

✅ Supply nunca excede 21M BRAZA
✅ Vesting respeita cliff de 5-7 anos
✅ Taxas aplicadas corretamente por tier
✅ Admin pode distribuir sem taxa

5. Testes de Segurança (tests/security/).
Objetivo: Identificar vulnerabilidades

```bash

tests/security/
├── test_reentrancy.rs           # Ataques de reentrância
├── test_overflow.rs             # Overflow/underflow
├── test_authorization.rs        # Controle de acesso
├── test_front_running.rs        # Front-running em allowances
├── test_flash_loans.rs          # Flash loan attacks
├── test_timestamp_manipulation.rs # Manipulação de timestamp
└── test_dos.rs                  # Denial of Servicetests/security/
├── test_reentrancy.rs           # Ataques de reentrância
├── test_overflow.rs             # Overflow/underflow
├── test_authorization.rs        # Controle de acesso
├── test_front_running.rs        # Front-running em allowances
├── test_flash_loans.rs          # Flash loan attacks
├── test_timestamp_manipulation.rs # Manipulação de timestamp
└── test_dos.rs                  # Denial of ServiceVulnerabilidades testadas:
❌ Reentrancy (cross-contract)
❌ Integer overflow/underflow
❌ Unauthorized access
❌ Front-running
❌ Flash loan attacks
❌ Timestamp manipulation
❌ DoS via storage explosion

6. Testes Fuzzy (tests/fuzzy/).
Objetivo: Encontrar edge cases com inputs aleatórios

```bash

tests/fuzzy/
├── fuzz_transfer.rs             # Inputs aleatórios para transfer
├── fuzz_mint_burn.rs            # Inputs aleatórios para mint/burn
├── fuzz_vesting.rs              # Inputs aleatórios para vesting
└── fuzz_fees.rs                 # Inputs aleatórios para cálculo de taxastests/fuzzy/
├── fuzz_transfer.rs             # Inputs aleatórios para transfer
├── fuzz_mint_burn.rs            # Inputs aleatórios para mint/burn
├── fuzz_vesting.rs              # Inputs aleatórios para vesting
└── fuzz_fees.rs                 # Inputs aleatórios para cálculo de taxasFerramentas:

cargo-fuzz (libFuzzer)
proptest (property-based testing)
Execução:

```bash

cargo fuzz run fuzz_transfer -- -max_total_time=3600cargo fuzz run fuzz_transfer -- -max_total_time=36007.

7. Testes de Cobertura (tests/coverage/).
Objetivo: Medir cobertura de código.

```bash

# Instalar tarpaulin
cargo install cargo-tarpaulin

# Executar testes com cobertura
cargo tarpaulin --out Html --output-dir coverage

# Visualizar relatório
open coverage/index.html# Instalar tarpaulin
cargo install cargo-tarpaulin

# Executar testes com cobertura
cargo tarpaulin --out Html --output-dir coverage

# Visualizar relatório
open coverage/index.htmlMeta: 95%+ de cobertura.

8. Testes de Regressão (tests/regression/)
Objetivo: Garantir que correções não quebrem funcionalidades.

``` bash

tests/regression/
├── test_critical_01_fix.rs      # Validar correção CRÍTICO-01
├── test_critical_02_fix.rs      # Validar correção CRÍTICO-02
├── test_critical_03_fix.rs      # Validar correção CRÍTICO-03
└── test_high_risk_fixes.rs      # Validar correções ALTO-01 a 05tests/regression/
├── test_critical_01_fix.rs      # Validar correção CRÍTICO-01
├── test_critical_02_fix.rs      # Validar correção CRÍTICO-02
├── test_critical_03_fix.rs      # Validar correção CRÍTICO-03
└── test_high_risk_fixes.rs      # Validar correções ALTO-01 a 059. Testes End-to-End (tests/e2e/)
Objetivo: Simular uso real na testnet.

``` bash

9. Testes end-to-end (ponta a ponta).
Objetivo: Verificam se todo o sistema funciona corretamente quando integrado.

tests/e2e/
├── test_user_journey.rs         # Jornada completa do usuário
├── test_exchange_integration.rs # Integração com exchanges
├── test_wallet_integration.rs   # Integração com wallets
└── test_mainnet_simulation.rs   # Simulação de mainnettests/e2e/
├── test_user_journey.rs         # Jornada completa do usuário
├── test_exchange_integration.rs # Integração com exchanges
├── test_wallet_integration.rs   # Integração com wallets
└── test_mainnet_simulation.rs   # Simulação de mainnet10.

10.  Testes de Stress (tests/stress/)
Objetivo: Validar comportamento sob carga

``` bash

tests/stress/
├── test_high_volume.rs          # 1000+ transações/segundo
├── test_concurrent_users.rs     # 100+ usuários simultâneos
├── test_storage_growth.rs       # Crescimento de storage
└── test_network_congestion.rs   # Rede congestionadatests/stress/
├── test_high_volume.rs          # 1000+ transações/segundo
├── test_concurrent_users.rs     # 100+ usuários simultâneos
├── test_storage_growth.rs       # Crescimento de storage
└── test_network_congestion.rs   # Rede congestionadaEstrutura de Testes Propostatests/
├── unit/                 # Testes unitários (80-90% cobertura)
├── integration/          # Testes de integração (70-80% cobertura)
├── performance/          # Testes de performance
├── functional/           # Testes de funcionalidade
├── security/             # Testes de segurança
├── fuzzy/                # Testes fuzzy
├── coverage/             # Relatórios de cobertura
├── regression/           # Testes de regressão
├── e2e/                  # Testes end-to-end
├── stress/               # Testes de stress
└── fixtures/             # Dados de teste compartilhados
    ├── accounts.rs       # Contas de teste
    ├── scenarios.rs      # Cenários comuns
    └── helpers.rs        # Funções auxiliares

# Testes unitários
cargo test --lib

# Testes de integração
cargo test --test '*'

# Testes com cobertura
cargo tarpaulin --out Html

# Testes fuzzy (1 hora)
cargo fuzz run fuzz_transfer -- -max_total_time=3600

# Testes de performance
cargo test --release --test test_performance

# Todos os testes
./scripts/run-all-tests.sh

📋 Checklist de Qualidade antes do Deploy em Mainnet

 ✅ Testes unitários (>90% cobertura)
 ✅ Testes de integração (>80% cobertura)
 ✅ Testes de segurança (todas vulnerabilidades corrigidas)
 ✅ Testes fuzzy (24h sem crashes)
 ✅ Auditoria externa (firma especializada)
 ✅ Bug bounty (programa ativo)
 ✅ Documentação completa
 ✅ Análise formal (opcional)

📄 Relatório de Auditoria Completo
🔒 Auditoria externa: Pendente
🐛 Bug Bounty: Planejado
📚 Documentação
📖 Whitepaper
🔧 Guia de Desenvolvimento
🚀 Guia de Deploy
�� Guia de Testes
🔐 Análise de Segurança
📊 Tokenomics Detalhado
🤝 ContribuindoContribuições são bem-vindas! Por favor, leia nosso Guia de Contribuição.

Como Contribuir

Fork o projeto
Crie uma branch (git checkout -b feature/nova-funcionalidade)
Commit suas mudanças (git commit -m 'Adiciona nova funcionalidade')
Push para a branch (git push origin feature/nova-funcionalidade)
Abra um Pull Request

📜 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo LICENSE para detalhes.

👥 Equipe

Fundador: Wesley (Desenvolvedor)
Desenvolvedor Principal: Wesley
Auditoria: Agente especialista Rust Soroban

📞 Contato

🌐 Website: braza.finance
📧 Email: contato@braza.finance
💬 Telegram: @brazatoken
🐦 Twitter: @brazatoken

🙏 Agradecimentos

Stellar Development Foundation
Comunidade Soroban
Empreendedores brasileiros

🇧🇷 BRAZA - A Moeda do Empreendedor Brasileiro.
🇧🇷 Feito com ❤️ por um empreendedor, para empreendedores.
