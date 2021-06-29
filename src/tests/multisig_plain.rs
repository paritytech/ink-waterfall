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

//! Tests for the `multisig_plain `example.

use crate::{
    uis::{
        Call,
        Ui,
        Upload,
    },
    utils::{
        self,
        cargo_contract,
    },
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn multisig_works_with_flipper_transaction(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("flipper/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let flipper_contract_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    let manifest_path = utils::example_path("multisig_plain/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(
            Upload::new(contract_file)
                .push_initial_value("requirement", "2")
                .add_item("owners", "ALICE")
                .add_item("owners", "BOB")
                .add_item("owners", "EVE"),
        )
        .await?;

    ui.execute_transaction(
        Call::new(&contract_addr, "submit_transaction")
                .caller("ALICE")
                .push_value("callee", &flipper_contract_addr)
                .push_value("selector", "0x633aa551") // `flip`
                .push_value("input", "0x00")
                .push_value("transferred_value", "0")
                //.push_value("gas_limit", "9999999000"),
                //.push_value("gas_limit", "20000"),
    )
    .await
    .expect("failed to `submit_transaction`");
    let id = "0";
    ui.execute_transaction(
        Call::new(&contract_addr, "confirm_transaction")
            .caller("ALICE")
            .push_value("transId", id),
    )
    .await
    .expect("failed to `confirm_transaction`");

    ui.execute_transaction(
        Call::new(&contract_addr, "confirm_transaction")
            .caller("BOB")
            .push_value("transId", id),
    )
    .await
    .expect("failed to `confirm_transaction`");

    assert_eq!(
        ui.execute_rpc(Call::new(&flipper_contract_addr, "get"))
            .await?,
        "false"
    );

    // when
    ui.execute_transaction(
        Call::new(&contract_addr, "invoke_transaction")
            .caller("ALICE")
            .push_value("transId", id),
    )
    .await
    .expect("failed to `invoke_transaction`");

    // then
    assert_eq!(
        ui.execute_rpc(Call::new(&flipper_contract_addr, "get"))
            .await?,
        "true"
    );

    Ok(())
}

#[waterfall_test]
async fn multisig_works_with_payable_transaction(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("contract-transfer/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let contract_transfer_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    let manifest_path = utils::example_path("multisig_plain/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(
            Upload::new(contract_file)
                .push_initial_value("requirement", "2")
                .add_item("owners", "ALICE")
                .add_item("owners", "BOB")
                .add_item("owners", "EVE"),
        )
        .await?;

    ui.execute_transaction(
        Call::new(&contract_addr, "submit_transaction")
                .caller("ALICE")
                .push_value("callee", &contract_transfer_addr)
                .push_value("selector", "0xcafebabe") // `was_it_ten`
                .push_value("input", "0x00")
                .push_value("transferred_value", "10")
                .push_value("gas_limit", "9999999000"),
    )
    .await
    .expect("failed to `submit_transaction`");
    let id = "0";
    ui.execute_transaction(
        Call::new(&contract_addr, "confirm_transaction")
            .caller("ALICE")
            .push_value("transId", id),
    )
    .await
    .expect("failed to `confirm_transaction`");

    ui.execute_transaction(
        Call::new(&contract_addr, "confirm_transaction")
            .caller("BOB")
            .push_value("transId", id),
    )
    .await
    .expect("failed to `confirm_transaction`");

    // when
    ui.execute_transaction(
        Call::new(&contract_addr, "invoke_transaction")
            .caller("ALICE")
            .push_value("transId", id)
            .payment("10", "pico"),
    )
    .await
    .expect("failed to `invoke_transaction`");

    // then
    assert!(utils::canvas_log_contains("received payment: 10\n"));

    Ok(())
}
