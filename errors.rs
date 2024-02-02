use ink::prelude::string::String;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PSP22Error {
    /// Custom error type for implementation-based errors.
    Custom(String),
    /// Returned when an account does not have enough tokens to complete the operation.
    InsufficientBalance,
    /// Returned if there is not enough allowance to complete the operation.
    InsufficientAllowance,
    /// Returned if recipient's address is zero [deprecated].
    ZeroRecipientAddress,
    /// Returned if sender's address is zero [deprecated].
    ZeroSenderAddress,
    /// Returned if a safe transfer check failed [deprecated].
    SafeTransferCheckFailed(String),
}
