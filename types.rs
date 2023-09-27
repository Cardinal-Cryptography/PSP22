use ink::{prelude::string::String, primitives::AccountId, storage::Mapping};

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP22Error {
    /// Custom error type for cases in which an implementation adds its own restrictions.
    Custom(String),
    /// Returned if not enough Balance to fulfill a request is available.
    InsufficientBalance,
    /// Returned if not enough allowance to fulfill a request is available.
    InsufficientAllowance,
    /// Returned if recipient's address is zero.
    ZeroRecipientAddress,
    /// Returned if sender's address is zero.
    ZeroSenderAddress,
    /// Returned if a safe transfer check fails (e.g. if the receiving contract does not accept tokens).
    SafeTransferCheckFailed(String),
}

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
