use ink::{primitives::AccountId, storage::Mapping};

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PSP22Data {
    pub total_supply: u128,
    pub balances: Mapping<AccountId, u128>,
    pub allowances: Mapping<(AccountId, AccountId), u128>,
}

impl PSP22Data {
    pub fn new(supply: u128, creator: AccountId) -> PSP22Data {
        let mut data = PSP22Data {
            total_supply: supply,
            balances: Default::default(),
            allowances: Default::default(),
        };
        data.balances.insert(creator, &supply);
        data
    }
}
