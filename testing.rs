#[macro_export]
macro_rules! tests {
    ($contract:ident) => {
        use super::*;
        use ink::env::{test::*, DefaultEnvironment as E};

        type Event = <$contract as ::ink::reflect::ContractEventBase>::Type;

        fn decode_events() -> Vec<Event> {
            recorded_events()
                .map(|e| <Event as scale::Decode>::decode(&mut &e.data[..]).unwrap())
                .collect()
        }

        fn assert_transfer(event: &Event, from_: AccountId, to_: AccountId, value_: u128) {
            if let Event::Transfer(Transfer { from, to, value }) = event {
                assert_eq!(*from, Some(from_), "Transfer event: 'from' mismatch");
                assert_eq!(*to, Some(to_), "Transfer event: 'to' mismatch");
                assert_eq!(*value, value_, "Transfer event: 'value' mismatch");
            } else {
                panic!("Event is not Transfer")
            }
        }

        fn assert_approval(event: &Event, owner_: AccountId, spender_: AccountId, amount_: u128) {
            if let Event::Approval(Approval {
                owner,
                spender,
                amount,
            }) = event
            {
                assert_eq!(*owner, owner_, "Approval event: 'owner' mismatch");
                assert_eq!(*spender, spender_, "Approval event: 'spender' mismatch");
                assert_eq!(*amount, amount_, "Approval event: 'amount' mismatch");
            } else {
                panic!("Event is not Approval")
            }
        }

        #[ink::test]
        fn constructor_works() {
            let acc = default_accounts::<E>();
            set_caller::<E>(acc.alice);
            let supply = 1000;
            let token = $contract::new(supply, None, None, 0);

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
            let mut token = $contract::new(supply, None, None, 0);

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
            let mut token = $contract::new(supply, None, None, 0);

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
            let mut token = $contract::new(supply, None, None, 0);

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
            let mut token = $contract::new(supply, None, None, 0);

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
            let mut token = $contract::new(supply, None, None, 0);

            assert!(token.transfer(acc.bob, value, vec![]).is_ok());
            let events = decode_events();
            assert_eq!(events.len(), 1);
            assert_transfer(&events[0], acc.alice, acc.bob, value);
        }

        #[ink::test]
        fn transfer_0_works() {
            let acc = default_accounts::<E>();
            set_caller::<E>(acc.alice);
            let (supply, value) = (1000, 0);
            let mut token = $contract::new(supply, None, None, 0);

            assert!(token.transfer(acc.bob, value, vec![]).is_ok());
            let events = decode_events();
            assert_eq!(events.len(), 0, "Transferring 0 tokens emitted event");
        }

        #[ink::test]
        fn multiple_transfers_emit_correct_events() {
            let acc = default_accounts::<E>();
            set_caller::<E>(acc.alice);
            let (supply, value) = (1000, 100);
            let mut token = $contract::new(supply, None, None, 0);

            assert!(token.transfer(acc.bob, value, vec![]).is_ok());
            assert!(token.transfer(acc.bob, 2 * value, vec![]).is_ok());
            set_caller::<E>(acc.bob);
            assert!(token.transfer(acc.charlie, 3 * value, vec![]).is_ok());

            let events = decode_events();
            assert_eq!(events.len(), 3);
            assert_transfer(&events[0], acc.alice, acc.bob, value);
            assert_transfer(&events[1], acc.alice, acc.bob, 2 * value);
            assert_transfer(&events[2], acc.bob, acc.charlie, 3 * value);
        }
    };
}
