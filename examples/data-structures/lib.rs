#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod data_structures {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping
    };

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct DataStructures {
        mapping: Mapping<u32, Option<bool>>
    }

    impl DataStructures {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.mapping = Default::default();
            })
        }

        /// Insert the given `value` at `key` into `DataStructures::mapping`.
        ///
        /// Returns the size of the pre-existing value at the specified key if any.
        #[ink(message)]
        pub fn overwrite_key(&mut self, key: u32, value: bool) -> Option<u32> {
            self.mapping.insert_return_size(key, &Some(value))
        }

        /// Removes the `value` at `key`.
        ///
        /// Returns the size of the pre-existing value at the specified key if any.
        #[ink(message)]
        pub fn remove_key(&mut self, key: u32) -> Option<u32> {
            let val: Option<bool> = None;
            self.mapping.insert_return_size(key, &val)
        }
    }
}
