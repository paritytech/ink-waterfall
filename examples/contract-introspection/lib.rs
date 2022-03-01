#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod contract_introspection {
    use ink_env::{
        call::{
            build_call,
            utils::ReturnType,
            ExecutionInput,
            Selector,
        },
        CallFlags,
        DefaultEnvironment,
    };

    #[ink(storage)]
    pub struct ContractInspection {}

    impl ContractInspection {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns `true` if the caller is a contract.
        #[ink(message, selector = 1)]
        pub fn is_caller_contract(&self) -> bool {
            self.env().is_contract(&self.env().caller())
        }

        /// Returns `true` if the caller is the origin of this call.
        #[ink(message, selector = 2)]
        pub fn is_caller_origin(&self) -> bool {
            self.env().caller_is_origin()
        }

        /// Calls into this contract to the [`is_caller_contract`] function
        /// and returns the return value of that function.
        #[ink(message)]
        pub fn calls_is_caller_contract(&self) -> bool {
            build_call::<DefaultEnvironment>()
                .callee(self.env().account_id())
                .exec_input(ExecutionInput::new(Selector::new([0x00, 0x00, 0x00, 0x01])))
                .returns::<ReturnType<bool>>()
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .fire()
                .expect("failed executing call")
        }

        /// Calls into this contract to the [`is_caller_origin`] function
        /// and returns the return value of that function.
        #[ink(message)]
        pub fn calls_is_caller_origin(&self) -> bool {
            build_call::<DefaultEnvironment>()
                .callee(self.env().account_id())
                .exec_input(ExecutionInput::new(Selector::new([0x00, 0x00, 0x00, 0x02])))
                .returns::<ReturnType<bool>>()
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .fire()
                .expect("failed executing call")
        }
    }
}
