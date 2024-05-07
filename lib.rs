#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod events;
mod testing;
mod traits;

pub use data::{PSP22Data, PSP22Event};
pub use errors::PSP22Error;
pub use events::{Approval, Transfer};
pub use traits::{PSP22Burnable, PSP22Metadata, PSP22Mintable, PSP22};

// An example code of a smart contract using PSP22Data struct to implement
// the functionality of PSP22 fungible token.
//
// Any contract can be easily enriched to act as PSP22 token by:
// (1) adding PSP22Data to contract storage
// (2) properly initializing it
// (3) implementing PSP22 trait based on PSP22Data methods
// (4) properly emitting resulting events
//
// It is a good practice to also implement the optional PSP22Metadata extension (5)
// and include unit tests (6).
#[ink::contract]
mod token {
    use crate::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct Token {
        data: PSP22Data, // (1)
        name: Option<String>,
        symbol: Option<String>,
        decimals: u8,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(
            supply: u128,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
        ) -> Self {
            let (data, events) = PSP22Data::new(supply, Self::env().caller()); // (2)
            let contract = Self {
                data,
                name,
                symbol,
                decimals,
            };
            contract.emit_events(events);
            contract
        }

        // A helper function emitting events contained in a vector of PSP22Events.
        // (4)
        fn emit_events(&self, events: Vec<PSP22Event>) {
            for event in events {
                match event {
                    PSP22Event::Transfer(e) => self.env().emit_event(e),
                    PSP22Event::Approval(e) => self.env().emit_event(e),
                }
            }
        }
    }

    // (3)
    impl PSP22 for Token {
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balance_of(owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.data.allowance(owner, spender)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self.data.transfer(self.env().caller(), to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .transfer_from(self.env().caller(), from, to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let events = self.data.approve(self.env().caller(), spender, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .increase_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .decrease_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }
    }

    // (5)
    impl PSP22Metadata for Token {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            self.name.clone()
        }
        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            self.symbol.clone()
        }
        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            self.decimals
        }
    }

    // (6)
    #[cfg(test)]
    mod tests {
        use super::Token;
        crate::tests!(Token, (|supply| Token::new(supply, None, None, 0)));
    }
}
