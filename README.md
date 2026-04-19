# Engram Compliance Oracle

**On-chain sanctions screening for Stellar/Soroban DeFi.**

A Soroban smart contract that any DeFi protocol on Stellar can call to check if an address is on a global sanctions list — on-chain, in the same transaction.

🔗 **[Live Demo →](https://kytdemo.throbbing-cloud-0f8e.workers.dev/)**

## How It Works


1. **Engram's off-chain pipeline** ingests sanctions data (OFAC, OpenSanctions, etc.)
2. **Admin pushes updates** to the Soroban contract via `add_sanctioned()` / `remove_sanctioned()`
3. **Any DeFi protocol** calls `is_sanctioned(address)` — free, instant, on-chain

## Live on Testnet

**Soroban Contract:** [`CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF`](https://stellar.expert/explorer/testnet/contract/CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF)

> View the contract activity, invocations, and state on [Stellar.Expert](https://stellar.expert/explorer/testnet/contract/CCDAXPPXNXCM25QHYVEWDYBU3FJTNU6Z6BYCHTRRHJEXU6RGVD32PWQF).

## Community Flagged Addresses (Testnet)

The following addresses have been flagged via community reports on the Stellar testnet and are currently flagged on-chain:

| Address | Status |
|---|---|
| `GA4ALNXXELASVP2S4FZXQFVXP3BPST7S2MZ5KBCSTR4PK3442NSQ5EQB` | 🚩 Flagged |
| `GAZLTY5QNQQ4WBU6E3T3KKPZAREGARH6JQS4WF76QSWZ7GYTMGBDJZ5X` | 🚩 Flagged |

### Verify with the CLI

```bash
# Check a community-flagged address
./compliance.sh check GA4ALNXXELASVP2S4FZXQFVXP3BPST7S2MZ5KBCSTR4PK3442NSQ5EQB

# Check the second flagged address
./compliance.sh check GAZLTY5QNQQ4WBU6E3T3KKPZAREGARH6JQS4WF76QSWZ7GYTMGBDJZ5X
```

### Verify via Contract Call

```rust
use soroban_sdk::Address;

let oracle = ComplianceOracleClient::new(&env, &oracle_contract_id);

// Community-flagged addresses will return true
let addr1: Address = "GA4ALNXXELASVP2S4FZXQFVXP3BPST7S2MZ5KBCSTR4PK3442NSQ5EQB".parse().unwrap();
let addr2: Address = "GAZLTY5QNQQ4WBU6E3T3KKPZAREGARH6JQS4WF76QSWZ7GYTMGBDJZ5X".parse().unwrap();

assert!(oracle.is_sanctioned(&addr1));  // true — community flagged
assert!(oracle.is_sanctioned(&addr2));  // true — community flagged
```

## Quick Start

### For DeFi Integrators

```rust
use soroban_sdk::Address;

// In your DeFi contract:
let oracle = ComplianceOracleClient::new(&env, &oracle_contract_id);

// Single check
if oracle.is_sanctioned(&user_address) {
    panic!("Address is sanctioned — transaction blocked");
}

// Batch check
let results = oracle.check_batch(&addresses);
```

### Build & Test

```bash
# Build
stellar contract build

# Test
cargo test

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/engram_compliance_oracle.wasm \
  --source <SECRET_KEY> \
  --network testnet
```

## Contract Interface

| Function | Auth | Cost | Description |
|---|---|---|---|
| `is_sanctioned(addr)` | None | Free | Check if address is sanctioned |
| `check_batch(addrs)` | None | Free | Batch check multiple addresses |
| `entity_count()` | None | Free | Total sanctioned entities on-chain |
| `last_updated()` | None | Free | Timestamp of last update |
| `data_hash()` | None | Free | SHA-256 of source dataset (for audit) |
| `add_sanctioned(addrs, hash, source)` | Admin | Gas | Add addresses to sanctions list |
| `remove_sanctioned(addrs, hash, source)` | Admin | Gas | Remove addresses from list |
| `transfer_admin(new_admin)` | Admin + New | Gas | Transfer admin rights |

## Audit Trail

Every update records:
- **`data_hash`**: SHA-256 of the complete off-chain dataset snapshot
- **`source`**: Which data source triggered the update (e.g. `"ofac_sdn"`)
- **Events**: All additions/removals emit events for indexers

Download the dataset from Engram/Shelby, hash it, and compare against `data_hash()` to independently verify.

## Data Sources

| Source | Type | Coverage |
|---|---|---|
| [OFAC SDN](https://sanctionslist.ofac.treas.gov/) | Government | US sanctions list |
| [OpenSanctions](https://opensanctions.org/) | Open data | Global sanctions & PEPs |
| [Chainalysis](https://www.chainalysis.com/) | Commercial | Blockchain-specific intelligence |
| Community Reports | Crowdsourced | User-submitted flagged addresses |

## License

MIT
