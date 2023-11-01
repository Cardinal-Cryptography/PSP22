#[macro_export]
macro_rules! tests {
    ($contract:ident) => {
        use super::{$contract, PSP22};
        use ink::env::{test::*, DefaultEnvironment as E, Error};

        #[ink::test]
        fn constructor_works() -> Result<(), Error> {
            let supply = 1000;
            let acc = default_accounts::<E>();
            set_caller::<E>(acc.alice);
            let token = $contract::new(supply, None, None, 0);
            assert_eq!(token.total_supply(), supply);
            assert_eq!(token.balance_of(acc.alice), supply);
            assert_eq!(token.balance_of(acc.bob), 0);
            assert_eq!(token.allowance(acc.alice, acc.alice), 0);
            assert_eq!(token.allowance(acc.alice, acc.bob), 0);
            assert_eq!(token.allowance(acc.bob, acc.alice), 0);
            Ok(())
        }
    };
}
