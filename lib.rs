#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod traits;

pub use data::PSP22Data;
pub use errors::PSP22Error;
pub use traits::PSP22;

#[cfg(feature = "contract")]
#[ink::contract]
mod token {
    use crate::data::PSP22Data;
    use crate::errors::PSP22Error;
    use crate::traits::PSP22;

    #[ink(storage)]
    pub struct Token {
        data: PSP22Data,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(supply: u128) -> Self {
            Self {
                data: PSP22Data::new(supply, Self::env().caller()),
            }
        }
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: u128,
    }

    impl PSP22 for Token {
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.data.total_supply
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.data
                .allowances
                .get((owner, spender))
                .unwrap_or_default()
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: ink::prelude::vec::Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from = self.env().caller();
            if from == to || value == 0 {
                return Ok(());
            }
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            if from_balance == value {
                self.data.balances.remove(from);
            } else {
                self.data
                    .balances
                    .insert(from, &(from_balance.saturating_sub(value)));
            }
            let to_balance = self.balance_of(to);
            // Total supply is limited by u128.MAX so no overflow is possible
            self.data
                .balances
                .insert(to, &(to_balance.saturating_add(value)));
            self.env().emit_event(Transfer { from, to, value });
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            data: ink::prelude::vec::Vec<u8>,
        ) -> Result<(), PSP22Error> {
            if from == to || value == 0 {
                return Ok(());
            }
            let caller = self.env().caller();
            if caller == from {
                return self.transfer(to, value, data);
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
                self.data.allowances.remove((from, caller));
            } else {
                self.data
                    .allowances
                    .insert((from, caller), &(allowance.saturating_sub(value)));
            }

            if from_balance == value {
                self.data.balances.remove(from);
            } else {
                self.data
                    .balances
                    .insert(from, &(from_balance.saturating_sub(value)));
            }
            let to_balance = self.balance_of(to);
            // Total supply is limited by u128.MAX so no overflow is possible
            self.data
                .balances
                .insert(to, &(to_balance.saturating_add(value)));
            self.env().emit_event(Approval {
                owner: from,
                spender: caller,
                amount: allowance.saturating_sub(value),
            });
            self.env().emit_event(Transfer { from, to, value });
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender {
                return Ok(());
            }
            if value == 0 {
                self.data.allowances.remove((owner, spender));
            } else {
                self.data.allowances.insert((owner, spender), &value);
            }
            self.env().emit_event(Approval {
                owner,
                spender,
                amount: value,
            });
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender || delta_value == 0 {
                return Ok(());
            }
            let allowance = self.allowance(owner, spender);
            let amount = allowance.saturating_add(delta_value);
            self.data.allowances.insert((owner, spender), &amount);
            self.env().emit_event(Approval {
                owner,
                spender,
                amount,
            });
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            if owner == spender || delta_value == 0 {
                return Ok(());
            }
            let allowance = self.allowance(owner, spender);
            if allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            let amount = allowance.saturating_sub(delta_value);
            if amount == 0 {
                self.data.allowances.remove((owner, spender));
            } else {
                self.data.allowances.insert((owner, spender), &amount);
            }
            self.env().emit_event(Approval {
                owner,
                spender,
                amount,
            });
            Ok(())
        }
    }
}
