#![cfg_attr(not(feature = "std"), no_std)]

//! This contract provides simple E2E tests for the functions
//! `seal_ecdsa_recover` and `seal_ecdsa_to_eth_address`.

use ink_lang as ink;

#[ink::contract]
mod seal_ecdsa {
    #[ink(storage)]
    pub struct SealEcdsa {}

    impl SealEcdsa {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Tests `seal_ecdsa_recover`.
        #[ink(message)]
        pub fn recover(&self, signature: [u8; 65], message_hash: [u8; 32]) -> [u8; 33] {
            let mut output = [0; 33];
            ink_env::ecdsa_recover(&signature, &message_hash, &mut output);
            output
        }

        /// Tests `seal_ecdsa_to_eth_address`.
        #[ink(message)]
        pub fn to_eth_address(&self, pub_key: [u8; 33]) -> [u8; 20] {
            let mut output = [0; 20];
            ink_env::ecdsa_to_eth_address(&pub_key, &mut output);
            output
        }
    }
}
