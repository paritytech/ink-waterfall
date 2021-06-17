// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the `contract-transfer `example.

use crate::utils::{
    self,
    canvas_ui::{
        Call,
        CanvasUi,
        Upload,
    },
    cargo_contract,
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn contract_must_transfer_value_to_sender(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("contract-transfer/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let contract_addr = canvas_ui.execute_upload(Upload::new(contract_file)).await?;
    let balance_before = canvas_ui.balance_postfix("BOB".to_string()).await?;

    // when
    let _events = canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "give_me")
                .push_value("value", "100")
                .max_gas("25000")
                .caller("BOB"),
        )
        .await
        .expect("failed to execute transaction");

    // then
    let balance_after = canvas_ui.balance_postfix("BOB".to_string()).await?;
    assert_eq!(balance_after - balance_before, 1);
    assert!(utils::canvas_log_contains("requested value: 100000000000000"));
    Ok(())
}

#[waterfall_test]
async fn transfer_exactly_ten_to_contract(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("contract-transfer/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let contract_addr = canvas_ui.execute_upload(Upload::new(contract_file)).await?;

    // when
    let result = canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "was_it_ten")
                .payment("10", "pico")
                .max_gas("25000"),
        )
        .await;

    // then
    assert!(result.is_ok());
    assert!(utils::canvas_log_contains("received payment: 10"));
    Ok(())
}
