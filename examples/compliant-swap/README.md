# Compliant Swap — DeFi Integration Example

A minimal Soroban DEX contract demonstrating how to integrate the **Engram Compliance Oracle** for on-chain taint screening. All compliance checks happen on-chain, in the same transaction — no API keys, no off-chain lookups.

## 3 Integration Patterns

### Pattern 1: Simple Gate (`swap_simple`)

The recommended starting point. One cross-contract call blocks any flagged address — sanctions, consensus, or taint.

```rust
let oracle = oracle::Client::new(&env, &oracle_id);

if oracle.is_flagged(&sender_str) {
    return Err(SwapError::SenderFlagged);
}
```

**Best for:** Most protocols. Zero configuration. Covers all 4 layers.

### Pattern 2: Score-Based (`swap_scored`)

More nuanced — allows addresses with low taint scores (e.g. 2-hop indirect recipients) while blocking high-risk addresses.

```rust
let sender_score = oracle.taint_score(&sender_str);
if sender_score >= threshold {
    return Err(SwapError::SenderTaintTooHigh);
}
```

**Best for:** DEXs that want to avoid blocking innocent users who received funds indirectly from a tainted address.

### Pattern 3: Full Provenance (`swap_audited`)

Maximum visibility. Emits taint metadata as contract events even for clean addresses, giving indexers and frontends rich compliance context.

```rust
let sender_score = oracle.taint_score(&sender_str);
let sender_chain = oracle.taint_chain(&sender_str);
let sender_hop = oracle.taint_hop(&sender_str);

env.events().publish(
    (Symbol::new(&env, "compliance_check"),),
    (sender_str, sender_score, sender_hop, sender_chain),
);
```

**Best for:** Institutional DEXs needing full audit trails.

## Pre-Swap Compliance Check

For frontends — check if a swap would pass before submitting the transaction:

```rust
let (is_clean, score, reason) = compliant_swap.check_compliance(&user_address);
// is_clean: true if would pass
// score: 0–100 taint score  
// reason: 0=clean, 1=consensus, 2=tainted, 3=both
```

## Build

```bash
# From the workspace root
cargo build --target wasm32-unknown-unknown --release -p compliant-swap
```

## Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/compliant_swap.wasm \
  --source <SECRET_KEY> \
  --network testnet

# Initialize with the oracle contract ID
stellar contract invoke \
  --id <SWAP_CONTRACT_ID> \
  --source <SECRET_KEY> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --oracle_id CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF \
  --block_threshold 60
```

## Oracle Contract

The example connects to the deployed Engram Compliance Oracle:

**Testnet:** `CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF`

See the [main README](../../README.md) for the full contract interface.
