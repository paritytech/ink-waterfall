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

//! Tests for the `ERC-1155 `example.

use crate::{
    uis::{
        Call,
        Result,
        Ui,
        Upload,
    },
    utils::{
        self,
        cargo_contract,
    },
};
use lang_macro::waterfall_test;

#[waterfall_test]
async fn erc1155(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("erc1155/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    ui.execute_transaction(
        Call::new(&contract_addr, "create")
            .caller("BOB")
            .push_value("value", "123"), // initial_supply
    )
    .await
    .expect("failed to execute transaction");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "BOB")
                .push_value("tokenId", "1"),
        )
        .await?;
    assert!(balance == "123,000,000,000,000" || balance == "123.0000 Unit");

    ui.execute_transaction(
        Call::new(&contract_addr, "mint")
            .caller("CHARLIE")
            .push_value("tokenId", "1")
            .push_value("value", "341"), // initial_supply
    )
    .await
    .expect("failed to execute transaction");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "CHARLIE")
                .push_value("tokenId", "1"),
        )
        .await?;
    assert!(balance == "341,000,000,000,000" || balance == "341.0000 Unit");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of_batch")
                .add_item("owners", "BOB")
                .add_item("owners", "CHARLIE")
                .add_item("tokenIds", "0")
                .add_item("tokenIds", "1"),
        )
        .await?;
    assert!(
        balance == "[ 0 123,000,000,000,000 0 341,000,000,000,000 ]"
            || balance == "0:\n0\n1:\n123000000000000\n2:\n0\n3:\n341000000000000"
    );

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "false");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,set_approval_for_all")
            .caller("CHARLIE")
            .push_value("operator", "DAVE")
            .push_value("approved", "true"),
    )
    .await
    .expect("failed to execute transaction");

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "true");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,safe_transfer_from")
            .caller("DAVE")
            .push_value("from", "CHARLIE")
            .push_value("to", "ALICE")
            .push_value("tokenId", "1")
            .push_value("value", "41"),
    )
    .await
    .expect("failed to execute transaction");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "CHARLIE")
                .push_value("tokenId", "1"),
        )
        .await?;
    assert!(balance == "300,000,000,000,000" || balance == "300.0000 Unit");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "ALICE")
                .push_value("tokenId", "1"),
        )
        .await?;
    assert!(balance == "41,000,000,000,000" || balance == "41.0000 Unit");

    ui.execute_transaction(
        Call::new(&contract_addr, "create")
            .caller("ALICE")
            .push_value("value", "99"),
    )
    .await
    .expect("failed to execute transaction");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,safe_batch_transfer_from")
            .caller("ALICE")
            .push_value("from", "ALICE")
            .push_value("to", "FERDIE")
            .add_item("tokenIds", "1")
            .add_item("tokenIds", "2")
            .add_item("values", "41000000000000")
            .add_item("values", "99000000000000"),
    )
    .await
    .expect("failed to execute transaction");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "FERDIE")
                .push_value("tokenId", "1"),
        )
        .await?;
    assert!(balance == "41,000,000,000,000" || balance == "41.0000 Unit");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,balance_of")
                .push_value("owner", "FERDIE")
                .push_value("tokenId", "2"),
        )
        .await?;
    assert!(balance == "99,000,000,000,000" || balance == "99.0000 Unit");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,set_approval_for_all")
            .caller("CHARLIE")
            .push_value("operator", "DAVE")
            .push_value("approved", "false"),
    )
    .await
    .expect("failed to execute transaction");

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "false");

    // TODO
    //   Has to be ignored until https://github.com/paritytech/ink/issues/641 makes `Result::Err`
    //   visible in the UI.
    assert!(
        true || ui
            .execute_transaction(
                Call::new(&contract_addr, "Erc1155,safe_transfer_from")
                    .caller("DAVE")
                    .push_value("from", "CHARLIE")
                    .push_value("to", "ALICE")
                    .push_value("tokenId", "1")
                    .push_value("value", "41")
            )
            .await
            .is_err()
    );

    Ok(())
}

#[waterfall_test]
async fn erc1155_approvals(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("erc1155/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "false");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,set_approval_for_all")
            .caller("CHARLIE")
            .push_value("operator", "DAVE")
            .push_value("approved", "true"),
    )
    .await
    .expect("failed to execute transaction");

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "true");

    ui.execute_transaction(
        Call::new(&contract_addr, "Erc1155,set_approval_for_all")
            .caller("CHARLIE")
            .push_value("operator", "DAVE")
            .push_value("approved", "false"),
    )
    .await
    .expect("failed to execute transaction");

    let is_approved_for_all = ui
        .execute_rpc(
            Call::new(&contract_addr, "Erc1155,is_approved_for_all")
                .push_value("owner", "CHARLIE")
                .push_value("operator", "DAVE"),
        )
        .await?;
    assert_eq!(is_approved_for_all, "false");
    Ok(())
}
