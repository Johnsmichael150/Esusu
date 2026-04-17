#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

use crate::contract::{EsusuContract, EsusuContractClient};
use crate::errors::ContractError;
use crate::types::{CircleStatus, MemberStatus, PayoutOrder};

// ─── helpers ────────────────────────────────────────────────────────────────

/// Deploy a minimal USDC mock token and return (token_id, admin).
fn create_token(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    (token_id, admin)
}

/// Mint `amount` of the token to `to`.
fn mint(env: &Env, token_id: &Address, admin: &Address, to: &Address, amount: i128) {
    let token_admin = token::StellarAssetClient::new(env, token_id);
    token_admin.mint(to, &amount);
}

/// Deploy the EsusuContract and return its client.
fn deploy_contract(env: &Env) -> (Address, EsusuContractClient) {
    let contract_id = env.register_contract(None, EsusuContract);
    let client = EsusuContractClient::new(env, &contract_id);
    (contract_id, client)
}

/// Standard circle setup: 2 members, 10 USDC contribution, 0 deposit, Fixed order.
fn setup_circle(
    env: &Env,
    client: &EsusuContractClient,
    creator: &Address,
    token_id: &Address,
    max_members: u32,
    contribution_amount: i128,
    deposit_amount: i128,
) {
    client.initialize(
        creator,
        &contribution_amount,
        &7_u32,
        &max_members,
        &deposit_amount,
        &PayoutOrder::Fixed,
    );
}

// ─── initialize tests ───────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    client.initialize(&creator, &10_000_000_i128, &7_u32, &3_u32, &0_i128, &PayoutOrder::Fixed);

    let state = client.get_state();
    assert_eq!(state.status, CircleStatus::Pending);
    assert_eq!(state.contribution_amount, 10_000_000);
    assert_eq!(state.max_members, 3);
    assert_eq!(state.current_cycle, 0);
}

#[test]
fn test_initialize_rejects_zero_contribution() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let result = client.try_initialize(&creator, &0_i128, &7_u32, &3_u32, &0_i128, &PayoutOrder::Fixed);
    assert_eq!(result, Err(Ok(ContractError::InvalidConfig)));
}

#[test]
fn test_initialize_rejects_member_count_out_of_bounds() {
    let env = Env::default();
    env.mock_all_auths();
    let creator = Address::generate(&env);

    // max_members = 1 (too low)
    let (_, client) = deploy_contract(&env);
    let result = client.try_initialize(&creator, &1_000_000_i128, &7_u32, &1_u32, &0_i128, &PayoutOrder::Fixed);
    assert_eq!(result, Err(Ok(ContractError::InvalidConfig)));

    // max_members = 51 (too high)
    let (_, client2) = deploy_contract(&env);
    let result2 = client2.try_initialize(&creator, &1_000_000_i128, &7_u32, &51_u32, &0_i128, &PayoutOrder::Fixed);
    assert_eq!(result2, Err(Ok(ContractError::InvalidConfig)));
}

#[test]
fn test_initialize_rejects_zero_cycle_length() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let result = client.try_initialize(&creator, &1_000_000_i128, &0_u32, &3_u32, &0_i128, &PayoutOrder::Fixed);
    assert_eq!(result, Err(Ok(ContractError::InvalidConfig)));
}

#[test]
fn test_initialize_cannot_be_called_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    client.initialize(&creator, &1_000_000_i128, &7_u32, &3_u32, &0_i128, &PayoutOrder::Fixed);
    let result = client.try_initialize(&creator, &1_000_000_i128, &7_u32, &3_u32, &0_i128, &PayoutOrder::Fixed);
    assert_eq!(result, Err(Ok(ContractError::AlreadyInitialized)));
}

// ─── join tests ─────────────────────────────────────────────────────────────

#[test]
fn test_join_success_no_deposit() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _admin) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 3, 10_000_000, 0);

    let member = Address::generate(&env);
    let pos = client.join(&member, &token_id);
    assert_eq!(pos, 1);

    let status = client.get_member_status(&member);
    assert_eq!(status, MemberStatus::Active);
}

#[test]
fn test_join_rejects_duplicate_member() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 3, 10_000_000, 0);

    let member = Address::generate(&env);
    client.join(&member, &token_id);
    let result = client.try_join(&member, &token_id);
    assert_eq!(result, Err(Ok(ContractError::AlreadyMember)));
}

#[test]
fn test_join_rejects_when_full() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 2, 10_000_000, 0);

    client.join(&Address::generate(&env), &token_id);
    client.join(&Address::generate(&env), &token_id); // fills circle — now Active

    // Circle is now Active (not Pending), so joining returns CircleNotActive
    let extra = Address::generate(&env);
    let result = client.try_join(&extra, &token_id);
    assert_eq!(result, Err(Ok(ContractError::CircleNotActive)));
}

#[test]
fn test_join_activates_circle_when_full() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 2, 10_000_000, 0);

    client.join(&Address::generate(&env), &token_id);
    client.join(&Address::generate(&env), &token_id);

    let state = client.get_state();
    assert_eq!(state.status, CircleStatus::Active);
    assert_eq!(state.current_cycle, 1);
    assert_eq!(state.payout_order.len(), 2);
}

#[test]
fn test_join_with_deposit_transfers_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, admin) = create_token(&env);
    let (contract_id, client) = deploy_contract(&env);
    let creator = Address::generate(&env);
    let member = Address::generate(&env);

    let deposit = 5_000_000_i128;
    setup_circle(&env, &client, &creator, &token_id, 3, 10_000_000, deposit);

    mint(&env, &token_id, &admin, &member, deposit);
    client.join(&member, &token_id);

    // Contract should hold the deposit
    let token_client = token::Client::new(&env, &token_id);
    assert_eq!(token_client.balance(&contract_id), deposit);
}

// ─── contribute tests ────────────────────────────────────────────────────────

/// Helper: create a 2-member active circle and return (client, token_id, contract_id, [m1, m2]).
fn active_circle_2(env: &Env) -> (EsusuContractClient, Address, Address, Address, Address) {
    env.mock_all_auths();
    let (token_id, admin) = create_token(env);
    let (contract_id, client) = deploy_contract(env);
    let creator = Address::generate(env);

    setup_circle(env, &client, &creator, &token_id, 2, 10_000_000, 0);

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    mint(env, &token_id, &admin, &m1, 100_000_000);
    mint(env, &token_id, &admin, &m2, 100_000_000);

    client.join(&m1, &token_id);
    client.join(&m2, &token_id); // activates

    (client, token_id, contract_id, m1, m2)
}

#[test]
fn test_contribute_success() {
    let env = Env::default();
    let (client, token_id, _, m1, _) = active_circle_2(&env);

    client.contribute(&m1, &token_id);
    // No panic = success; state should record contribution
    let state = client.get_state();
    assert_eq!(state.current_cycle, 1);
}

#[test]
fn test_contribute_rejects_non_member() {
    let env = Env::default();
    let (client, token_id, _, _, _) = active_circle_2(&env);

    let outsider = Address::generate(&env);
    let result = client.try_contribute(&outsider, &token_id);
    assert_eq!(result, Err(Ok(ContractError::NotAMember)));
}

#[test]
fn test_contribute_rejects_double_payment() {
    let env = Env::default();
    let (client, token_id, _, m1, _) = active_circle_2(&env);

    client.contribute(&m1, &token_id);
    let result = client.try_contribute(&m1, &token_id);
    assert_eq!(result, Err(Ok(ContractError::AlreadyPaid)));
}

#[test]
fn test_contribute_rejects_after_window_closed() {
    let env = Env::default();
    let (client, token_id, _, m1, _) = active_circle_2(&env);

    let state = client.get_state();
    // Advance ledger past cycle_end_timestamp
    env.ledger().set(LedgerInfo {
        timestamp: state.cycle_end_timestamp + 1,
        protocol_version: 20,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let result = client.try_contribute(&m1, &token_id);
    assert_eq!(result, Err(Ok(ContractError::WindowClosed)));
}

// ─── try_payout tests ────────────────────────────────────────────────────────

#[test]
fn test_try_payout_returns_false_when_not_all_paid() {
    let env = Env::default();
    let (client, token_id, _, m1, _) = active_circle_2(&env);

    // Only m1 contributes
    client.contribute(&m1, &token_id);
    let result = client.try_payout(&token_id);
    assert!(!result);
}

#[test]
fn test_try_payout_executes_when_all_paid() {
    let env = Env::default();
    let (client, token_id, _, m1, m2) = active_circle_2(&env);

    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);

    let result = client.try_payout(&token_id);
    assert!(result);

    // Cycle should have advanced
    let state = client.get_state();
    assert_eq!(state.current_cycle, 2);
}

#[test]
fn test_try_payout_completes_circle_on_last_cycle() {
    let env = Env::default();
    let (client, token_id, _, m1, m2) = active_circle_2(&env);

    // Cycle 1
    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);

    // Cycle 2 — need fresh funds
    let (_, admin) = create_token(&env); // won't work; reuse existing token
    // Mint more for cycle 2
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&m1, &10_000_000_i128);
    token_admin.mint(&m2, &10_000_000_i128);

    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);

    let state = client.get_state();
    assert_eq!(state.status, CircleStatus::Completed);
}

// ─── payout amount correctness ───────────────────────────────────────────────

#[test]
fn test_payout_amount_equals_pooled_contributions() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, admin) = create_token(&env);
    let (contract_id, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let contribution = 10_000_000_i128;
    let n = 3_u32;
    setup_circle(&env, &client, &creator, &token_id, n, contribution, 0);

    let members: std::vec::Vec<Address> = (0..n).map(|_| Address::generate(&env)).collect();
    for m in &members {
        mint(&env, &token_id, &admin, m, contribution * 2);
        client.join(m, &token_id);
    }

    // All contribute cycle 1
    for m in &members {
        client.contribute(m, &token_id);
    }

    let recipient = client.get_state().payout_order.get(0).unwrap();
    let token_client = token::Client::new(&env, &token_id);
    let before = token_client.balance(&recipient);

    client.try_payout(&token_id);

    let after = token_client.balance(&recipient);
    assert_eq!(after - before, contribution * (n as i128));
}

// ─── payout order / schedule tests ──────────────────────────────────────────

#[test]
fn test_payout_schedule_is_permutation_of_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 4, 10_000_000, 0);

    let members: std::vec::Vec<Address> = (0..4).map(|_| Address::generate(&env)).collect();
    for m in &members {
        client.join(m, &token_id);
    }

    let schedule = client.get_payout_schedule();
    assert_eq!(schedule.len(), 4);

    // Positions must be 1..4 with no duplicates
    let mut positions: std::vec::Vec<u32> = schedule.iter().map(|e| e.payout_position).collect();
    positions.sort();
    assert_eq!(positions, std::vec![1, 2, 3, 4]);
}

#[test]
fn test_fixed_payout_order_matches_join_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 3, 10_000_000, 0);

    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    let m3 = Address::generate(&env);
    client.join(&m1, &token_id);
    client.join(&m2, &token_id);
    client.join(&m3, &token_id);

    let state = client.get_state();
    assert_eq!(state.payout_order.get(0).unwrap(), m1);
    assert_eq!(state.payout_order.get(1).unwrap(), m2);
    assert_eq!(state.payout_order.get(2).unwrap(), m3);
}

// ─── claim_deposit tests ─────────────────────────────────────────────────────

#[test]
fn test_claim_deposit_after_completion() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, admin) = create_token(&env);
    let (contract_id, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let deposit = 5_000_000_i128;
    let contribution = 10_000_000_i128;
    setup_circle(&env, &client, &creator, &token_id, 2, contribution, deposit);

    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    mint(&env, &token_id, &admin, &m1, deposit + contribution * 2);
    mint(&env, &token_id, &admin, &m2, deposit + contribution * 2);

    client.join(&m1, &token_id);
    client.join(&m2, &token_id);

    // Cycle 1
    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);

    // Cycle 2
    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);

    let state = client.get_state();
    assert_eq!(state.status, CircleStatus::Completed);

    let token_client = token::Client::new(&env, &token_id);
    let before = token_client.balance(&m1);
    client.claim_deposit(&m1, &token_id);
    let after = token_client.balance(&m1);
    assert_eq!(after - before, deposit);
}

#[test]
fn test_claim_deposit_rejects_before_completion() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, admin) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let deposit = 5_000_000_i128;
    setup_circle(&env, &client, &creator, &token_id, 2, 10_000_000, deposit);

    let m1 = Address::generate(&env);
    mint(&env, &token_id, &admin, &m1, deposit);
    client.join(&m1, &token_id);

    let result = client.try_claim_deposit(&m1, &token_id);
    assert_eq!(result, Err(Ok(ContractError::NotCompleted)));
}

#[test]
fn test_claim_deposit_rejects_double_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, admin) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    let deposit = 5_000_000_i128;
    let contribution = 10_000_000_i128;
    setup_circle(&env, &client, &creator, &token_id, 2, contribution, deposit);

    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    mint(&env, &token_id, &admin, &m1, deposit + contribution * 2);
    mint(&env, &token_id, &admin, &m2, deposit + contribution * 2);

    client.join(&m1, &token_id);
    client.join(&m2, &token_id);

    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);
    client.contribute(&m1, &token_id);
    client.contribute(&m2, &token_id);
    client.try_payout(&token_id);

    client.claim_deposit(&m1, &token_id);
    let result = client.try_claim_deposit(&m1, &token_id);
    assert_eq!(result, Err(Ok(ContractError::AlreadyPaid)));
}

// ─── flag_defaults tests ─────────────────────────────────────────────────────

#[test]
fn test_flag_defaults_marks_unpaid_members() {
    let env = Env::default();
    let (client, token_id, _, m1, m2) = active_circle_2(&env);

    // Only m1 pays; advance past window
    client.contribute(&m1, &token_id);

    let state = client.get_state();
    env.ledger().set(LedgerInfo {
        timestamp: state.cycle_end_timestamp + 1,
        protocol_version: 20,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let organizer = Address::generate(&env);
    client.flag_defaults(&organizer);

    assert_eq!(client.get_member_status(&m2), MemberStatus::Defaulter);
    assert_eq!(client.get_member_status(&m1), MemberStatus::Active);
}

#[test]
fn test_flag_defaults_rejects_before_window_closes() {
    let env = Env::default();
    let (client, token_id, _, _, _) = active_circle_2(&env);

    let organizer = Address::generate(&env);
    let result = client.try_flag_defaults(&organizer);
    assert_eq!(result, Err(Ok(ContractError::WindowClosed)));
}

// ─── get_member_status tests ─────────────────────────────────────────────────

#[test]
fn test_get_member_status_not_member() {
    let env = Env::default();
    env.mock_all_auths();
    let (token_id, _) = create_token(&env);
    let (_, client) = deploy_contract(&env);
    let creator = Address::generate(&env);

    setup_circle(&env, &client, &creator, &token_id, 2, 10_000_000, 0);

    let outsider = Address::generate(&env);
    assert_eq!(client.get_member_status(&outsider), MemberStatus::NotMember);
}
