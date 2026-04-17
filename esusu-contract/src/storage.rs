use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    State,
    Contribution(Address, u32), // (member, cycle)
    DepositClaimed(Address),    // member -> bool
}

pub fn has_state(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::State)
}
