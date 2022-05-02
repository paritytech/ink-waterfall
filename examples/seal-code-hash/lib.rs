#![cfg_attr(not(feature = "std"), no_std)]

//! This contract provides a simple E2E test for the functions
//! `seal_own_code_hash` and `seal_code_hash`.

use ink_lang as ink;

#[ink::contract]
mod seal_code_hash {
    #[ink(storage)]
    pub struct SealCodeHash {}

    impl SealCodeHash {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the code hash of this contract.
        #[ink(message)]
        pub fn own_code_hash(&self) -> Hash {
            self.env().own_code_hash().expect("must exist")
        }

        /// Returns the code hash of the contract at `account_id`.
        #[ink(message)]
        pub fn code_hash(&self, account_id: AccountId) -> Hash {
            self.env().code_hash(&account_id).expect("must exist")
        }
    }
}
