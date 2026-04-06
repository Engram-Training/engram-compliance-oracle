#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, BytesN, Env, String, Symbol, Vec};

// ─── Storage Keys ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin address (only entity that can update sanctions data)
    Admin,
    /// Whether an address is sanctioned: DataKey::Sanctioned(addr) → bool
    Sanctioned(Address),
    /// Total number of sanctioned entities currently on-chain
    EntityCount,
    /// Ledger timestamp of last sanctions list update
    LastUpdated,
    /// SHA-256 hash of the full off-chain dataset (for audit verification)
    DataHash,
    /// Human-readable data source identifier (e.g. "ofac_sdn")
    DataSource(u32),
    /// Number of data sources
    SourceCount,
}

// ─── Error Codes ────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OracleError {
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet
    NotInitialized = 2,
    /// Caller is not the admin
    Unauthorized = 3,
    /// Empty address list provided
    EmptyList = 4,
    /// Batch size exceeds maximum (200)
    BatchTooLarge = 5,
}

// ─── Events ─────────────────────────────────────────────────────────────────

const MAX_BATCH_SIZE: u32 = 200;

// ─── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct ComplianceOracle;

#[contractimpl]
impl ComplianceOracle {
    // ── Admin / Lifecycle ───────────────────────────────────────────────

    /// Initialize the oracle with an admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), OracleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EntityCount, &0u32);
        env.storage().instance().set(&DataKey::SourceCount, &0u32);

        env.events().publish(
            (Symbol::new(&env, "initialized"),),
            admin,
        );
        Ok(())
    }

    /// Transfer admin to a new address. Both old and new admin must authorize.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), OracleError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();
        new_admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events().publish(
            (Symbol::new(&env, "admin_transferred"),),
            new_admin,
        );
        Ok(())
    }

    /// Returns the current admin address.
    pub fn admin(env: Env) -> Result<Address, OracleError> {
        Self::require_admin(&env)
    }

    // ── Read (Free) ─────────────────────────────────────────────────────

    /// Check if an address is sanctioned.
    /// Returns `true` if the address is on the sanctions list.
    /// This is a free `view` call — no transaction fees for callers.
    pub fn is_sanctioned(env: Env, addr: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Sanctioned(addr))
            .unwrap_or(false)
    }

    /// Returns the total number of sanctioned entities on-chain.
    pub fn entity_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EntityCount)
            .unwrap_or(0u32)
    }

    /// Returns the ledger timestamp of the last sanctions list update.
    pub fn last_updated(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastUpdated)
            .unwrap_or(0u64)
    }

    /// Returns the SHA-256 hash of the off-chain dataset used for the
    /// latest update. Enables independent audit — download the dataset
    /// from Engram/Shelby, hash it, and compare.
    pub fn data_hash(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::DataHash)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }

    // ── Write (Admin Only) ──────────────────────────────────────────────

    /// Add addresses to the sanctions list.
    /// - `addresses`: up to 200 addresses per call
    /// - `data_hash`: SHA-256 of the full dataset snapshot
    /// - `source`: data source identifier (e.g. "ofac_sdn")
    pub fn add_sanctioned(
        env: Env,
        addresses: Vec<Address>,
        data_hash: BytesN<32>,
        data_source: String,
    ) -> Result<u32, OracleError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();

        let count = addresses.len();
        if count == 0 {
            return Err(OracleError::EmptyList);
        }
        if count > MAX_BATCH_SIZE {
            return Err(OracleError::BatchTooLarge);
        }

        let mut added: u32 = 0;
        for addr in addresses.iter() {
            let key = DataKey::Sanctioned(addr.clone());
            if !env.storage().persistent().has(&key) {
                env.storage().persistent().set(&key, &true);
                added += 1;
            }
        }

        // Update metadata
        let prev_count: u32 = env.storage().instance().get(&DataKey::EntityCount).unwrap_or(0);
        env.storage().instance().set(&DataKey::EntityCount, &(prev_count + added));
        env.storage().instance().set(&DataKey::LastUpdated, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::DataHash, &data_hash);

        env.events().publish(
            (Symbol::new(&env, "sanctioned_added"),),
            (added, data_source),
        );

        Ok(added)
    }

    /// Remove addresses from the sanctions list.
    /// Used when OFAC delists an entity.
    pub fn remove_sanctioned(
        env: Env,
        addresses: Vec<Address>,
        data_hash: BytesN<32>,
        data_source: String,
    ) -> Result<u32, OracleError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();

        let count = addresses.len();
        if count == 0 {
            return Err(OracleError::EmptyList);
        }
        if count > MAX_BATCH_SIZE {
            return Err(OracleError::BatchTooLarge);
        }

        let mut removed: u32 = 0;
        for addr in addresses.iter() {
            let key = DataKey::Sanctioned(addr.clone());
            if env.storage().persistent().has(&key) {
                env.storage().persistent().remove(&key);
                removed += 1;
            }
        }

        // Update metadata
        let prev_count: u32 = env.storage().instance().get(&DataKey::EntityCount).unwrap_or(0);
        let new_count = if removed > prev_count { 0 } else { prev_count - removed };
        env.storage().instance().set(&DataKey::EntityCount, &new_count);
        env.storage().instance().set(&DataKey::LastUpdated, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::DataHash, &data_hash);

        env.events().publish(
            (Symbol::new(&env, "sanctioned_removed"),),
            (removed, data_source),
        );

        Ok(removed)
    }

    /// Batch check multiple addresses at once.
    /// Returns a vector of booleans in the same order as the input.
    pub fn check_batch(env: Env, addresses: Vec<Address>) -> Vec<bool> {
        let mut results = Vec::new(&env);
        for addr in addresses.iter() {
            let sanctioned = env
                .storage()
                .persistent()
                .get(&DataKey::Sanctioned(addr))
                .unwrap_or(false);
            results.push_back(sanctioned);
        }
        results
    }

    // ── Internal Helpers ────────────────────────────────────────────────

    fn require_admin(env: &Env) -> Result<Address, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test;
