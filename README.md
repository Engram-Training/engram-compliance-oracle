# Engram Compliance Oracle

**Multi-layer on-chain compliance for Stellar/Soroban DeFi.**

A Soroban smart contract providing 4 layers of protection — Merkle-verified sanctions screening, agent consensus flagging, community reporting, and **automatic taint propagation with cross-chain bridge tracking** — all callable on-chain, in the same transaction.

🔗 **[Live Demo →](https://kytdemo.throbbing-cloud-0f8e.workers.dev/)**

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                      On-Chain (Soroban)                               │
│                                                                      │
│  Layer 1: Merkle Sanctions  → OFAC/OpenSanctions via Merkle proofs  │
│  Layer 2: Agent Consensus   → Multiple agents report → auto-flag    │
│  Layer 3: Community Reports → On-chain reports with operator review  │
│  Layer 4: Taint Propagation → Auto-flag recipients of flagged wallets│
│                                                                      │
│  is_flagged()  → covers ALL 4 layers in a single call                │
└──────────────────────────────────────────────────────────────────────┘
                              ▲
                              │ set_taint()
                              │
┌──────────────────────────────────────────────────────────────────────┐
│                   Off-Chain (Taint Watcher v0.7.0)                   │
│                                                                      │
│  ┌────────────────┐   ┌──────────────────────┐                      │
│  │ StellarWatcher │   │ NearIntentsWatcher   │                      │
│  │ (SSE stream)   │   │ (bridge polling)     │                      │
│  └───────┬────────┘   └──────────┬───────────┘                      │
│          │                       │                                   │
│          ▼                       ▼                                   │
│  ┌──────────────────────────────────────────┐                       │
│  │         Scoring → Whitelist → Push       │                       │
│  │     (dual: Engram API + Soroban)         │                       │
│  └──────────────────────────────────────────┘                       │
└──────────────────────────────────────────────────────────────────────┘
```

## Live on Testnet

**Contract:** [`CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF`](https://stellar.expert/explorer/testnet/contract/CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF)
**Version:** v0.7.0 — 38 exported functions
**WASM Hash:** `1a661658c3f6d8c9d2851e6b48cb62f701064ecde3456f08f3f2c96567a14770`

## Quick Start for DeFi Protocols

Add compliance to your Soroban DEX/lending protocol with **2 lines of code**:

```rust
// Import the oracle client (generated from the deployed WASM)
let oracle = ComplianceOracleClient::new(&env, &oracle_contract_id);

// Single check — covers sanctions, consensus flags, AND taint
if oracle.is_flagged(&user_address) {
    panic!("Address is flagged — transaction blocked");
}
```

That's it. One cross-contract call covers all 4 layers.

### Need more granularity?

```rust
// Why is it flagged? 0=clean, 1=consensus, 2=tainted, 3=both
let reason = oracle.flag_reason(&user_address);

// Taint details — score, source, chain, hop depth
let score = oracle.taint_score(&user_address);   // 0–100
let source = oracle.taint_source(&user_address);  // who sent funds
let chain = oracle.taint_chain(&user_address);     // which blockchain
let hop = oracle.taint_hop(&user_address);         // distance from sanctioned

// Is this address whitelisted? (exchanges, custodians)
let safe = oracle.is_whitelisted(&user_address);

// Merkle proof verification (sanctions list)
let verified = oracle.verify_merkle_proof(&address, &proof, &leaf_index);
```

### Example Contract

See [`examples/compliant-swap/`](examples/compliant-swap/) for a full DEX integration showing 3 patterns:

| Pattern | Function | Use Case |
|---|---|---|
| **Simple Gate** | `swap_simple()` | Block all flagged addresses. Start here. |
| **Score-Based** | `swap_scored()` | Allow low-risk, block high-risk (configurable threshold) |
| **Full Provenance** | `swap_audited()` | Emit taint context as events for audit trails |

```bash
# Build the example
cargo build --target wasm32-unknown-unknown --release -p compliant-swap
```

## Build & Test

```bash
# Build the oracle
stellar contract build

# Run tests (40 tests)
cargo test

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/engram_compliance_oracle.wasm \
  --source <SECRET_KEY> \
  --network testnet
```

## Taint Watcher Service (v0.7.0)

The `watcher/` directory contains the multi-chain Taint Watcher — an off-chain service that monitors payment streams from flagged addresses and automatically propagates taint to recipients, **both on-chain and off-chain**.

```bash
cd watcher
cp .env.example .env   # Configure API keys
npm install
npm run dev             # Start watching
```

### Multi-Chain Support

| Chain Watcher | Method | Coverage |
|---|---|---|
| **StellarWatcher** | Horizon SSE stream | All Stellar payments in real-time |
| **NearIntentsWatcher** | Explorer API polling | 25+ chains via NEAR Intents bridge (ETH, BTC, SOL, Base, Arb, etc.) |

When the watcher detects a flagged address sending funds (on Stellar or through a bridge), it:

1. **Scores** the payment (severity × amount × hop decay)
2. **Checks whitelist** (skip if recipient is a known exchange)
3. **Pushes taint** to both the Engram API and the Soroban contract
4. **Subscribes** the recipient for recursive monitoring

### Cross-Chain Bridge Detection

```
OFAC address GABCD... sends 1000 XLM to NEAR Intents bridge
  → Bridge routes to 0xEF89... on Ethereum
  → Watcher detects cross-chain transfer
  → Taint propagated with provenance:
      0xEF89... (ETH, hop 2) ← GABCD... (XLM, hop 1) ← OFAC source
```

### Safety Features

- **Dust attack protection** — configurable minimum amount (default 100 XLM)
- **Exchange whitelist** — Coinbase, Binance, Kraken, Lobstr auto-skip
- **Rate limiting** — 500 taints/hour, 200 new watches/hour
- **Storm detection** — >50 from single source in 10 min → manual review
- **Circuit breakers** — automatic pause on anomalous activity

## CLI (compliance.sh)

```bash
# Full compliance check (flags, taint, whitelist, Merkle)
./compliance.sh check <address>

# Report a suspicious address
./compliance.sh report <address> "reason"

# Set taint for an address (operator)
./compliance.sh taint <address> <score> <source_addr> <hop> [chain]

# Clear taint (operator)
./compliance.sh clear-taint <address>

# Manage exchange whitelist
./compliance.sh whitelist add <address>
./compliance.sh whitelist remove <address>
./compliance.sh whitelist check <address>

# View taint configuration
./compliance.sh taint-config

# View contract status
./compliance.sh status
```

## Contract Interface

### Core Screening (Free — Anyone)

| Function | Description |
|---|---|
| `is_flagged(addr)` | Returns true if flagged by consensus **or** tainted |
| `flag_reason(addr)` | 0=clean, 1=consensus, 2=tainted, 3=both |
| `verify_merkle_proof(addr, proof, idx)` | Verify against sanctions Merkle tree |
| `verify_batch_proofs(addrs, proofs, idxs)` | Batch verify (max 200) |

### Taint Propagation (Free — Anyone can read, Operator can write)

| Function | Auth | Description |
|---|---|---|
| `is_tainted(addr)` | None | Check if address is tainted |
| `taint_score(addr)` | None | Taint score (0–100) |
| `taint_source(addr)` | None | Source address that caused taint |
| `taint_hop(addr)` | None | Hop depth from original sanctioned |
| `taint_chain(addr)` | None | Chain where taint originated |
| `set_taint(addr, score, src, hop, chain)` | Operator | Set taint (auto-called by watcher) |
| `set_taint_batch(addrs, scores, ...)` | Operator | Batch set (max 200) |
| `clear_taint(addr)` | Operator | Remove taint |

### Whitelist

| Function | Auth | Description |
|---|---|---|
| `whitelist_address(addr)` | Operator | Exempt from taint (exchanges, custodials) |
| `unwhitelist_address(addr)` | Operator | Remove exemption |
| `is_whitelisted(addr)` | None | Check whitelist status |

### Configuration

| Function | Auth | Description |
|---|---|---|
| `taint_config()` | None | Returns (min_amount, max_hops) |
| `set_taint_min_amount(amount)` | Operator | Minimum taint amount (stroops) |
| `set_taint_max_hops(max_hops)` | Operator | Max propagation depth |
| `set_report_threshold(n)` | Operator | Auto-flag threshold |

### Community Reporting

| Function | Auth | Description |
|---|---|---|
| `report_address(reporter, target, reason)` | Reporter | Submit community report |
| `reports_for(addr)` | None | Report count for address |
| `get_report(id)` | None | Report details |
| `review_report(id, accept)` | Operator | Accept/reject report |

### Admin

| Function | Auth | Description |
|---|---|---|
| `initialize(owner, operator)` | None | One-time setup |
| `set_merkle_root(root, hash, count)` | Operator | Update sanctions tree |
| `set_operator(new_op)` | Owner | Change operator |
| `transfer_owner(new_owner)` | Owner | Transfer ownership |
| `upgrade(new_wasm_hash)` | Owner | Upgrade contract |

## Data Sources

| Source | Type | Layer | Coverage |
|---|---|---|---|
| [OFAC SDN](https://sanctionslist.ofac.treas.gov/) | Government | 1 | US sanctions list |
| [OpenSanctions](https://opensanctions.org/) | Open data | 1 | Global sanctions & PEPs |
| Community Reports | Crowdsourced | 2–3 | Agent-submitted flags |
| Taint Propagation | Automated | 4 | Recipients of flagged wallets |
| NEAR Intents Bridge | Cross-chain | 4 | Bridge transfers across 25+ chains |

## MCP Integration

AI agents can query compliance data through [Engram MCP tools](https://engram.sh):

```
screen_address  → Full compliance screening with taint enrichment
get_taint       → Taint status for a specific address
trace_taint     → Full provenance chain from destination → source
```

## License

MIT
