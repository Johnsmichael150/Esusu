use soroban_sdk::{
    contract, contractimpl, symbol_short, token, Address, Env, Vec,
};

use crate::errors::ContractError;
use crate::storage::{has_state, DataKey};
use crate::types::{CircleState, CircleStatus, MemberStatus, PayoutEntry, PayoutOrder};

#[contract]
pub struct EsusuContract;

#[contractimpl]
impl EsusuContract {
    // -------------------------------------------------------------------------
    // 2.2 initialize
    // -------------------------------------------------------------------------
    /// Deploy and configure a new savings circle.
    pub fn initialize(
        env: Env,
        creator: Address,
        contribution_amount: i128,
        cycle_length_days: u32,
        max_members: u32,
        deposit_amount: i128,
        payout_order_mode: PayoutOrder,
    ) -> Result<(), ContractError> {
        creator.require_auth();

        if has_state(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if contribution_amount <= 0 {
            return Err(ContractError::InvalidConfig);
        }
        if max_members < 2 || max_members > 50 {
            return Err(ContractError::InvalidConfig);
        }
        if cycle_length_days < 1 {
            return Err(ContractError::InvalidConfig);
        }
        if deposit_amount < 0 {
            return Err(ContractError::InvalidConfig);
        }

        let state = CircleState {
            status: CircleStatus::Pending,
            creator,
            current_cycle: 0,
            total_cycles: max_members, // one cycle per member
            cycle_end_timestamp: 0,
            contribution_amount,
            deposit_amount,
            max_members,
            payout_order_mode,
            members: Vec::new(&env),
            payout_order: Vec::new(&env),
            defaulters: Vec::new(&env),
        };

        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2.3 join  +  2.4 payout order finalization
    // -------------------------------------------------------------------------
    /// Member joins the circle and stakes the deposit via USDC SAC transfer.
    /// Returns the member's 1-indexed payout position (fixed mode) or 0 (randomized, TBD at activation).
    pub fn join(env: Env, member: Address, usdc_token: Address) -> Result<u32, ContractError> {
        member.require_auth();

        let mut state: CircleState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .ok_or(ContractError::CircleNotActive)?;

        if state.status != CircleStatus::Pending {
            return Err(ContractError::CircleNotActive);
        }
        if state.members.contains(&member) {
            return Err(ContractError::AlreadyMember);
        }
        if state.members.len() >= state.max_members {
            return Err(ContractError::CircleFull);
        }

        // Require deposit transfer from member to this contract (USDC SAC)
        if state.deposit_amount > 0 {
            let token_client = token::Client::new(&env, &usdc_token);
            token_client.transfer(&member, &env.current_contract_address(), &state.deposit_amount);
        }

        state.members.push_back(member.clone());
        let position = state.members.len(); // 1-indexed, valid for Fixed mode

        // When the final member joins, activate the circle
        if state.members.len() == state.max_members {
            Self::activate_circle(&env, &mut state);
        }

        env.storage().instance().set(&DataKey::State, &state);
        Ok(position)
    }

    /// Internal: finalize payout order and open the first Cycle_Window.
    fn activate_circle(env: &Env, state: &mut CircleState) {
        state.status = CircleStatus::Active;
        state.current_cycle = 1;

        let cycle_secs = (state.total_cycles as u64) * 86_400;
        state.cycle_end_timestamp = env.ledger().timestamp() + cycle_secs;

        // 2.4 Payout order finalization
        match state.payout_order_mode {
            PayoutOrder::Fixed => {
                // Positions already assigned in join order — members vec IS the order
                state.payout_order = state.members.clone();
            }
            PayoutOrder::Randomized => {
                // Fisher-Yates shuffle using on-chain entropy (ledger sequence)
                let mut order = state.members.clone();
                let seed = env.ledger().sequence() as u64;
                let len = order.len() as u64;
                for i in (1..len).rev() {
                    // Simple LCG-derived index — deterministic, unpredictable pre-activation
                    let j = (seed.wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407)
                        .wrapping_add(i)) % (i + 1);
                    let a = order.get(i as u32).unwrap();
                    let b = order.get(j as u32).unwrap();
                    order.set(i as u32, b);
                    order.set(j as u32, a);
                }
                state.payout_order = order;
            }
        }

        // Emit event with finalized payout order
        env.events().publish(
            (symbol_short!("activated"),),
            state.payout_order.clone(),
        );
    }

    // -------------------------------------------------------------------------
    // 2.5 contribute
    // -------------------------------------------------------------------------
    /// Member contributes the exact contribution amount for the current cycle.
    pub fn contribute(
        env: Env,
        member: Address,
        usdc_token: Address,
    ) -> Result<(), ContractError> {
        member.require_auth();

        let state: CircleState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .ok_or(ContractError::CircleNotActive)?;

        if state.status != CircleStatus::Active {
            return Err(ContractError::CircleNotActive);
        }
        if !state.members.contains(&member) {
            return Err(ContractError::NotAMember);
        }
        if env.ledger().timestamp() > state.cycle_end_timestamp {
            return Err(ContractError::WindowClosed);
        }

        let contrib_key = DataKey::Contribution(member.clone(), state.current_cycle);
        if env.storage().instance().has(&contrib_key) {
            return Err(ContractError::AlreadyPaid);
        }

        // Transfer exact contribution amount from member to contract
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(
            &member,
            &env.current_contract_address(),
            &state.contribution_amount,
        );

        env.storage().instance().set(&contrib_key, &true);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2.6 try_payout
    // -------------------------------------------------------------------------
    /// Check if all members have contributed; if so, execute payout to current cycle's recipient.
    pub fn try_payout(env: Env, usdc_token: Address) -> Result<bool, ContractError> {
        let mut state: CircleState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .ok_or(ContractError::CircleNotActive)?;

        if state.status != CircleStatus::Active {
            return Err(ContractError::CircleNotActive);
        }

        // Check all members have paid this cycle
        for member in state.members.iter() {
            let key = DataKey::Contribution(member.clone(), state.current_cycle);
            if !env.storage().instance().has(&key) {
                return Ok(false); // NotAllPaid — no payout yet
            }
        }

        // Determine recipient: payout_order index is (current_cycle - 1)
        let recipient_idx = (state.current_cycle - 1) as u32;
        let recipient = state
            .payout_order
            .get(recipient_idx)
            .ok_or(ContractError::CircleNotActive)?;

        let payout_amount = state.contribution_amount * (state.members.len() as i128);

        // Transfer pooled USDC to recipient
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &payout_amount,
        );

        // Emit payout event
        env.events().publish(
            (symbol_short!("payout"),),
            (state.current_cycle, recipient.clone(), payout_amount),
        );

        let completed_cycle = state.current_cycle;

        // Advance cycle or complete circle
        if completed_cycle >= state.total_cycles {
            state.status = CircleStatus::Completed;
        } else {
            state.current_cycle += 1;
            let cycle_secs = (state.total_cycles as u64) * 86_400;
            state.cycle_end_timestamp = env.ledger().timestamp() + cycle_secs;
        }

        env.storage().instance().set(&DataKey::State, &state);
        Ok(true)
    }

    // -------------------------------------------------------------------------
    // 2.7 Default flagging
    // -------------------------------------------------------------------------
    /// Called (by organizer or backend) after Cycle_Window expires to flag unpaid members as defaulters.
    pub fn flag_defaults(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let mut state: CircleState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .ok_or(ContractError::CircleNotActive)?;

        if state.status != CircleStatus::Active {
            return Err(ContractError::CircleNotActive);
        }
        // Only callable after the window has closed
        if env.ledger().timestamp() <= state.cycle_end_timestamp {
            return Err(ContractError::WindowClosed);
        }

        for member in state.members.iter() {
            let key = DataKey::Contribution(member.clone(), state.current_cycle);
            if !env.storage().instance().has(&key) {
                // Mark as defaulter if not already flagged
                if !state.defaulters.contains(&member) {
                    state.defaulters.push_back(member.clone());
                    env.events().publish(
                        (symbol_short!("default"),),
                        member.clone(),
                    );
                }
            }
        }

        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2.8 claim_deposit
    // -------------------------------------------------------------------------
    /// After circle completes, a non-defaulting member can reclaim their staked deposit.
    pub fn claim_deposit(env: Env, member: Address, usdc_token: Address) -> Result<(), ContractError> {
        member.require_auth();

        let state: CircleState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .ok_or(ContractError::CircleNotActive)?;

        if state.status != CircleStatus::Completed {
            return Err(ContractError::NotCompleted);
        }
        if !state.members.contains(&member) {
            return Err(ContractError::NotAMember);
        }
        if state.defaulters.contains(&member) {
            return Err(ContractError::HasDefaults);
        }

        // Prevent double-claim
        let claimed_key = DataKey::DepositClaimed(member.clone());
        if env.storage().instance().has(&claimed_key) {
            return Err(ContractError::AlreadyPaid);
        }

        if state.deposit_amount > 0 {
            let token_client = token::Client::new(&env, &usdc_token);
            token_client.transfer(
                &env.current_contract_address(),
                &member,
                &state.deposit_amount,
            );
        }

        env.storage().instance().set(&claimed_key, &true);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2.9 Read-only query functions
    // -------------------------------------------------------------------------

    /// Returns the full circle state.
    pub fn get_state(env: Env) -> CircleState {
        env.storage()
            .instance()
            .get(&DataKey::State)
            .unwrap()
    }

    /// Returns the status of a specific member.
    pub fn get_member_status(env: Env, member: Address) -> MemberStatus {
        let state: CircleState = env.storage().instance().get(&DataKey::State).unwrap();
        if !state.members.contains(&member) {
            return MemberStatus::NotMember;
        }
        if state.defaulters.contains(&member) {
            MemberStatus::Defaulter
        } else {
            MemberStatus::Active
        }
    }

    /// Returns the ordered payout schedule (position 1..N with cycle numbers).
    pub fn get_payout_schedule(env: Env) -> Vec<PayoutEntry> {
        let state: CircleState = env.storage().instance().get(&DataKey::State).unwrap();
        let mut schedule = Vec::new(&env);
        for (i, member) in state.payout_order.iter().enumerate() {
            schedule.push_back(PayoutEntry {
                member,
                payout_position: (i + 1) as u32,
                cycle: (i + 1) as u32,
            });
        }
        schedule
    }
}
