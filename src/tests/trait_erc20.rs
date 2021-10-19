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
async fn trait_erc20(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("trait-erc20/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(
            Upload::new(contract_file)
                .caller("BOB")
                .push_initial_value("initialSupply", "1000"),
        )
        .await?;
    let total_supply = ui
        .execute_rpc(Call::new(&contract_addr, "BaseErc20,total_supply"))
        .await?;
    assert!(total_supply == "1,000,000,000,000,000" || total_supply == "1.0000 kUnit");
    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,balance_of").push_value("owner", "bob"),
        )
        .await?;
    assert!(balance == "1,000,000,000,000,000" || balance == "1.0000 kUnit");

    ui.execute_transaction(
        Call::new(&contract_addr, "BaseErc20,transfer")
            .caller("BOB")
            .push_value("to", "ALICE")
            .push_value("value", "500"),
    )
    .await
    .expect("failed to execute transaction");

    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,balance_of")
                .push_value("owner", "ALICE"),
        )
        .await?;
    assert!(balance == "500,000,000,000,000" || balance == "500.0000 Unit");

    Ok(())
}

#[waterfall_test]
async fn trait_erc20_allowances(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("trait-erc20/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
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
        true || ui
            .execute_transaction(
                Call::new(&contract_addr, "BaseErc20,transfer_from")
                    .caller("ALICE")
                    .push_value("from: AccountId", "BOB")
                    .push_value("to: AccountId", "ALICE")
                    .push_value("value", "400"),
            )
            .await
            .is_err()
    );

    // Bob approves Alice being able to withdraw up the `value` amount on his behalf.
    ui.execute_transaction(
        Call::new(&contract_addr, "BaseErc20,approve")
            .caller("BOB")
            .push_value("spender", "ALICE")
            .push_value("value", "600"),
    )
    .await
    .expect("`approve` must succeed");
    let allowance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,allowance")
                .push_value("owner", "BOB")
                .push_value("spender", "ALICE"),
        )
        .await?;
    assert!(allowance == "600,000,000,000,000" || allowance == "600.0000 Unit");

    // Alice tries again to transfer tokens on behalf ob Bob
    ui.execute_transaction(
        Call::new(&contract_addr, "BaseErc20,transfer_from")
            .caller("ALICE")
            .push_value("from: AccountId", "BOB")
            .push_value("to: AccountId", "ALICE")
            .push_value("value", "400"),
    )
    .await
    .expect("second `transfer_from` must succeed");
    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,balance_of")
                .push_value("owner", "ALICE"),
        )
        .await?;
    assert!(balance == "400,000,000,000,000" || balance == "400.0000 Unit");
    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,balance_of").push_value("owner", "BOB"),
        )
        .await?;
    assert!(balance == "600,000,000,000,000" || balance == "600.0000 Unit");

    // Alice tries to transfer even more tokens on behalf ob Bob, this time exhausting the allowance
    // TODO
    //   Has to be ignored until https://github.com/paritytech/ink/issues/641 makes `Result::Err`
    //   visible in the UI.
    assert!(
        true || ui
            .execute_transaction(
                Call::new(&contract_addr, "BaseErc20,transfer_from")
                    .caller("ALICE")
                    .push_value("from: AccountId", "BOB")
                    .push_value("to: AccountId", "ALICE")
                    .push_value("value", "201"),
            )
            .await
            .is_err()
    );

    // Balance of Bob must have stayed the same
    let balance = ui
        .execute_rpc(
            Call::new(&contract_addr, "BaseErc20,balance_of").push_value("owner", "BOB"),
        )
        .await?;
    assert!(balance == "600,000,000,000,000" || balance == "600.0000 Unit");

    Ok(())
}
