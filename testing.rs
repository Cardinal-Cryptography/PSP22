/// Inserts a suite of ink! unit tests intended for a contract implementing PSP22 trait.
/// `$contract` argument should be the name of the contract struct.
/// `$constructor` argument should be the name of a function, which initializes `$contract`
/// with the given total supply of tokens.
/// This macro should be invoked inside `#[ink::contract]` module.
#[macro_export]
macro_rules! tests {
    ($contract:ident, $constructor:expr) => {
        #[allow(clippy::redundant_closure_call)]
        mod psp22_unit_tests {
            use super::*;
            use ink::env::test::*;
            use ink::env::DefaultEnvironment as E;
            use ink::primitives::AccountId;
            use $crate::{Approval, PSP22Error, Transfer, PSP22};

            // Gathers all emitted events, skip `shift` first, and return as a vector.
            fn get_events(shift: usize) -> Vec<EmittedEvent> {
                recorded_events().skip(shift).collect()
            }

            // Checks if the given event is a Transfer
            fn is_transfer(event: &EmittedEvent) -> bool {
                <Transfer as ink::scale::Decode>::decode(&mut &event.data[..]).is_ok()
            }

            // Asserts if the given event is a Transfer with particular from_, to_ and value_
            fn assert_transfer(
                event: &EmittedEvent,
                from: Option<AccountId>,
                to: Option<AccountId>,
                value: u128,
            ) {
                let e = <Transfer as ink::scale::Decode>::decode(&mut &event.data[..])
                    .expect("Event is not Transfer");
                assert_eq!(e.from, from, "Transfer event: 'from' mismatch");
                assert_eq!(e.to, to, "Transfer event: 'to' mismatch");
                assert_eq!(e.value, value, "Transfer event: 'value' mismatch");
            }

            // Asserts if the given event is a Approval with particular owner_, spender_ and amount_
            fn assert_approval(
                event: &EmittedEvent,
                owner: AccountId,
                spender: AccountId,
                amount: u128,
            ) {
                let e = <Approval as ink::scale::Decode>::decode(&mut &event.data[..])
                    .expect("Event is not Approval");
                assert_eq!(e.owner, owner, "Approval event: 'owner' mismatch");
                assert_eq!(e.spender, spender, "Approval event: 'spender' mismatch");
                assert_eq!(e.amount, amount, "Approval event: 'amount' mismatch");
            }

            #[ink::test]
            fn constructor_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 1000;
                let token = $constructor(supply);

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);
                assert_eq!(token.allowance(acc.alice, acc.alice), 0);
                assert_eq!(token.allowance(acc.alice, acc.bob), 0);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);
            }

            #[ink::test]
            fn transfer_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply - value);
                assert_eq!(token.balance_of(acc.bob), value);
            }

            #[ink::test]
            fn double_transfer_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                assert!(token.transfer(acc.bob, 2 * value, vec![]).is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply - 3 * value);
                assert_eq!(token.balance_of(acc.bob), 3 * value);
            }

            #[ink::test]
            fn transfer_back_and_forth_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.transfer(acc.alice, value, vec![]).is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);
            }

            #[ink::test]
            fn transfer_cycle_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 2137;
                let mut token = $constructor(supply);

                assert!(token.transfer(acc.bob, supply, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.transfer(acc.charlie, supply, vec![]).is_ok());
                set_caller::<E>(acc.charlie);
                assert!(token.transfer(acc.alice, supply, vec![]).is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);
                assert_eq!(token.balance_of(acc.charlie), 0);
            }

            #[ink::test]
            fn transfer_emits_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_transfer(&events[0], Some(acc.alice), Some(acc.bob), value);
            }

            #[ink::test]
            fn constructor_emits_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 1000;
                let start = recorded_events().count();
                $constructor(supply);

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_transfer(&events[0], None, Some(acc.alice), supply);
            }

            #[ink::test]
            fn constructor_with_0_supply_emits_no_events() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let start = recorded_events().count();
                $constructor(0);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn multiple_transfers_emit_correct_events() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                assert!(token.transfer(acc.bob, 2 * value, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.transfer(acc.charlie, 3 * value, vec![]).is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 3);
                assert_transfer(&events[0], Some(acc.alice), Some(acc.bob), value);
                assert_transfer(&events[1], Some(acc.alice), Some(acc.bob), 2 * value);
                assert_transfer(&events[2], Some(acc.bob), Some(acc.charlie), 3 * value);
            }

            #[ink::test]
            fn transfer_0_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 0);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                let events = get_events(start);
                assert_eq!(events.len(), 0, "Transferring 0 tokens emitted event");
            }

            #[ink::test]
            fn transfer_from_empty_account_fails() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                set_caller::<E>(acc.bob);
                assert_eq!(
                    token.transfer(acc.charlie, value, vec![]),
                    Err(PSP22Error::InsufficientBalance)
                );
            }

            #[ink::test]
            fn insufficient_balance_transfer_fails() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert_eq!(
                    token.transfer(acc.charlie, value + 1, vec![]),
                    Err(PSP22Error::InsufficientBalance)
                );
            }

            #[ink::test]
            fn failed_transfer_does_not_emit_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 1000;
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert_eq!(
                    token.transfer(acc.bob, supply + 1, vec![]),
                    Err(PSP22Error::InsufficientBalance)
                );
                let events = get_events(start);
                assert_eq!(events.len(), 0)
            }

            #[ink::test]
            fn approve_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.approve(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);
            }

            #[ink::test]
            fn approve_a_lot_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100000);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_approval(&events[0], acc.alice, acc.bob, value);

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply);
                assert_eq!(token.balance_of(acc.bob), 0);
            }

            #[ink::test]
            fn approve_emits_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_approval(&events[0], acc.alice, acc.bob, value);
            }

            #[ink::test]
            fn multiple_approves_work_and_emit_correct_events() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert!(token.approve(acc.charlie, 2 * value).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.approve(acc.alice, 3 * value).is_ok());

                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert_eq!(token.allowance(acc.alice, acc.charlie), 2 * value);
                assert_eq!(token.allowance(acc.bob, acc.alice), 3 * value);

                set_caller::<E>(acc.alice);
                assert!(token.approve(acc.bob, 4 * value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), 4 * value);

                let events = get_events(start);
                assert_eq!(events.len(), 4);
                assert_approval(&events[0], acc.alice, acc.bob, value);
                assert_approval(&events[1], acc.alice, acc.charlie, 2 * value);
                assert_approval(&events[2], acc.bob, acc.alice, 3 * value);
                assert_approval(&events[3], acc.alice, acc.bob, 4 * value);
            }

            #[ink::test]
            fn approve_to_self_is_no_op() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.alice, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.alice), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn increase_allowance_works_and_emits_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert!(token.increase_allowance(acc.bob, supply).is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 2);
                assert_approval(&events[0], acc.alice, acc.bob, value);
                assert_approval(&events[1], acc.alice, acc.bob, value + supply);
            }

            #[ink::test]
            fn decrease_allowance_works_and_emits_event() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, 2 * value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), 2 * value);
                assert!(token.decrease_allowance(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert!(token.decrease_allowance(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 3);
                assert_approval(&events[0], acc.alice, acc.bob, 2 * value);
                assert_approval(&events[1], acc.alice, acc.bob, value);
                assert_approval(&events[2], acc.alice, acc.bob, 0);
            }

            #[ink::test]
            fn decrease_allowance_too_much_fails() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert_eq!(
                    token.decrease_allowance(acc.bob, 2 * value),
                    Err(PSP22Error::InsufficientAllowance)
                );

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_approval(&events[0], acc.alice, acc.bob, value);
            }

            #[ink::test]
            fn increase_and_decrease_allowance_by_0_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.approve(acc.bob, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), value);
                assert!(token.increase_allowance(acc.bob, 0).is_ok());
                assert!(token.decrease_allowance(acc.bob, 0).is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_approval(&events[0], acc.alice, acc.bob, value);
            }

            #[ink::test]
            fn increase_allowance_to_self_is_no_op() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.increase_allowance(acc.alice, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.alice), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn decrease_allowance_to_self_is_no_op() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token.decrease_allowance(acc.alice, value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.alice), 0);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn transfer_from_works() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.approve(acc.bob, value).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token
                    .transfer_from(acc.alice, acc.charlie, value, vec![])
                    .is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply - value);
                assert_eq!(token.balance_of(acc.bob), 0);
                assert_eq!(token.balance_of(acc.charlie), value);
            }

            #[ink::test]
            fn transfer_from_decreases_allowance() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);

                assert!(token.approve(acc.bob, 3 * value).is_ok());
                assert_eq!(token.allowance(acc.alice, acc.bob), 3 * value);
                assert_eq!(token.allowance(acc.alice, acc.charlie), 0);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);
                assert_eq!(token.allowance(acc.bob, acc.charlie), 0);

                set_caller::<E>(acc.bob);
                assert!(token
                    .transfer_from(acc.alice, acc.charlie, value, vec![])
                    .is_ok());

                assert_eq!(token.allowance(acc.alice, acc.bob), 2 * value);
                assert_eq!(token.allowance(acc.alice, acc.charlie), 0);
                assert_eq!(token.allowance(acc.bob, acc.alice), 0);
                assert_eq!(token.allowance(acc.bob, acc.charlie), 0);
            }

            #[ink::test]
            fn transfer_from_emits_events() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                assert!(token.approve(acc.bob, 2 * value).is_ok());
                let start = recorded_events().count();

                set_caller::<E>(acc.bob);
                assert!(token
                    .transfer_from(acc.alice, acc.charlie, value, vec![])
                    .is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 2);
                if is_transfer(&events[0]) {
                    assert_transfer(&events[0], Some(acc.alice), Some(acc.charlie), value);
                    assert_approval(&events[1], acc.alice, acc.bob, value);
                } else {
                    assert_approval(&events[0], acc.alice, acc.bob, value);
                    assert_transfer(&events[1], Some(acc.alice), Some(acc.charlie), value);
                }
            }

            #[ink::test]
            fn transfer_from_fails_with_insufficient_allowance() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                assert!(token.approve(acc.bob, value).is_ok());
                let start = recorded_events().count();

                set_caller::<E>(acc.bob);
                assert_eq!(
                    token.transfer_from(acc.alice, acc.charlie, 2 * value, vec![]),
                    Err(PSP22Error::InsufficientAllowance)
                );

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn transfer_from_fails_with_insufficient_balance() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.approve(acc.charlie, 2 * value).is_ok());
                let start = recorded_events().count();

                assert_eq!(token.balance_of(acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.charlie), 2 * value);
                set_caller::<E>(acc.charlie);
                assert_eq!(
                    token.transfer_from(acc.bob, acc.alice, value + 1, vec![]),
                    Err(PSP22Error::InsufficientBalance)
                );
                assert_eq!(token.balance_of(acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.charlie), 2 * value);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn transfer_from_with_not_enough_balance_and_allowance_fails_with_insuficient_allowance(
            ) {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                assert!(token.transfer(acc.bob, value, vec![]).is_ok());
                set_caller::<E>(acc.bob);
                assert!(token.approve(acc.charlie, value).is_ok());
                let start = recorded_events().count();

                assert_eq!(token.balance_of(acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.charlie), value);
                set_caller::<E>(acc.charlie);
                assert_eq!(
                    token.transfer_from(acc.bob, acc.alice, value + 1, vec![]),
                    Err(PSP22Error::InsufficientAllowance)
                );
                assert_eq!(token.balance_of(acc.bob), value);
                assert_eq!(token.allowance(acc.bob, acc.charlie), value);

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn transfer_from_myself_works_without_allowance() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let (supply, value) = (1000, 100);
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                assert!(token
                    .transfer_from(acc.alice, acc.bob, value, vec![])
                    .is_ok());

                assert_eq!(token.total_supply(), supply);
                assert_eq!(token.balance_of(acc.alice), supply - value);
                assert_eq!(token.balance_of(acc.bob), value);

                let events = get_events(start);
                assert_eq!(events.len(), 1);
                assert_transfer(&events[0], Some(acc.alice), Some(acc.bob), value);
            }

            #[ink::test]
            fn transfer_from_for_0_is_no_op() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 1000;
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                set_caller::<E>(acc.bob);
                assert!(token
                    .transfer_from(acc.alice, acc.charlie, 0, vec![])
                    .is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }

            #[ink::test]
            fn transfer_from_to_the_same_address_is_no_op() {
                let acc = default_accounts::<E>();
                set_caller::<E>(acc.alice);
                let supply = 1000;
                let mut token = $constructor(supply);
                let start = recorded_events().count();

                set_caller::<E>(acc.bob);
                assert!(token
                    .transfer_from(acc.alice, acc.alice, 2 * supply, vec![])
                    .is_ok());

                let events = get_events(start);
                assert_eq!(events.len(), 0);
            }
        }
    };
}
