use crate::errors::PSP22Error;
use crate::events::{Approval, Transfer};
use ink::prelude::string::String;
use ink::{
    prelude::{vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};
use ink::env::call::{build_call, ExecutionInput, Selector};
use ink::env::DefaultEnvironment;

/// Common wrapper type for events emitted during operations that change the
/// state of PSP22Data struct.
pub enum PSP22Event {
    Transfer(Transfer),
    Approval(Approval),
}

// Shortcut for Approval PSP22Event constructor.
fn approval_event(owner: AccountId, spender: AccountId, amount: u128) -> PSP22Event {
    PSP22Event::Approval(Approval {
        owner,
        spender,
        amount,
    })
}

// Shortcut for Transfer PSP22Event constructor.
fn transfer_event(from: Option<AccountId>, to: Option<AccountId>, value: u128) -> PSP22Event {
    PSP22Event::Transfer(Transfer { from, to, value })
}

/// A class implementing the internal logic of a PSP22 token.
//
/// Holds the state of all account balances and allowances.
/// Each method of this class corresponds to one type of transaction
/// as defined in the PSP22 standard.
//
/// Since this code is outside of `ink::contract` macro, the caller's
/// address cannot be obtained automatically. Because of that, all
/// the methods that need to know the caller require an additional argument
/// (compared to transactions defined by the PSP22 standard or the PSP22 trait).
//
/// `lib.rs` contains an example implementation of a smart contract using this class.
#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PSP22Data {
    total_supply: u128,
    balances: Mapping<AccountId, u128>,
    allowances: Mapping<(AccountId, AccountId), u128>,
}

impl PSP22Data {
    /// Creates a token with `supply` balance, initially held by the `creator` account.
    pub fn new(supply: u128, creator: AccountId) -> (PSP22Data, Vec<PSP22Event>) {
        let mut data: PSP22Data = Default::default();
        let events = data.mint(creator, supply).unwrap();
        (data, events)
    }

    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    pub fn balance_of(&self, owner: AccountId) -> u128 {
        self.balances.get(owner).unwrap_or_default()
    }

    pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
        self.allowances.get((owner, spender)).unwrap_or_default()
    }

    /// Transfers `value` tokens from `caller` to `to`.
    pub fn transfer(
        &mut self,
        caller: AccountId,
        to: AccountId,
        value: u128,
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if caller == to || value == 0 {
            return Ok(vec![]);
        }
        let from_balance = self.balance_of(caller);
        if from_balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }

        if from_balance == value {
            self.balances.remove(caller);
        } else {
            self.balances
                .insert(caller, &(from_balance.saturating_sub(value)));
        }
        let to_balance = self.balance_of(to);
        // Total supply is limited by u128.MAX so no overflow is possible
        self.balances
            .insert(to, &(to_balance.saturating_add(value)));
        Ok(vec![transfer_event(Some(caller), Some(to), value)])
    }

    /// Transfers `value` tokens from `from` to `to`, but using the allowance
    /// granted be `from` to `caller.
    pub fn transfer_from(
        &mut self,
        caller: AccountId,
        from: AccountId,
        to: AccountId,
        value: u128,
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if from == to || value == 0 {
            return Ok(vec![]);
        }
        if caller == from {
            return self.transfer(caller, to, value);
        }

        let allowance = self.allowance(from, caller);
        if allowance < value {
            return Err(PSP22Error::InsufficientAllowance);
        }
        let from_balance = self.balance_of(from);
        if from_balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }

        if allowance == value {
            self.allowances.remove((from, caller));
        } else {
            self.allowances
                .insert((from, caller), &(allowance.saturating_sub(value)));
        }

        if from_balance == value {
            self.balances.remove(from);
        } else {
            self.balances
                .insert(from, &(from_balance.saturating_sub(value)));
        }
        let to_balance = self.balance_of(to);
        // Total supply is limited by u128.MAX so no overflow is possible
        self.balances
            .insert(to, &(to_balance.saturating_add(value)));
        Ok(vec![
            approval_event(from, caller, allowance.saturating_sub(value)),
            transfer_event(Some(from), Some(to), value),
        ])
    }

    /// Sets a new `value` for allowance granted by `owner` to `spender`.
    /// Overwrites the previously granted value.
    pub fn approve(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        value: u128,
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if owner == spender {
            return Ok(vec![]);
        }
        if value == 0 {
            self.allowances.remove((owner, spender));
        } else {
            self.allowances.insert((owner, spender), &value);
        }
        Ok(vec![approval_event(owner, spender, value)])
    }

    /// Increases the allowance granted  by `owner` to `spender` by `delta_value`.
    pub fn increase_allowance(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if owner == spender || delta_value == 0 {
            return Ok(vec![]);
        }
        let allowance = self.allowance(owner, spender);
        let amount = allowance.saturating_add(delta_value);
        self.allowances.insert((owner, spender), &amount);
        Ok(vec![approval_event(owner, spender, amount)])
    }

    /// Decreases the allowance granted  by `owner` to `spender` by `delta_value`.
    pub fn decrease_allowance(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if owner == spender || delta_value == 0 {
            return Ok(vec![]);
        }
        let allowance = self.allowance(owner, spender);
        if allowance < delta_value {
            return Err(PSP22Error::InsufficientAllowance);
        }
        let amount = allowance.saturating_sub(delta_value);
        if amount == 0 {
            self.allowances.remove((owner, spender));
        } else {
            self.allowances.insert((owner, spender), &amount);
        }
        Ok(vec![approval_event(owner, spender, amount)])
    }

    /// Mints a `value` of new tokens to `to` account.
    pub fn mint(&mut self, to: AccountId, value: u128) -> Result<Vec<PSP22Event>, PSP22Error> {
        if value == 0 {
            return Ok(vec![]);
        }
        let new_supply = self
            .total_supply
            .checked_add(value)
            .ok_or(PSP22Error::Custom(String::from(
                "Max PSP22 supply exceeded. Max supply limited to 2^128-1.",
            )))?;
        self.total_supply = new_supply;
        let new_balance = self.balance_of(to).saturating_add(value);
        self.balances.insert(to, &new_balance);
        Ok(vec![transfer_event(None, Some(to), value)])
    }

    /// Burns `value` tokens from `from` account.
    pub fn burn(&mut self, from: AccountId, value: u128) -> Result<Vec<PSP22Event>, PSP22Error> {
        if value == 0 {
            return Ok(vec![]);
        }
        let balance = self.balance_of(from);
        if balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }
        if balance == value {
            self.balances.remove(from);
        } else {
            self.balances.insert(from, &(balance.saturating_sub(value)));
        }
        self.total_supply = self.total_supply.saturating_sub(value);
        Ok(vec![transfer_event(Some(from), None, value)])
    }

    /// Burns `value` tokens from `from` account.
    pub fn burn_from(&mut self,
                     caller: AccountId,
                     from: AccountId,
                     value: u128
    ) -> Result<Vec<PSP22Event>, PSP22Error> {
        if value == 0 {
            return Ok(vec![]);
        }
        let allowance = self.allowance(from, caller);
        if allowance < value {
            return Err(PSP22Error::InsufficientAllowance);
        }
        let balance = self.balance_of(from);
        if balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }

        if allowance == value {
            self.allowances.remove((from, caller));
        } else {
            self.allowances
                .insert((from, caller), &(allowance.saturating_sub(value)));
        }
        if balance == value {
            self.balances.remove(from);
        } else {
            self.balances.insert(from, &(balance.saturating_sub(value)));
        }
        self.total_supply = self.total_supply.saturating_sub(value);
        Ok(vec![PSP22Event::Transfer {
            from: Some(from),
            to: None,
            value,
        }])
    }

    /// Deposits a specified amount of tokens from the `underlying` token contract to this contract.
    ///
    /// This method transfers tokens from `sender` to the `contract` account (the current contract),
    /// using the `underlying` token's `transfer_from` method. It's typically used in wrapper implementations.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The AccountId of the underlying token contract.
    /// * `sender` - The AccountId of the sender who is depositing tokens.
    /// * `contract` - The AccountId of this contract, which will receive the tokens.
    /// * `value` - The amount of tokens to be deposited.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating the success or failure of the operation.
    pub fn deposit(&mut self,
                   underlying: AccountId,
                   sender: AccountId,
                   contract: AccountId,
                   value: u128
    ) -> Result<(), PSP22Error> {
        pub const TRANSFER_FROM_SELECTOR: [u8; 4] = [84, 179, 199, 110];

        build_call::<DefaultEnvironment>()
            .call(underlying)
            .gas_limit(0)
            .transferred_value(0)
            .exec_input(
                ExecutionInput::new(Selector::new(TRANSFER_FROM_SELECTOR))
                    .push_arg(sender)
                    .push_arg(contract)
                    .push_arg(value)
                    .push_arg(Vec::<u8>::new())
            )
            .returns::<Result<(), PSP22Error>>()
            .invoke()
    }

    /// Withdraws a specified amount of tokens from this contract to a specified account.
    ///
    /// This method transfers tokens from this contract to the `account` specified,
    /// using the `underlying` token's `transfer` method. It's typically used in wrapper implementations.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The AccountId of the underlying token contract.
    /// * `account` - The AccountId where tokens will be withdrawn to.
    /// * `value` - The amount of tokens to be withdrawn.
    ///
    /// # Returns
    ///
    /// A `Result<(), PSP22Error>` indicating the success or failure of the operation.
    pub fn withdraw(&mut self,
                    underlying: AccountId,
                    account: AccountId,
                    value: u128
    ) -> Result<(), PSP22Error> {
        pub const TRANSFER_SELECTOR: [u8; 4] = [219, 32, 249, 245];

        build_call::<DefaultEnvironment>()
            .call(underlying)
            .gas_limit(0)
            .transferred_value(0)
            .exec_input(
                ExecutionInput::new(Selector::new(TRANSFER_SELECTOR))
                    .push_arg(account)
                    .push_arg(value)
                    .push_arg(Vec::<u8>::new())
            )
            .returns::<Result<(), PSP22Error>>()
            .invoke()
    }

}
