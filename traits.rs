use ink::{
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
};

use crate::errors::PSP22Error;
use crate::errors::OwnableError;

#[ink::trait_definition]
pub trait PSP22 {
    /// Returns the total token supply.
    #[ink(message)]
    fn total_supply(&self) -> u128;

    /// Returns the account balance for the specified `owner`.
    ///
    /// Returns `0` if the account is non-existent.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> u128;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    /// Returns `0` if no allowance has been set.
    #[ink(message)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> u128;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if the caller and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: u128, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// If `from` and the caller are different addresses, the caller must be allowed
    /// by `from` to spend at least `value` tokens.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if `from` and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// If `from` and the caller are different addresses, a successful transfer results
    /// in decreased allowance by `from` to the caller and an `Approval` event with
    /// the new allowance amount is emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account `from`.
    ///
    /// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses and
    /// the `value` exceeds the allowance granted by `from` to the caller.
    ///
    /// If conditions for both `InsufficientBalance` and `InsufficientAllowance` errors are met,
    /// reverts with `InsufficientAllowance`.
    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the total amount of `value`.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Events
    ///
    /// An `Approval` event is emitted.
    ///
    /// No-op if the caller and `spender` is the same address, returns success and no events are emitted.
    #[ink(message)]
    fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error>;

    /// Increases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    #[ink(message)]
    fn increase_allowance(
        &mut self,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;

    /// Decreases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientAllowance` if `spender` and the caller are different addresses and
    /// the `delta-value` exceeds the allowance granted by the caller to `spender`.
    #[ink(message)]
    fn decrease_allowance(
        &mut self,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait PSP22Metadata {
    /// Returns the token name.
    #[ink(message)]
    fn token_name(&self) -> Option<String>;
    /// Returns the token symbol.
    #[ink(message)]
    fn token_symbol(&self) -> Option<String>;
    /// Returns the token decimals.
    #[ink(message)]
    fn token_decimals(&self) -> u8;
}

#[ink::trait_definition]
pub trait PSP22Burnable {
    /// Burns `value` tokens from the senders account.
    ///
    /// The selector for this message is `0x7a9da510` (first 4 bytes of `blake2b_256("PSP22Burnable::burn")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` recipient.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn burn(&mut self, value: u128) -> Result<(), PSP22Error>;

    /// Burns `value` tokens from the "account" account id. Spends allowances.
    ///
    /// The selector for this message are
    /// first 4 bytes of `blake2b_256("PSP22Burnable::burn_from")`
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` recipient.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn burn_from(&mut self, account: AccountId, value: u128) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait PSP22Mintable {
    /// Mints `value` tokens to the senders account.
    ///
    /// The selector for this message is `0xfc3c75d4` (first 4 bytes of `blake2b_256("PSP22Mintable::mint")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` sender.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `Custom (max supply exceeded)` if the total supply increased by
    /// `value` exceeds maximal value of `u128` type.
    #[ink(message)]
    fn mint(&mut self, to: AccountId, value: u128) -> Result<(), PSP22Error>;
}


/// Trait for pausing and unpausing token transfers.
///
/// This trait allows the contract owner to pause or unpause token transfers,
/// which can be useful in emergency situations or during maintenance.
#[ink::trait_definition]
pub trait PSP22Pausable {
    /// Pauses all token transfers.
    ///
    /// This method is used to temporarily halt all transfer operations.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating whether the operation was successful.
    #[ink(message)]
    fn pause(&mut self) -> Result<(), PSP22Error>;

    /// Unpauses all token transfers.
    ///
    /// This method re-enables token transfer operations.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating whether the operation was successful.
    #[ink(message)]
    fn unpause(&mut self) -> Result<(), PSP22Error>;
}

/// Trait for wrapping and unwrapping PSP22 tokens.
///
/// This trait provides methods for depositing and withdrawing tokens,
/// often used in implementations that wrap other token standards.
#[ink::trait_definition]
pub trait PSP22Wrapper {
    /// Deposits tokens into the contract for a specified account.
    ///
    /// This method allows a user to add tokens to the contract, which can be used
    /// for various functionalities like staking or liquidity provision.
    ///
    /// # Arguments
    ///
    /// * `account` - The account for which the tokens will be deposited.
    /// * `amount` - The amount of tokens to deposit.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating the success or failure of the operation.
    #[ink(message)]
    fn deposit_for(&mut self, account: AccountId, amount: u128) -> Result<(), PSP22Error>;

    /// Withdraws tokens from the contract to a specified account.
    ///
    /// This method allows users to withdraw their tokens from the contract.
    ///
    /// # Arguments
    ///
    /// * `account` - The account to which the tokens will be withdrawn.
    /// * `amount` - The amount of tokens to withdraw.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating the success or failure of the operation.
    #[ink(message)]
    fn withdraw_to(&mut self, account: AccountId, amount: u128) -> Result<(), PSP22Error>;
}

/// Trait for ownership-related functionalities.
///
/// Provides methods for managing ownership of the contract, including
/// transferring and renouncing ownership.
#[ink::trait_definition]
pub trait Ownable {
    /// Returns the address of the current owner.
    ///
    /// # Returns
    ///
    /// The `AccountId` of the current owner.
    #[ink(message)]
    fn owner(&self) -> Option<AccountId>;

    /// Renounces ownership of the contract.
    ///
    /// This method is used to permanently transfer control of the contract
    /// away from the current owner, leaving it without an owner.
    ///
    /// # Returns
    ///
    /// A `Result<(), OwnableError>` indicating whether the operation was successful.
    #[ink(message)]
    fn renounce_ownership(&mut self) -> Result<(), OwnableError>;

    /// Transfers ownership of the contract to a new account.
    ///
    /// # Arguments
    ///
    /// * `new_owner` - The `AccountId` of the new owner.
    ///
    /// # Returns
    ///
    /// A `Result<(), OwnableError>` indicating whether the operation was successful.
    #[ink(message)]
    fn transfer_ownership(&mut self, new_owner: Option<AccountId>) -> Result<(), OwnableError>;
}
