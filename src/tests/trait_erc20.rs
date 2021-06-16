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

//! Tests for the `trait-erc20 `example.

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
#[ignore] // TODO until https://github.com/paritytech/canvas-ui/issues/105 is fixed
async fn erc20(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("trait-erc20/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = canvas_ui
        .execute_upload(
            Upload::new(contract_file)
                .caller("BOB")
                .push_initial_value("initialSupply", "1000"),
        )
        .await?;
    assert_eq!(
        canvas_ui
            .execute_rpc(Call::new(&contract_addr, "total_supply"))
            .await?,
        "1000000000000000"
    );
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "balance_of").push_value("owner", "bob")
            )
            .await?,
        "1000000000000000"
    );

    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "transfer")
                .caller("BOB")
                .push_value("to", "ALICE")
                .push_value("value", "500"),
        )
        .await
        .expect("failed to execute transaction");

    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "balance_of").push_value("owner", "ALICE")
            )
            .await?,
        "500000000000000"
    );

    Ok(())
}

#[waterfall_test]
#[ignore] // TODO until https://github.com/paritytech/canvas-ui/issues/105 is fixed
async fn erc20_allowances(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("trait-erc20/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = canvas_ui
        .execute_upload(
            Upload::new(contract_file)
                .caller("BOB")
                .push_initial_value("initialSupply", "1000"),
        )
        .await?;

    // Alice tries to transfer tokens on behalf ob Bob
    // TODO
    //   Has to be ignored until https://github.com/paritytech/ink/issues/641 makes `Result::Err`
    //   visible in the UI.
    assert!(
        true || canvas_ui
            .execute_transaction(
                Call::new(&contract_addr, "transfer_from")
                    .caller("ALICE")
                    .push_value("from: AccountId", "BOB")
                    .push_value("to: AccountId", "ALICE")
                    .push_value("value", "400"),
            )
            .await
            .is_err()
    );

    // Bob approves Alice being able to withdraw up the `value` amount on his behalf.
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "approve")
                .caller("BOB")
                .push_value("spender", "ALICE")
                .push_value("value", "600"),
        )
        .await
        .expect("`approve` must succeed");
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "allowance")
                    .push_value("owner", "BOB")
                    .push_value("spender", "ALICE")
            )
            .await?,
        "600000000000000"
    );

    // Alice tries again to transfer tokens on behalf ob Bob
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "transfer_from")
                .caller("ALICE")
                .push_value("from: AccountId", "BOB")
                .push_value("to: AccountId", "ALICE")
                .push_value("value", "400"),
        )
        .await
        .expect("second `transfer_from` must succeed");
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "balance_of").push_value("owner", "ALICE")
            )
            .await?,
        "400000000000000"
    );
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "balance_of").push_value("owner", "BOB")
            )
            .await?,
        "600000000000000"
    );

    // Alice tries to transfer even more tokens on behalf ob Bob, this time exhausting the allowance
    // TODO
    //   Has to be ignored until https://github.com/paritytech/ink/issues/641 makes `Result::Err`
    //   visible in the UI.
    assert!(
        true || canvas_ui
            .execute_transaction(
                Call::new(&contract_addr, "transfer_from")
                    .caller("ALICE")
                    .push_value("from: AccountId", "BOB")
                    .push_value("to: AccountId", "ALICE")
                    .push_value("value", "201"),
            )
            .await
            .is_err()
    );

    // Balance of Bob must have stayed the same
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "balance_of").push_value("owner", "BOB")
            )
            .await?,
        "600000000000000"
    );

    Ok(())
}