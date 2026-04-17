use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CircleStatus {
    Pending,
    Active,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum MemberStatus {
    Active,
    Defaulter,
    NotMember,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum PayoutOrder {
    Fixed,
    Randomized,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PayoutEntry {
    pub member: Address,
    pub payout_position: u32,
    pub cycle: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CircleState {
    pub status: CircleStatus,
    pub creator: Address,
    pub current_cycle: u32,
    pub total_cycles: u32,
    pub cycle_end_timestamp: u64,
    pub contribution_amount: i128,
    pub deposit_amount: i128,
    pub max_members: u32,
    pub payout_order_mode: PayoutOrder,
    pub members: Vec<Address>,
    pub payout_order: Vec<Address>,
    pub defaulters: Vec<Address>,
}
