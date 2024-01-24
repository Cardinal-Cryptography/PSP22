use ink::primitives::AccountId;

#[ink::event]
pub struct Approval {
    #[ink(topic)]
    pub owner: AccountId,
    #[ink(topic)]
    pub spender: AccountId,
    pub amount: u128,
}

#[ink::event]
pub struct Transfer {
    #[ink(topic)]
    pub from: Option<AccountId>,
    #[ink(topic)]
    pub to: Option<AccountId>,
    pub value: u128,
}
