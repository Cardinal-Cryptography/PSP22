use ink::{prelude::string::String, primitives::AccountId, storage::Mapping};

pub type Balance = u128;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP22Error {
    /// Custom error type for cases in which an implementation adds its own restrictions.
    Custom(String),
    /// Returned if not enough balance to fulfill a request is available.
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
    pub total_supply: Balance,
    pub balances: Mapping<AccountId, Balance>,
    pub allowances: Mapping<(AccountId, AccountId), Balance>,
}

impl PSP22Data {
    pub fn new(supply: Balance, creator: AccountId) -> PSP22Data {
        let mut data = PSP22Data {
            total_supply: supply,
            balances: Default::default(),
            allowances: Default::default(),
        };
        data.balances.insert(creator, &supply);
        data
    }
}
