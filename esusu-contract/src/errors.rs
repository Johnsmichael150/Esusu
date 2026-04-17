use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    AlreadyMember = 2,
    CircleFull = 3,
    WrongAmount = 4,
    WindowClosed = 5,
    AlreadyPaid = 6,
    NotAllPaid = 7,
    CircleNotActive = 8,
    NotAMember = 9,
    InvalidConfig = 10,
    NotCompleted = 11,
    HasDefaults = 12,
}
