#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

fn setup_env() -> (Env, Address, ComplianceOracleClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ComplianceOracle, ());
    let client = ComplianceOracleClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    (env, admin, client)
}

fn mock_data_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ])
}

// ─── Initialization ─────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (_env, admin, client) = setup_env();
    assert_eq!(client.admin(), admin);
    assert_eq!(client.entity_count(), 0);
    assert_eq!(client.last_updated(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize_fails() {
    let (env, _admin, client) = setup_env();
    let another = Address::generate(&env);
    client.initialize(&another);
}

// ─── Add Sanctioned ─────────────────────────────────────────────────────

#[test]
fn test_add_sanctioned() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);

    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1.clone());
    addresses.push_back(addr2.clone());

    let added = client.add_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(added, 2);
    assert_eq!(client.entity_count(), 2);
    assert!(client.is_sanctioned(&addr1));
    assert!(client.is_sanctioned(&addr2));
}

#[test]
fn test_add_duplicate_does_not_double_count() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let addr1 = Address::generate(&env);
    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1.clone());

    client.add_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(client.entity_count(), 1);

    // Add same address again
    let added = client.add_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(added, 0); // Should not count duplicates
    assert_eq!(client.entity_count(), 1);
}

// ─── Remove Sanctioned ──────────────────────────────────────────────────

#[test]
fn test_remove_sanctioned() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);

    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1.clone());
    addresses.push_back(addr2.clone());

    client.add_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(client.entity_count(), 2);

    // Remove addr1 only
    let mut to_remove = Vec::new(&env);
    to_remove.push_back(addr1.clone());

    let removed = client.remove_sanctioned(&to_remove, &hash, &data_source);
    assert_eq!(removed, 1);
    assert_eq!(client.entity_count(), 1);
    assert!(!client.is_sanctioned(&addr1));
    assert!(client.is_sanctioned(&addr2));
}

#[test]
fn test_remove_nonexistent_is_noop() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let addr1 = Address::generate(&env);
    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1.clone());

    let removed = client.remove_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(removed, 0);
    assert_eq!(client.entity_count(), 0);
}

// ─── is_sanctioned ──────────────────────────────────────────────────────

#[test]
fn test_unknown_address_is_clean() {
    let (env, _admin, client) = setup_env();
    let unknown = Address::generate(&env);
    assert!(!client.is_sanctioned(&unknown));
}

// ─── Batch Check ────────────────────────────────────────────────────────

#[test]
fn test_batch_check() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let sanctioned = Address::generate(&env);
    let clean = Address::generate(&env);

    let mut to_add = Vec::new(&env);
    to_add.push_back(sanctioned.clone());
    client.add_sanctioned(&to_add, &hash, &data_source);

    let mut to_check = Vec::new(&env);
    to_check.push_back(sanctioned.clone());
    to_check.push_back(clean.clone());

    let results = client.check_batch(&to_check);
    assert_eq!(results.get(0).unwrap(), true);   // sanctioned
    assert_eq!(results.get(1).unwrap(), false);  // clean
}

// ─── Admin Transfer ─────────────────────────────────────────────────────

#[test]
fn test_transfer_admin() {
    let (env, _admin, client) = setup_env();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);
}

// ─── Data Hash / Audit ──────────────────────────────────────────────────

#[test]
fn test_data_hash_updated_on_add() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let data_source = String::from_str(&env, "ofac_sdn");

    let addr = Address::generate(&env);
    let mut addresses = Vec::new(&env);
    addresses.push_back(addr);

    client.add_sanctioned(&addresses, &hash, &data_source);
    assert_eq!(client.data_hash(), hash);
}

// ─── Edge Cases ─────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_add_empty_list_fails() {
    let (env, _admin, client) = setup_env();
    let hash = mock_data_hash(&env);
    let source = String::from_str(&env, "ofac_sdn");
    let empty: Vec<Address> = Vec::new(&env);

    client.add_sanctioned(&empty, &hash, &source);
}
